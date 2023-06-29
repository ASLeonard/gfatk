use std::path::PathBuf;

use crate::load::load_gfa;
use crate::utils::{self, GFAGraphLookups};
use crate::{gfa::gfa::GFAtk, gfa::graph::segments_subgraph, load::load_gfa_stdin};
use anyhow::{bail, Result};
use petgraph::algo::{is_cyclic_directed,tarjan_scc};
use petgraph::graph::{Graph, NodeIndex, UnGraph};
use std::collections::HashMap;

/// Enumeration of the genomes we are interested in.
#[derive(PartialEq, Clone, Copy)]
pub enum GenomeType {
    /// The mitochondrial genome
    Mitochondria,
    /// The chloroplast/plastid genome
    Chloroplast,
    /// This will process the stats and return nothing
    None,
}

/// The statistics associated with a subgraph in a GFA.
#[derive(Clone, Debug)]
pub struct Stat {
    /// Arbitrary index of the subgraph(s).
    pub index: usize,
    /// The average GC% across a subgraph.
    pub gc: f32,
    /// The node count of the graph.
    pub node_count: usize,
    /// The edge count of the graph.
    pub edge_count: usize,
    /// The segments of the (sub)graph.
    pub graph_indices_subgraph: GFAGraphLookups,
    /// The average coverage across a subgraph.
    pub cov: f32,
    /// Names of the segments.
    pub segments: Vec<usize>,
    /// Total sequence length of all the segments.
    pub total_sequence_length: usize,
    /// Whether the subgraph is circular
    /// (only applies to mitochondrial genomes).
    pub is_circular: bool,
}

/// A vector of `Stat`.
pub struct Stats(pub Vec<Stat>);

impl Stats {
    /// Add a new `Stat` to `Stats`.
    pub fn push(&mut self, stat: Stat) {
        let stats = &mut self.0;
        stats.push(stat);
    }

    /// Print tabular form of [`Stats`] to STDOUT.
    pub fn print_tabular(&self) {
        let headers = [
            "subgraph_index",
            "gc",
            "node_count",
            "edge_count",
            "coverage",
            "segments",
            "total_seq_len",
            "is_circular",
        ];
        // print headers
        println!("{}", headers.join("\t"));
        // fill the rows
        for Stat {
            index,
            gc,
            node_count,
            edge_count,
            graph_indices_subgraph: _,
            cov,
            segments,
            total_sequence_length,
            is_circular,
        } in &self.0
        {
            let segment_string = segments
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(",");

            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                index,
                gc,
                node_count,
                edge_count,
                cov,
                segment_string,
                total_sequence_length,
                is_circular
            );
        }
    }

    /// Extract the putative mitochondrial/chloroplast genome from a GFA
    /// file.
    ///
    /// The upper and lower limits of genome size and GC content are supplied through the
    /// CLI. As the defaults will be different, the same function is accessed entry points
    /// in the CLI.

    pub fn extract_organelle(
        &mut self,
        size_lower: usize,
        mut size_upper: usize,
        gc_lower: f32,
        gc_upper: f32,
    ) -> Result<Vec<usize>> {
        // just going to hard code these for the moment
        // these values are taken from GoaT
        // these values are within 2 stddevs of the mean,
        // so most chloroplasts should pop out

        // adjust because of overlaps between segments
        // kind of arbitrary...
        let seq_len_adj = 20000;
        size_upper += seq_len_adj;

        let stat_vec = &mut self.0;
        let stat_vec_len = stat_vec.len();
        // filter this vector to have stats in line with the span/gc

        if stat_vec_len > 0 {
            // apply the filter
            let stat_vec: Vec<&Stat> = stat_vec
                .iter()
                .filter(
                    |Stat {
                         index: _,
                         gc,
                         cov: _,
                         segments: _,
                         total_sequence_length,
                         is_circular: _,
                         node_count: _,
                         edge_count: _,
                         graph_indices_subgraph: _,
                     }| {
                        (gc > &gc_lower && gc < &gc_upper)
                            && (total_sequence_length > &size_lower
                                && total_sequence_length < &size_upper)
                    },
                )
                .collect();
            // let's return all the filtered segments
            // and see if it works for now
            match stat_vec.len() {
                0 => bail!("No subgraphs within the bounds:\nsize_upper: {size_upper}\nsize_lower: {size_lower}\ngc_upper: {gc_upper}\ngc_lower: {gc_lower}\nTry changing limits?"),
                1.. => {
                    // extract all segments
                    let segments = stat_vec.iter().flat_map(|Stat { segments, .. }| segments.clone()).collect();
                    Ok(segments)
                },
                _ => unreachable!()
            }
        } else {
            bail!("There were no segments to be extracted. Check input GFA file.");
        }
    }
}

// I've handled 'further' here really badly...
// I want node indices & segment names printed too (maybe optionally.)

/// Internal function called in `gfatk stats`.
///
/// Used in `gfatk stats`, `gfatk extract-mito`, and `gfatk extract-chloro`.
///
/// For example:
/// ```bash
/// gfatk stats in.gfa
/// ```
pub fn stats(
    matches: &clap::ArgMatches,
    genome_type: GenomeType,
) -> Result<Option<(GFAtk, Vec<usize>)>> {
    let gfa_file = matches.get_one::<PathBuf>("GFA");
    let tabular = matches.get_flag("tabular");
    // only passed through extract_mito
    let mito_args = if matches!(genome_type, GenomeType::Mitochondria) {
        let size_lower = *matches
            .get_one::<usize>("size-lower")
            .expect("defaulted by clap");
        let size_upper = *matches
            .get_one::<usize>("size-upper")
            .expect("defaulted by clap");
        let gc_lower = *matches
            .get_one::<f32>("gc-lower")
            .expect("defaulted by clap");
        let gc_upper = *matches
            .get_one::<f32>("gc-upper")
            .expect("defaulted by clap");
        Some((size_lower, size_upper, gc_lower, gc_upper))
    } else {
        None
    };
    // only required for extract_chloro
    let chloro_args = if matches!(genome_type, GenomeType::Chloroplast) {
        let size_lower = *matches
            .get_one::<usize>("size-lower")
            .expect("defaulted by clap");
        let size_upper = *matches
            .get_one::<usize>("size-upper")
            .expect("defaulted by clap");
        let gc_lower = *matches
            .get_one::<f32>("gc-lower")
            .expect("defaulted by clap");
        let gc_upper = *matches
            .get_one::<f32>("gc-upper")
            .expect("defaulted by clap");
        Some((size_lower, size_upper, gc_lower, gc_upper))
    } else {
        None
    };

    let gfa = match gfa_file {
        Some(f) => {
            let ext = f.extension();
            match ext {
                Some(e) => {
                    if e == "gfa" {
                        GFAtk(load_gfa(f)?)
                    } else {
                        bail!("Input is not a GFA.")
                    }
                }
                None => bail!("Could not read file."),
            }
        }
        None => match utils::is_stdin() {
            true => GFAtk(load_gfa_stdin(std::io::stdin().lock())?),
            false => bail!(
                "No input from STDIN. Run `gfatk {} -h` for help.",
                match genome_type {
                    GenomeType::Chloroplast => "extract-chloro",
                    GenomeType::Mitochondria => "extract-mito",
                    GenomeType::None => "stats",
                }
            ),
        },
    };

    // load gfa into graph structure
    let (graph_indices, gfa_graph) = gfa.into_digraph()?;
    let X = tarjan_scc(&gfa_graph.0);
    let Y = invert(graph_indices);
    for SCC in X.iter(){
      if SCC.len() > 5 {
        let mut Z: Vec<_> = SCC.iter().map(|x| Y[x]).collect::<Vec<_>>();
        Z.sort();
        println!("{} {}",Z.first().unwrap()-1,Z.last().unwrap()+1);
      }
    }

    Ok(None)
}

fn invert(map: HashMap<usize,NodeIndex>) -> HashMap<NodeIndex,usize> {
    let mut invert = HashMap::new();
    for (key, value) in map.into_iter() {
        invert.insert(value, key);
    }
    return invert;
}

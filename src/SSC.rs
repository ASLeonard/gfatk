use std::path::PathBuf;

use crate::load::load_gfa;
//use crate::utils::{self, GFAGraphLookups};
//use crate::{gfa::graph::segments_subgraph, load::load_gfa_stdin};
use crate::{gfa::graph::GFAdigraph};
use crate::{gfa::gfa::into_digraph};
use anyhow::{bail, Result};
use petgraph::algo::{is_cyclic_directed,tarjan_scc};
use petgraph::graph::{Graph, NodeIndex, UnGraph};
use std::collections::HashMap;

pub fn get_strong_terminal_nodes(
    matches: &clap::ArgMatches,
) -> Result<Option<(GFAdigraph, Vec<usize>)>> {
    let gfa_file = matches.get_one::<PathBuf>("GFA").expect("Shit on it");
    
    let gfa = load_gfa(gfa_file).expect("k");
    
    /*
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
    };
    */

    // load gfa into graph structure
    let (node_to_index, gfa_graph) = into_digraph(gfa)?;
    eprintln!("[+]\tFinished reading GFA into a directed graph.");
    let SSCs = tarjan_scc(&gfa_graph.0);

    eprintln!("[+]\tFinished reading GFA into a directed graph.");
    let index_to_node = invert(node_to_index);

    let min_length = *matches.get_one::<usize>("Size").expect("defaulted by clap");

    for SCC in SSCs.iter(){
      if SCC.len() >= min_length {
        let mut nodes: Vec<_> = SCC.iter().map(|x| index_to_node[x]).collect::<Vec<_>>();
        nodes.sort();
        println!("{} {}",nodes.first().unwrap()-1,nodes.last().unwrap()+1);
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

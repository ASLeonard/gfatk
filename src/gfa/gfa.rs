use crate::gfa::{
    gfa_string,
    graph::{segments_subgraph, GFAdigraph},
};
//use crate::path::GFAPath;
//use crate::stats::GenomeType;
//use crate::utils::{
//    self, get_edge_coverage, parse_cigar, reverse_complement, GFAGraphLookups, GFAGraphPair,
//};
use anyhow::{bail, Context, Result};
use gfa::gfa::{GFA};
use petgraph::graph::{Graph, NodeIndex, UnGraph};
use std::collections::HashMap;

/// A wrapper around GFA from the gfa crate
/// TODO: make GFAtk generic for any segment name, not just usize.
    /// Returns a tuple of GFAGraphLookups (a struct of indices/node names) and an directed GFA graph structure.
    ///
    /// Most functionality of this binary is on directed graph structures
    pub fn into_digraph(&gfa) -> Result<(HashMap::<usize,NodeIndex>, GFAdigraph)> {
        //let gfa = &self.0;
        eprintln!("[+]\tReading GFA into a directed graph.");
        let mut gfa_graph: Graph<usize, ()> = Graph::with_capacity(gfa.segments.len(),gfa.links.len());

        eprintln!("[+]\tPopulating nodes.");
        //let mut graph_indices = GFAGraphLookups::new();

        let mut graph_indices = HashMap::<usize,NodeIndex>::new();
        // read the segments into graph nodes
        // save the indexes for populating the edges
        for node in &gfa.segments {
            let index = gfa_graph.add_node(node.name);
            graph_indices.insert(node.name,index);
            //graph_indices.push(GFAGraphPair {
            //    node_index: index,
            //    seg_id: node.name,
            //});
        }
        eprintln!("[+]\tPopulating edges for {}.",gfa.links.len());
        // populate the edges
        for edge in &gfa.links {
            let from = edge.from_segment;
            let to = edge.to_segment;

            let from_index = graph_indices[&from];//.seg_id_to_node_index(from)?;
            let to_index = graph_indices[&to];//.seg_id_to_node_index(to)?;

            // add the edges
            gfa_graph.add_edge(from_index, to_index, ());//, (from_orient, to_orient, Some(ec)));
        }

        Ok((graph_indices, GFAdigraph(gfa_graph)))
    }

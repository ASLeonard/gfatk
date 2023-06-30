//use crate::gfa::gfa::GFAtk;
//use crate::utils::{format_usize_to_kb, GFAGraphLookups};
use anyhow::{bail, Context, Result};
use gfa::gfa::Orientation;
use gfa::gfa::GFA;
use gfa::optfields::OptFields;
use itertools::Itertools;
use petgraph::{
    graph::{Graph, IndexType, NodeIndex},
    visit::{EdgeRef, IntoNodeIdentifiers, IntoNodeReferences, NodeIndexable, NodeRef},
    Directed,
    Direction::Outgoing,
    Undirected,
};
use std::collections::HashMap;
use std::collections::HashSet;

/// A wrapper of petgraph's undirected `Graph` struct, applied to a GFA. No weights.


// weights are the orientations, used at various points, and an optional
// coverage weight, used in gfatk linear.
// GFA's should always specify Links in a specific direction..?
// so digraphs should be where all the functionality lies.

/// A wrapper of petgraph's directed `Graph` struct, applied to a GFA. The edge weights included are the `Orientation`'s of the adjacent segments, and the coverage of this edge.
pub struct GFAdigraph(pub Graph<usize, ()>);

impl GFAdigraph {
    /// The main function called from `gfatk dot`.
    ///
    /// It is a somewhat modified, simplified version of this:
    /// <https://docs.rs/petgraph/latest/src/petgraph/dot.rs.html#1-349>
    ///
    /// Generating a DOT language output of a GFA file.
    // we want weakly connected components, as there may only be an edge in one
    // orientation (perhaps unlikely... but still)

    /// Split the GFA digraph into subgraphs which are the weakly connected components of the graph.
    ///
    /// Taken from <https://github.com/Qiskit/retworkx/blob/79900cf8da0c0665ac5ce1ccb0f57373434b14b8/src/connectivity/mod.rs>

    /// The main function called from `gfatk linear`.
    ///
    /// This function will generate the longest path through the GFA, by
    /// filtering the output of `all_paths`, and choosing the path with
    /// the highest cumulative edge coverage.

    /// Simple wrapper of `Graph.node_count()` in petgraph.
    pub fn node_count(&self) -> usize {
        let gfa_graph = &self.0;

        gfa_graph.node_count()
    }

    /// Simple wrapper of `Graph.edge_count()` in petgraph.
    pub fn edge_count(&self) -> usize {
        let gfa_graph = &self.0;

        gfa_graph.edge_count()
    }

}

/// A function generic over certain types of `Directed` petgraph `Graph`s.
///
/// Given a graph, a start node, an end node, and optionally a map of the coverage of each node, compute all simple paths between these nodes.
///
/// Modified from: <https://github.com/Ninjani/rosalind/blob/e22ecf2c9f0935d970b137684029957c0850d63f/t_ba11b/src/lib.rs>
pub fn all_paths<T, U, Ix: IndexType>(
    graph: &Graph<T, U, Directed, Ix>,
    start_node: NodeIndex<Ix>,
    end_node: NodeIndex<Ix>,
    rel_coverage_map: Option<&HashMap<NodeIndex<Ix>, usize>>,
    index: usize,
) -> Result<Vec<Vec<NodeIndex<Ix>>>> {
    match rel_coverage_map {
        Some(cov_map) => {
            // for the set of visited nodes
            let mut visited = HashMap::new();
            visited.insert(start_node, 1);
            // a counter for recursion depth.
            let depth = 0;
            match recursive_path_finder_incl_coverage(
                graph,
                start_node,
                end_node,
                &mut visited,
                cov_map,
                depth,
                index,
            ) {
                Some(p) => p,
                None => {
                    // copy of the chunk below!
                    // so if we go past the self imposed stack limit
                    // we default to our other (not including coverage) method.
                    let mut visited = HashSet::new();
                    visited.insert(start_node);
                    recursive_path_finder_no_coverage(graph, start_node, end_node, &mut visited)
                }
            }
        }
        None => {
            let mut visited = HashSet::new();
            visited.insert(start_node);
            recursive_path_finder_no_coverage(graph, start_node, end_node, &mut visited)
        }
    }
}

/// A recursion depth limit, so we don't hit a stack overflow
/// and instead, abort and call another function.
///
/// Why is it 1000? Seemed sensible, and that's what python's is.
const MAX_RECURSION_DEPTH: usize = 1000;

/// Function called by `all_paths` where a `HashMap` is supplied instead of a
/// `HashSet` in order to keep track of how many times a segment/node has been
/// passed in a path.
///
/// Should be no more stack overflows.
fn recursive_path_finder_incl_coverage<T, U, Ix: IndexType>(
    graph: &Graph<T, U, Directed, Ix>,
    start_node: NodeIndex<Ix>,
    end_node: NodeIndex<Ix>,
    visited: &mut HashMap<NodeIndex<Ix>, usize>,
    rel_coverage_map: &HashMap<NodeIndex<Ix>, usize>,
    depth: usize,
    index: usize,
) -> Option<Result<Vec<Vec<NodeIndex<Ix>>>>> {
    if depth > MAX_RECURSION_DEPTH {
        eprint!(
            "\r[-]\tRecursion depth limit ({}) exceeded in permutation {}. Switching to default path finder.",
            MAX_RECURSION_DEPTH,
            index + 1
        );
        return None;
    }
    // if the start node is the same as the end
    // the path is just to the end node
    if start_node == end_node {
        Some(Ok(vec![vec![end_node]]))
    } else {
        let mut paths = Vec::new();
        for edge in graph.edges_directed(start_node, Outgoing) {
            let next_node = edge.target();

            let test = *rel_coverage_map.get(&next_node).unwrap();

            if !visited.contains_key(&next_node) || *visited.get(&next_node).unwrap() != test {
                *visited.entry(next_node).or_insert(0) += 1;
                let descendant_paths = match recursive_path_finder_incl_coverage(
                    graph,
                    next_node,
                    end_node,
                    visited,
                    rel_coverage_map,
                    depth + 1,
                    index,
                ) {
                    // can I get rid of this unwrap? is it safe?
                    Some(p) => p.unwrap(),
                    None => return None,
                };
                visited.remove(&next_node);
                paths.extend(
                    descendant_paths
                        .into_iter()
                        .map(|path| {
                            let mut new_path = vec![start_node];
                            new_path.extend(path);
                            new_path
                        })
                        .collect::<Vec<_>>(),
                )
            }
        }
        Some(Ok(paths))
    }
}

/// A safer and more reliable alternative to `recursive_path_finder_incl_coverage` where
/// a `HashSet` determines whether a segment/node has already been seen or not.
fn recursive_path_finder_no_coverage<T, U, Ix: IndexType>(
    graph: &Graph<T, U, Directed, Ix>,
    start_node: NodeIndex<Ix>,
    end_node: NodeIndex<Ix>,
    visited: &mut HashSet<NodeIndex<Ix>>,
) -> Result<Vec<Vec<NodeIndex<Ix>>>> {
    if start_node == end_node {
        Ok(vec![vec![end_node]])
    } else {
        let mut paths = Vec::new();
        for edge in graph.edges_directed(start_node, Outgoing) {
            let next_node = edge.target();
            if !visited.contains(&next_node) {
                visited.insert(next_node);
                let descendant_paths =
                    recursive_path_finder_no_coverage(graph, next_node, end_node, visited)?;
                visited.remove(&next_node);
                paths.extend(
                    descendant_paths
                        .into_iter()
                        .map(|path| {
                            let mut new_path = vec![start_node];
                            new_path.extend(path);
                            new_path
                        })
                        .collect::<Vec<_>>(),
                )
            }
        }
        Ok(paths)
    }
}

/// Returns a subgraph GFA that only contains elements with the provided segment names.
///
/// Taken from <https://github.com/chfi/rs-gfa-utils/blob/master/src/subgraph.rs>
pub fn segments_subgraph<T: OptFields + Clone>(
    gfa: &GFA<usize, T>,
    segment_names: Vec<usize>,
) -> GFA<usize, T> {
    let segments = gfa
        .segments
        .iter()
        .filter(|s| segment_names.contains(&s.name))
        .cloned()
        .collect();

    let links = gfa
        .links
        .iter()
        .filter(|l| {
            segment_names.contains(&l.from_segment) && segment_names.contains(&l.to_segment)
        })
        .cloned()
        .collect();

    let containments = gfa
        .containments
        .iter()
        .filter(|l| {
            segment_names.contains(&l.container_name) && segment_names.contains(&l.contained_name)
        })
        .cloned()
        .collect();

    let paths: Vec<_> = gfa
        .paths
        .iter()
        .filter(|p| p.iter().any(|(s, _)| segment_names.contains(&s)))
        .cloned()
        .collect();

    GFA {
        header: gfa.header.clone(),
        segments,
        links,
        paths,
        containments,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // we want to make a test graph to play with
    // make the inner graph representation of:
    // ./examples/mito_NC_037304.1.MZ323108.1.fasta.BOTH.HiFiMapped.bam.filtered.1k.gfa

    fn make_graph() -> GFAdigraph {
        let mut graph = Graph::<usize, (Orientation, Orientation, Option<i64>)>::new();

        // node weights are usize
        let node0 = graph.add_node(0);
        let node1 = graph.add_node(1);
        let node2 = graph.add_node(2);
        let node3 = graph.add_node(3);
        let node4 = graph.add_node(4);
        let node5 = graph.add_node(5);

        // we create the following graph
        //
        //  0 <-----> 3 <-----> 1
        //    \     / ˄ \     /
        //      \ /   |   \ /
        //      / \   |   / \
        //    /     \ ˅ /     \
        //  5 <-----> 2 <-----> 4
        //

        graph.extend_with_edges(&[
            (
                node0,
                node3,
                (Orientation::Backward, Orientation::Backward, Some(379)),
            ),
            (
                node0,
                node2,
                (Orientation::Forward, Orientation::Backward, Some(338)),
            ),
            (
                node1,
                node3,
                (Orientation::Backward, Orientation::Backward, Some(380)),
            ),
            (
                node1,
                node2,
                (Orientation::Forward, Orientation::Backward, Some(374)),
            ),
            (
                node2,
                node4,
                (Orientation::Backward, Orientation::Forward, Some(347)),
            ),
            (
                node2,
                node5,
                (Orientation::Backward, Orientation::Forward, Some(399)),
            ),
            (
                node2,
                node1,
                (Orientation::Forward, Orientation::Backward, Some(374)),
            ),
            (
                node2,
                node0,
                (Orientation::Forward, Orientation::Backward, Some(338)),
            ),
            (
                node3,
                node5,
                (Orientation::Backward, Orientation::Backward, Some(397)),
            ),
            (
                node3,
                node4,
                (Orientation::Backward, Orientation::Backward, Some(349)),
            ),
            (
                node3,
                node1,
                (Orientation::Forward, Orientation::Forward, Some(380)),
            ),
            (
                node3,
                node0,
                (Orientation::Forward, Orientation::Forward, Some(379)),
            ),
            (
                node4,
                node2,
                (Orientation::Backward, Orientation::Forward, Some(347)),
            ),
            (
                node4,
                node3,
                (Orientation::Forward, Orientation::Forward, Some(349)),
            ),
            (
                node5,
                node2,
                (Orientation::Backward, Orientation::Forward, Some(399)),
            ),
            (
                node5,
                node3,
                (Orientation::Forward, Orientation::Forward, Some(397)),
            ),
        ]);

        GFAdigraph(graph)
    }

    // there are 6 nodes in this graph
    #[test]
    fn test_node_count() {
        let graph = make_graph();

        assert_eq!(graph.node_count(), 6);
    }

    // there are 16 edges in this graph (incl +/- orientations)
    #[test]
    fn test_edge_count() {
        let graph = make_graph();

        assert_eq!(graph.edge_count(), 16);
    }

    // there are two possible paths between node indexes 0 and 2
    #[test]
    fn test_path_generation() {
        let graph = make_graph();

        let paths = all_paths(&graph.0, NodeIndex::new(0), NodeIndex::new(2), None, 0).unwrap();

        // there should be two paths
        let path1: Vec<NodeIndex> = vec![NodeIndex::new(0), NodeIndex::new(2)];
        let path2: Vec<NodeIndex> = vec![
            NodeIndex::new(0),
            NodeIndex::new(3),
            NodeIndex::new(1),
            NodeIndex::new(2),
        ];
        let path3: Vec<NodeIndex> = vec![
            NodeIndex::new(0),
            NodeIndex::new(3),
            NodeIndex::new(4),
            NodeIndex::new(2),
        ];
        let path4: Vec<NodeIndex> = vec![
            NodeIndex::new(0),
            NodeIndex::new(3),
            NodeIndex::new(5),
            NodeIndex::new(2),
        ];

        assert!(paths.contains(&path1));
        assert!(paths.contains(&path2));
        assert!(paths.contains(&path3));
        assert!(paths.contains(&path4));
    }

    //
    #[test]
    fn test_path_generation_incl_node_cov() {
        let graph = make_graph();

        let mut map: HashMap<NodeIndex, usize> = HashMap::new();

        // we can provide a map to say we want to visit certain nodes twice
        map.insert(NodeIndex::new(0), 1);
        map.insert(NodeIndex::new(1), 1);
        map.insert(NodeIndex::new(2), 2);
        map.insert(NodeIndex::new(3), 2);
        map.insert(NodeIndex::new(4), 1);
        map.insert(NodeIndex::new(5), 1);

        let lookup = GFAGraphLookups(vec![
            crate::utils::GFAGraphPair {
                node_index: NodeIndex::new(0),
                seg_id: 4,
            },
            crate::utils::GFAGraphPair {
                node_index: NodeIndex::new(2),
                seg_id: 6,
            },
            crate::utils::GFAGraphPair {
                node_index: NodeIndex::new(5),
                seg_id: 9,
            },
            crate::utils::GFAGraphPair {
                node_index: NodeIndex::new(3),
                seg_id: 7,
            },
            crate::utils::GFAGraphPair {
                node_index: NodeIndex::new(1),
                seg_id: 5,
            },
            crate::utils::GFAGraphPair {
                node_index: NodeIndex::new(4),
                seg_id: 8,
            },
        ]);

        // generate the paths
        let paths = graph.all_paths_all_node_pairs(&lookup, Some(&map));

        // either this path
        let longest_path1: Vec<NodeIndex> = vec![
            NodeIndex::new(2),
            NodeIndex::new(5),
            NodeIndex::new(3),
            NodeIndex::new(1),
            NodeIndex::new(2),
            NodeIndex::new(4),
            NodeIndex::new(3),
            NodeIndex::new(0),
        ];

        // or this path
        let longest_path2: Vec<NodeIndex> = vec![
            NodeIndex::new(2),
            NodeIndex::new(4),
            NodeIndex::new(3),
            NodeIndex::new(1),
            NodeIndex::new(2),
            NodeIndex::new(5),
            NodeIndex::new(3),
            NodeIndex::new(0),
        ];

        // will be chosen
        let both = vec![longest_path1, longest_path2];

        let path = &paths.unwrap().0.iter().map(|(a, _)| *a).collect::<Vec<_>>();

        assert!(both.contains(path));
    }
}

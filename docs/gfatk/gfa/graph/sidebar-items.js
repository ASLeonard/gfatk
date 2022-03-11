initSidebarItems({"fn":[["all_paths","A function generic over certain types of `Directed` petgraph `Graph`s."],["recursive_path_finder_incl_coverage","Function called by `all_paths` where a `HashMap` is supplied instead of a `HashSet` in order to keep track of how many times a segment/node has been passed in a path."],["recursive_path_finder_no_coverage","A safer and more reliable alternative to `recursive_path_finder_incl_coverage` where a `HashSet` determines whether a segment/node has already been seen or not."],["segments_subgraph","Returns a subgraph GFA that only contains elements with the provided segment names."]],"struct":[["GFAdigraph","A wrapper of petgraph’s directed `Graph` struct, applied to a GFA. The edge weights included are the `Orientation`’s of the adjacent segments, and the coverage of this edge."],["GFAungraph","A wrapper of petgraph’s undirected `Graph` struct, applied to a GFA. No weights."]]});
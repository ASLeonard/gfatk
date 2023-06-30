//! `gfatk` is a tool for Graphical Fragment Assembly (GFA) manipulation.
//!
//! GFA's are at their heart, simple, directed graphs. As such, all internal
//! representations of GFA's are [`petgraph::Graph`'s](https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html).
//!
//! `gfatk` is designed mainly for reasonably small GFA files, and was designed for
//! de-tangling and linearising plant mitochondrial genomes on the command line.
//!
//! Almost all of the core functionality of `gfatk` resides in the [`gfatk::gfa`](./gfa/index.html)
//! module. The other modules are entry points for the command line application.

/// Make a DOT language representation of a GFA.
pub mod SSC;
pub mod load;
pub mod gfa;
//pub mod path;

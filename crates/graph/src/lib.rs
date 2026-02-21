pub mod graph;

// Re-export digraph functionality from walrs_digraph
pub mod digraph {
  pub use walrs_digraph::*;
}

pub use graph::*;

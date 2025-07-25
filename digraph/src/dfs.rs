use std::fmt::Debug;

use crate::Digraph;

/// Simple trait to contain shared definitions for Digraph*DFS structs.
pub trait DigraphDFSShape {
  /// Returns a `Result` indicating whether  a path from 'source vertex' to 'i' exists.
  fn marked(&self, i: usize) -> Result<bool, String>;
}

/// Isolated "vertex marked" declaration (helps DRY up code a bit).
pub fn vertex_marked(_marked: &Vec<bool>, i: usize) -> Result<bool, String> {
  if i >= _marked.len() {
    return Err(format!("{} is out of range", i));
  }
  Ok(_marked[i])
}

/// The `DigraphDFS` struct represents a populated, ready to be queried,
/// depth-first-search result that contains a record of vertices reachable from source vertex `s`.
///
/// This implementation uses depth-first search.
///
/// The constructor takes &Theta;(`V` + `E`) time in the  worst case, where `V` is
/// the number of vertices and `E`  is the number of edges.
/// Each instance method takes &Theta;(1) time.
/// It uses &Theta;(`V`) extra space (not including the digraph).
///
#[derive(Debug)]
pub struct DigraphDFS {
  _marked: Vec<bool>,
  _count: usize,
}

impl DigraphDFS {
  /// Creates a populated, ready to be queried, depth-first-search structure.
  pub fn new(g: &Digraph, source_vertex: usize) -> Result<Self, String> {
    let mut out = DigraphDFS {
      _marked: vec![false; g.vert_count()],
      _count: 0,
    };
    g.validate_vertex(source_vertex)?;
    out.dfs(g, source_vertex)?;
    Ok(out)
  }

  /// Runs 'depth-first-search' algorithm on contained graph and stores results on `self`.
  fn dfs(&mut self, g: &Digraph, v: usize) -> Result<&Self, String> {
    g.validate_vertex(v)?;
    self._count += 1;
    self._marked[v] = true;
    let adj = g.adj(v)?;
    for w in adj {
      if !self._marked[*w] {
        self.dfs(g, *w)?;
      }
    }
    Ok(self)
  }

  /// Returns the number of vertices reachable from `source_vertex`.
  pub fn count(&self) -> usize {
    self._count
  }
}

impl DigraphDFSShape for DigraphDFS {
  /// Returns a `Result` indicating whether  a path from 'source vertex' to 'i' exists.
  fn marked(&self, i: usize) -> Result<bool, String> {
    vertex_marked(&self._marked, i)
  }
}

#[cfg(test)]
mod test {
  use crate::math::triangular_num;
  use crate::symbol_digraph::DisymGraph;

  use super::*;

    #[test]
  pub fn test_dfs_with_symbol_dag() -> Result<(), Box<dyn std::error::Error>> {
    // Get vertices
    let vowels: Vec<&str> = "a e i o u".split_ascii_whitespace().rev().collect();
    let mut sym_graph = DisymGraph::new();
    let mut sym_graph_2 = DisymGraph::new();
    let v_len = vowels.len();
    let limit = v_len - 1;

    // Chain every symbol in list to it's left adjacent vertex
    for (i, v) in vowels.iter().enumerate() {
      let edges: Vec<&str> = if i < limit {
        vec![vowels[i + 1]]
      } else {
        vec![]
      };
      let edges_2: Vec<&str> = if i < limit {
        vowels[i + 1..v_len].to_vec()
      } else {
        vec![]
      };
      sym_graph.add_edge(v, &edges);
      sym_graph_2.add_edge(v, &edges_2);
    }

    // Log graph
    // println!("{:?}", &sym_graph_2);

    // Ensure vertices, and edges, are added
    // ----
    assert_eq!(
      sym_graph.vert_count(),
      vowels.len(),
      "`#.vert_count` is invalid (1)"
    );
    assert_eq!(
      sym_graph.edge_count(),
      vowels.len() - 1,
      "`#.edge_count` is invalid (1)"
    );
    assert_eq!(
      sym_graph_2.vert_count(),
      vowels.len(),
      "`#.vert_count` is invalid (2)"
    );
    assert_eq!(
      sym_graph_2.edge_count(),
      triangular_num(vowels.len() - 1),
      "`#.edge_count` is invalid (2)"
    );

    // For each vertex in graph check that each left adjacent vertex is reachable from itself
    for i in 0..v_len {
      // println!("i: {}", i);
      let dfs_rslt = DigraphDFS::new(sym_graph.graph(), i)?;
      let dfs_rslt_2 = DigraphDFS::new(sym_graph_2.graph(), i)?;

      for j in i + 1..v_len {
        // println!("j: {}", i);
        assert_eq!(
          dfs_rslt.marked(j)?,
          true,
          "vertex `{}` not reachable from vertex `{}` (1)",
          j,
          i
        );
        assert_eq!(
          dfs_rslt_2.marked(j)?,
          true,
          "vertex `{}` not reachable from vertex `{}` (2)",
          j,
          i
        );
      }

      // Check "vertices reachable from `i`" count
      assert_eq!(
        dfs_rslt.count(),
        v_len - i,
        "`dfs_rslt.count()` should be equal to `{}` (1)",
        v_len - i
      );
      assert_eq!(
        dfs_rslt_2.count(),
        v_len - i,
        "`dfs_rslt.count()` should be equal to `{}` (2)",
        v_len - i
      );

      // Check out of bounds vert
      assert_eq!(
        dfs_rslt.marked(99).is_err(),
        true,
        "vertex `99` should not be reachable from `{}` (1)",
        i
      );
      assert_eq!(
        dfs_rslt_2.marked(99).is_err(),
        true,
        "vertex `99` should not be reachable from `{}` (2)",
        i
      );
    }

    Ok(())
  }
}

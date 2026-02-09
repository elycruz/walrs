use crate::Digraph;

/// The `DigraphDicycle` struct represents a data type for determining whether a digraph has a
/// directed cycle. The `has_cycle` method determines whether the digraph has a simple directed
/// cycle and, if so, the `cycle` method returns one.
///
/// This implementation uses depth-first search.
/// The constructor takes Θ(V + E) time in the worst case, where V is the number of vertices
/// and E is the number of edges.
/// Each instance method takes Θ(1) time.
/// It uses Θ(V) extra space (not including the digraph).
///
/// Based on the DirectedCycle implementation from Algorithms, 4th Edition by Robert Sedgewick
/// and Kevin Wayne.
pub struct DirectedCycle {
  _marked: Vec<bool>,
  _edge_to: Vec<Option<usize>>,
  _on_stack: Vec<bool>,
  _cycle: Option<Vec<usize>>,
}

impl DirectedCycle {
  /// Determines whether the given digraph has a directed cycle and, if so, finds such a cycle.
  ///
  /// ```rust
  /// use walrs_digraph::Digraph;
  /// use walrs_digraph::DirectedCycle;
  ///
  /// let mut g = Digraph::new(3);
  /// g.add_edge(0, 1).unwrap()
  ///  .add_edge(1, 2).unwrap()
  ///  .add_edge(2, 0).unwrap(); // Creates a cycle
  ///
  /// let finder = DirectedCycle::new(&g);
  /// assert_eq!(finder.has_cycle(), true);
  /// ```
  pub fn new(g: &Digraph) -> Self {
    let vert_count = g.vert_count();
    let mut out = DirectedCycle {
      _marked: vec![false; vert_count],
      _on_stack: vec![false; vert_count],
      _edge_to: vec![None; vert_count],
      _cycle: None,
    };

    // Run DFS from each vertex to find a cycle
    for v in 0..vert_count {
      if !out._marked[v] && out._cycle.is_none() {
        out.dfs(g, v);
      }
    }

    out
  }

  /// Runs depth-first search and finds a directed cycle (if one exists).
  fn dfs(&mut self, g: &Digraph, v: usize) {
    self._on_stack[v] = true;
    self._marked[v] = true;

    if let Ok(adj) = g.adj(v) {
      for &w in adj {
        // Short circuit if directed cycle found
        if self._cycle.is_some() {
          return;
        }
        // Found new vertex, so recurse
        else if !self._marked[w] {
          self._edge_to[w] = Some(v);
          self.dfs(g, w);
        }
        // Trace back directed cycle
        else if self._on_stack[w] {
          let mut cycle = Vec::new();
          let mut x = v;
          while x != w {
            cycle.push(x);
            x = self._edge_to[x].unwrap();
          }
          cycle.push(w);
          cycle.push(v);
          self._cycle = Some(cycle);
        }
      }
    }

    self._on_stack[v] = false;
  }

  /// Returns whether the digraph has a directed cycle.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use walrs_digraph::Digraph;
  /// use walrs_digraph::DirectedCycle;
  ///
  /// let mut g = Digraph::new(3);
  /// g.add_edge(0, 1).unwrap();
  /// g.add_edge(1, 2).unwrap();
  ///
  /// let finder = DirectedCycle::new(&g);
  /// assert_eq!(finder.has_cycle(), false);
  /// ```
  pub fn has_cycle(&self) -> bool {
    self._cycle.is_some()
  }

  /// Returns a directed cycle if the graph has one, else `None` otherwise.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use walrs_digraph::Digraph;
  /// use walrs_digraph::DirectedCycle;
  ///
  /// let mut g = Digraph::new(3);
  /// g.add_edge(0, 1).unwrap();
  /// g.add_edge(1, 2).unwrap();
  /// g.add_edge(2, 0).unwrap();
  ///
  /// let finder = DirectedCycle::new(&g);
  /// let cycle = finder.cycle();
  /// assert!(cycle.is_some());
  /// ```
  pub fn cycle(&self) -> Option<&Vec<usize>> {
    self._cycle.as_ref()
  }

  /// Certifies that digraph has a directed cycle if it reports one.
  /// Returns true if the cycle is valid (begins and ends with the same vertex).
  #[allow(dead_code)]
  fn check(&self) -> bool {
    if self.has_cycle() {
      if let Some(cycle) = &self._cycle {
        if cycle.is_empty() {
          return false;
        }
        let first = cycle[0];
        let last = cycle[cycle.len() - 1];
        if first != last {
          // eprintln!("cycle begins with {} and ends with {}", first, last);
          return false;
        }
      }
    }
    true
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_directed_cycle_with_cycle() {
    // Create a simple cycle: 0 -> 1 -> 2 -> 0
    let mut g = Digraph::new(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap(); // Creates cycle

    let finder = DirectedCycle::new(&g);
    assert_eq!(finder.has_cycle(), true, "Should detect cycle");
    assert!(finder.cycle().is_some(), "Should return cycle");

    let cycle = finder.cycle().unwrap();
    assert!(!cycle.is_empty(), "Cycle should not be empty");
    assert_eq!(
      cycle[0],
      cycle[cycle.len() - 1],
      "Cycle should start and end with same vertex"
    );
  }

  #[test]
  fn test_directed_cycle_no_cycle() {
    // Create a DAG: 0 -> 1 -> 2
    let mut g = Digraph::new(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();

    let finder = DirectedCycle::new(&g);
    assert_eq!(finder.has_cycle(), false, "Should not detect cycle in DAG");
    assert!(finder.cycle().is_none(), "Should return None for DAG");
  }

  #[test]
  fn test_directed_cycle_self_loop() {
    // Create a self-loop: 0 -> 0
    let mut g = Digraph::new(3);
    g.add_edge(0, 0).unwrap();

    let finder = DirectedCycle::new(&g);
    assert_eq!(finder.has_cycle(), true, "Should detect self-loop as cycle");
  }

  #[test]
  fn test_directed_cycle_complex() {
    // Create a more complex graph with cycle: 3 -> 5 -> 4 -> 3
    let mut g = Digraph::new(13);
    g.add_edge(0, 1).unwrap();
    g.add_edge(0, 5).unwrap();
    g.add_edge(2, 0).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(3, 2).unwrap();  // Cycle: 2 <-> 3
    g.add_edge(3, 5).unwrap();
    g.add_edge(4, 2).unwrap();
    g.add_edge(4, 3).unwrap();
    g.add_edge(5, 4).unwrap();
    g.add_edge(6, 0).unwrap();
    g.add_edge(6, 4).unwrap();
    g.add_edge(6, 9).unwrap();
    g.add_edge(7, 6).unwrap();
    g.add_edge(7, 8).unwrap();
    g.add_edge(8, 7).unwrap();  // Cycle: 7 <-> 8
    g.add_edge(8, 9).unwrap();
    g.add_edge(9, 10).unwrap();
    g.add_edge(9, 11).unwrap();
    g.add_edge(10, 12).unwrap();
    g.add_edge(11, 4).unwrap();
    g.add_edge(11, 12).unwrap();
    g.add_edge(12, 9).unwrap();

    let finder = DirectedCycle::new(&g);
    assert_eq!(finder.has_cycle(), true, "Should detect cycle in complex graph");
  }

  #[test]
  fn test_directed_cycle_empty_graph() {
    let g = Digraph::new(0);
    let finder = DirectedCycle::new(&g);
    assert_eq!(finder.has_cycle(), false, "Empty graph should have no cycle");
  }

  #[test]
  fn test_directed_cycle_single_vertex() {
    let g = Digraph::new(1);
    let finder = DirectedCycle::new(&g);
    assert_eq!(finder.has_cycle(), false, "Single vertex with no edges should have no cycle");
  }

  #[test]
  fn test_check_validation() {
    let mut g = Digraph::new(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();

    let finder = DirectedCycle::new(&g);
    assert_eq!(finder.check(), true, "Valid cycle should pass check");
  }

  #[test]
  fn test_check_with_empty_cycle() {
    // Test the edge case where cycle is empty (line 137)
    let mut g = Digraph::new(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();

    let mut finder = DirectedCycle::new(&g);
    // Manually create an invalid state with empty cycle for testing
    finder._cycle = Some(Vec::new());

    assert_eq!(finder.check(), false, "Empty cycle should fail check");
  }

  #[test]
  fn test_check_with_mismatched_endpoints() {
    // Test the edge case where first != last (line 143)
    let mut g = Digraph::new(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap();

    let mut finder = DirectedCycle::new(&g);
    // Manually create an invalid cycle with mismatched endpoints for testing
    finder._cycle = Some(vec![0, 1, 2]); // Should start and end with same vertex

    assert_eq!(finder.check(), false, "Cycle with mismatched endpoints should fail check");
  }

  #[test]
  fn test_check_with_no_cycle() {
    // Test when there's no cycle at all
    let mut g = Digraph::new(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();

    let finder = DirectedCycle::new(&g);
    assert_eq!(finder.check(), true, "No cycle should pass check");
  }
}

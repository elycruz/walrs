use crate::graph::shared_utils::extract_vert_and_edge_counts_from_bufreader;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

/// A basic index graph that tracks edges on vertex indices in adjacency lists.
///
/// This is an undirected graph implementation using adjacency list representation.
/// Vertices are identified by `usize` indices (0-based). Edges are stored bidirectionally.
///
/// # Examples
///
/// ```
/// use walrs_graph::Graph;
///
/// // Create a graph with 5 vertices
/// let mut g = Graph::new(5);
///
/// // Add edges (automatically adds both directions)
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(1, 2).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// assert_eq!(g.vert_count(), 5);
/// assert_eq!(g.edge_count(), 6); // 3 logical edges × 2 directions
///
/// // Query adjacency
/// let adj = g.adj(1).unwrap();
/// assert!(adj.contains(&0));
/// assert!(adj.contains(&2));
/// ```
#[derive(Debug)]
pub struct Graph {
  // @todo - Should be `Vec<Option<Vec<usize>>>`, more memory efficient.
  _adj_lists: Vec<Vec<usize>>,
  _edge_count: usize,
  // @todo - Error message for invalid vertex should be customizable.
}

impl Graph {
  /// Returns a new graph containing given `vert_count` number of vertex slots.
  ///
  /// ```rust
  /// use walrs_graph::Graph;
  ///
  /// for vert_count in 0..3 {
  ///   let g = Graph::new(vert_count);
  ///
  ///   assert_eq!(g.vert_count(), vert_count);
  ///   assert_eq!(g.edge_count(), 0);
  /// }
  /// ```
  pub fn new(vert_count: usize) -> Self {
    Graph {
      _adj_lists: vec![Vec::new(); vert_count],
      _edge_count: 0,
    }
  }

  /// Returns vertex count.
  pub fn vert_count(&self) -> usize {
    self._adj_lists.len()
  }

  /// Returns number of edges in graph.
  pub fn edge_count(&self) -> usize {
    self._edge_count
  }

  /// Returns a result containing given vertex' adjacency list, or a string
  /// containing the "out-of-bounds index" error.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_graph::Graph;
  ///
  /// let mut g = Graph::new(3);
  /// g.add_edge(0, 1).unwrap();
  /// g.add_edge(0, 2).unwrap();
  ///
  /// let adj = g.adj(0).unwrap();
  /// assert_eq!(adj.len(), 2);
  /// assert!(adj.contains(&1));
  /// assert!(adj.contains(&2));
  ///
  /// // Invalid vertex returns error
  /// assert!(g.adj(99).is_err());
  /// ```
  pub fn adj(&self, i: usize) -> Result<&[usize], String> {
    self
      .validate_vertex(i)
      .map(|_| self._adj_lists[i].as_slice())
  }

  /// Returns a `Result` containing the number of edges touching a given vertex,
  ///   or a `String` representing the 'out-of-bounds index' (error) message.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_graph::Graph;
  ///
  /// let mut g = Graph::new(4);
  /// g.add_edge(0, 1).unwrap();
  /// g.add_edge(0, 2).unwrap();
  /// g.add_edge(0, 3).unwrap();
  ///
  /// assert_eq!(g.degree(0).unwrap(), 3);
  /// assert_eq!(g.degree(1).unwrap(), 1);
  /// assert_eq!(g.degree(2).unwrap(), 1);
  /// assert_eq!(g.degree(3).unwrap(), 1);
  /// ```
  pub fn degree(&self, v: usize) -> Result<usize, String> {
    self.adj(v).map(|adj| adj.len())
  }

  /// Adds vertex to graph and returns given vertex.
  pub fn add_vertex(&mut self, v: usize) -> usize {
    let mut v_len = self._adj_lists.len();
    if v >= v_len {
      loop {
        if v_len > v {
          break;
        }
        self._adj_lists.push(Vec::new());
        v_len += 1;
      }
    }
    v
  }

  /// Returns a boolean indicating whether or not graph contains given vertex, `v`, or not.
  pub fn has_vertex(&self, v: usize) -> bool {
    let len = self._adj_lists.len();
    len > 0 && len <= v + 1
  }

  /// Removes a vertex from the graph.
  pub fn remove_vertex(&mut self, v: usize) -> Result<&mut Self, String> {
    if let Err(err) = self.validate_vertex(v) {
      return Err(err);
    }

    let target_v_adj = self.adj(v).unwrap();

    // Remove related edges and offset vertices greater than target, in adj lists
    target_v_adj.to_owned().into_iter().for_each(|w| {
      // Remove edge
      let found_v_idx = match self.remove_edge(v, w) {
        Ok(val) => val,
        Err(err) => panic!("{}", err),
      };

      let w_adj = &mut self._adj_lists[w];

      // Offset vertices greater than target
      for i in found_v_idx..w_adj.len() {
        let target = w_adj[i];
        if target >= v && target > 0 {
          w_adj[i] = target - 1;
        }
      }
    });

    self._adj_lists.remove(v);
    Ok(self)
  }

  /// Adds an edge to the graph and returns a `Result` containing self, else a string representing
  /// an 'index is out of bounds' error.
  ///
  /// This method adds an undirected edge by adding both `v` to `w`'s adjacency list
  /// and `w` to `v`'s adjacency list. Adjacency lists are kept sorted.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_graph::Graph;
  ///
  /// let mut g = Graph::new(3);
  ///
  /// // Method chaining supported
  /// g.add_edge(0, 1).unwrap()
  ///  .add_edge(1, 2).unwrap()
  ///  .add_edge(0, 2).unwrap();
  ///
  /// assert_eq!(g.edge_count(), 6); // 3 logical edges × 2 directions
  /// assert!(g.has_edge(0, 1));
  /// assert!(g.has_edge(1, 0)); // Undirected: both directions exist
  /// ```
  pub fn add_edge(&mut self, v: usize, w: usize) -> Result<&mut Self, String> {
    self
      .validate_vertex(v)
      .and(self.validate_vertex(w))
      .map(|_| {
        let adj_list_1 = &mut self._adj_lists[v];
        adj_list_1.push(w);
        adj_list_1.sort_unstable();
        let adj_list_2 = &mut self._adj_lists[w];
        adj_list_2.push(v);
        adj_list_2.sort_unstable();
        self._edge_count += 2;
        self
      })
  }

  /// Returns a `bool` indicating whether graph contains edge `v -> w` or not.
  ///
  /// Uses binary search for O(log E) lookup time.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_graph::Graph;
  ///
  /// let mut g = Graph::new(3);
  /// g.add_edge(0, 1).unwrap();
  ///
  /// assert!(g.has_edge(0, 1));
  /// assert!(g.has_edge(1, 0)); // Undirected
  /// assert!(!g.has_edge(0, 2));
  /// assert!(!g.has_edge(99, 0)); // Invalid vertex
  /// ```
  pub fn has_edge(&self, v: usize, w: usize) -> bool {
    let len = self.vert_count();
    if len == 0 || v >= len || w >= len {
      return false;
    }
    self._adj_lists[v].binary_search(&w).map_or(false, |_| true)
  }

  /// Removes edge `v` to `w` from graph.
  pub fn remove_edge(&mut self, v: usize, w: usize) -> Result<usize, String> {
    if let Err(err) = self
      .validate_vertex(v)
      .and_then(|_| self.validate_vertex(w))
    {
      return Err(err);
    }
    let adj = &mut self._adj_lists;

    let adj_v = &mut adj[v];
    adj_v.remove(adj_v.binary_search(&w).unwrap());

    let adj_w = &mut adj[w];
    let found_v_idx = adj_w.binary_search(&v).unwrap();
    adj_w.remove(found_v_idx);

    self._edge_count -= 2;
    Ok(found_v_idx)
  }

  /// Checks if vertex exists in graph.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_graph::Graph;
  ///
  /// let g = Graph::new(5);
  ///
  /// assert!(g.validate_vertex(0).is_ok());
  /// assert!(g.validate_vertex(4).is_ok());
  /// assert!(g.validate_vertex(5).is_err());
  /// assert!(g.validate_vertex(99).is_err());
  /// ```
  pub fn validate_vertex(&self, v: usize) -> Result<usize, String> {
    let len = self._adj_lists.len();
    if v >= len {
      return Err(format!(
        "{}",
        invalid_vertex_msg(v, if len > 0 { len - 1 } else { 0 })
      ));
    }
    Ok(v)
  }

  /// Process graph representation `Lines` into graph.
  pub fn digest_lines<R: std::io::Read>(
    &mut self,
    lines: Lines<&mut BufReader<R>>,
  ) -> Result<&mut Self, Box<dyn std::error::Error>> {
    // Loop through lines
    for line in lines {
      // For each edge definition, enter them into graph
      match line {
        // If line
        Ok(_line) => {
          // Split and parse edge values to integers
          let verts: Vec<usize> = _line
            .split_ascii_whitespace()
            .map(|x| x.parse::<usize>().unwrap())
            .collect();

          if let Err(err) = self.add_edge(verts[0], verts[1]) {
            return Err(Box::from(err));
          }
        }
        Err(err) => {
          return Err(Box::new(err));
        }
      }
    }

    Ok(self)
  }
}

/// Returns panic message for invalid vertices;  Exported for use in testing.
pub fn invalid_vertex_msg(v: usize, max_v: usize) -> String {
  format!("Vertex {} is outside defined range 0-{}", v, max_v)
}

impl<R: std::io::Read> TryFrom<&mut BufReader<R>> for Graph {
  type Error = Box<dyn std::error::Error>;

  ///  Creates a Graph from a buffer reader representing a text file formatted as:
  ///
  ///  ```text
  ///  {num_verts}
  ///  {num_edges}
  ///  {vertex} {vertex}
  ///  {vertex} {vertex}
  ///  ...
  ///  ```
  fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
    // Extract vert count, and move cursor passed edge count line, for reader
    let vert_count = extract_vert_and_edge_counts_from_bufreader(reader)?.0;

    // Construct graph
    let mut g = Graph::new(vert_count);

    // Populate graph from buffer lines
    g.digest_lines(reader.lines())?;

    // Return graph
    Ok(g)
  }
}

impl<R: std::io::Read> TryFrom<BufReader<R>> for Graph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(mut reader: BufReader<R>) -> Result<Self, Self::Error> {
    Graph::try_from(&mut reader)
  }
}

impl TryFrom<&File> for Graph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(file_struct: &File) -> Result<Self, Self::Error> {
    Graph::try_from(&mut BufReader::new(file_struct))
  }
}

impl TryFrom<File> for Graph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(file_struct: File) -> Result<Self, Self::Error> {
    Graph::try_from(&mut BufReader::new(file_struct))
  }
}

#[cfg(test)]
mod test {
  use crate::graph::{invalid_vertex_msg, Graph};
  use std::fs::File;
  use std::io::BufReader;

  #[test]
  pub fn test_new() {
    for vert_count in 0..2 {
      let g = Graph::new(vert_count);
      assert_eq!(g.edge_count(), 0);
      assert_eq!(g.vert_count(), vert_count);
      assert_eq!(g._adj_lists.len(), vert_count);
    }
  }

  #[test]
  pub fn test_adj() {
    // Test valid
    for vert_count in 0..4 {
      let g = Graph::new(vert_count);

      for i in 0..vert_count {
        match g.adj(i) {
          Ok(adj) => assert_eq!(adj.len(), 0),
          Err(err) => panic!("{}", err),
        }
      }
    }
  }

  #[test]
  pub fn test_degree() {
    let verts_limit = 6;
    let verts_start = 0;
    for graph_size in verts_start..verts_limit {
      let mut g = Graph::new(graph_size);

      // If graph is expected to have edges (for our example)
      if graph_size > 1 {
        // Add edges from index `i` to every other index, in graph.
        for i in verts_start..graph_size {
          // `start..end` range is non-inclusive (on right hand side)
          for j in (i + 1)..graph_size {
            // ""
            if let Err(err) = g.add_edge(i, j) {
              panic!("{}", err);
            }
          }
        }
      }

      println!("\nFor {:?}", &g);

      // Test `degree` results
      // ----
      if graph_size <= 1 {
        let rslt = Err(invalid_vertex_msg(graph_size, 0));
        println!("Checking `g.degree({}) == {:?}`", graph_size, &rslt);
        assert_eq!(g.degree(graph_size), rslt);
      }
      // If graph can have edges, in our example, test for all vertices
      // having the same `degree`, since our graph is bi-directional, and
      // We added edges from every vertex, to every other vertex
      else {
        for target_idx in verts_start..graph_size {
          let rslt = Ok(graph_size - 1);
          println!("Checking `g.degree({}) == {:?}`", target_idx, &rslt);
          assert_eq!(
            g.degree(target_idx),
            rslt,
            "g.degree({}) == {:?}",
            target_idx,
            rslt
          );
        }
      }
    }
  }

  #[test]
  pub fn test_vert_count() {
    for vert_count in 0..3 {
      let g = Graph::new(vert_count);
      assert_eq!(g.vert_count(), vert_count);
      assert_eq!(g._adj_lists.len(), vert_count);
    }
  }

  #[test]
  pub fn test_edge_count() {
    for graph_size in 0..9 {
      let mut g = Graph::new(graph_size);
      let mut edge_count_sum = 0;

      // Add edges for each vertex to every other vertex in graph
      for i in 0..graph_size {
        for j in (i + 1)..graph_size {
          if let Err(err) = g.add_edge(i, j) {
            panic!("{}", err);
          }

          // For every edge `i` to `j` there should be `2` edges added to the graph
          edge_count_sum += 2;
        }
      }

      // Assert edge count
      assert_eq!(g.edge_count(), edge_count_sum, "#.edge_count is invalid");
    }
  }

  #[test]
  pub fn test_add_vertex() {
    // Test `add_vertex` for different expected graph sizes
    for vert_count in 0..5 {
      let mut g = Graph::new(vert_count);

      // Add vertices 0 to num verts to add (non-inclusive on the right hand side)
      for i in 0..vert_count {
        assert_eq!(g.add_vertex(i), i);
      }

      assert_eq!(g.edge_count(), 0);
      assert_eq!(g.vert_count(), vert_count);
    }
  }

  #[test]
  pub fn test_add_edge() {
    // Test for vertices population (on instantiation, or after the fact) and
    // vertex degree.
    // In the following examples expected edges for
    // undirected graphs is `(num_vertices - 1) * num_vertices)`.
    // ----
    // test cases array is  mutable here to allow `#.add_edge()` to be called
    for (graph_size, num_expected_edges) in [(0, 0), (1, 0), (3, 6)] {
      let mut g = Graph::new(graph_size);

      // Add vertices 0 to "num vertices to add" (non-inclusive
      // on the right hand side).
      if graph_size > 0 {
        for i in 0..graph_size {
          assert_eq!(g.add_vertex(i), i);
        }
      }

      // Add edges for each vertex.
      if graph_size > 1 {
        for i in 0..graph_size {
          for j in (i + 1)..graph_size {
            if let Err(err) = g.add_edge(i, j) {
              panic!("{}", err);
            }

            let adj_i = g.adj(i).unwrap();
            let adj_j = g.adj(j).unwrap();

            // Ensure adjacency lists for both vertices contain entries back to each other
            assert!(
              adj_i.contains(&j),
              "`graph.adj({})` should contain `{}`",
              i,
              j
            );
            assert!(
              adj_j.contains(&i),
              "`graph.adj({})` should contain `{}`",
              j,
              i
            );

            // Ensure last inserted edge vertex is the last inserted
            // in 'forward' adjacency lists (list for `i`)
            // - Helps ensure adjacency lists are sorted
            assert_eq!(
              *adj_i.last().unwrap(),
              j,
              "`{}` should be the last inserted vertex in `{:?}`",
              j,
              &adj_i
            );
          }
        }
      }

      // Print intro message for test.
      println!(
        "For {:?} with {} vertices;  Expecting {} edges",
        &g, graph_size, num_expected_edges
      );

      // Test `degree` for each vertex
      for i in 0..graph_size {
        let expected_degree = if graph_size > 0 { graph_size - 1 } else { 0 };
        // println!("`#.degree({})` should return {}", i, expected_degree);
        assert_eq!(
          g.degree(i).unwrap(),
          expected_degree,
          "{}",
          format!("`#.degree({})` should return {}", i, expected_degree)
        );
      }

      assert_eq!(g.edge_count(), num_expected_edges, "Edge count is invalid");
      assert_eq!(g.vert_count(), graph_size, "Vert count is invalid");
    }
  }

  #[test]
  pub fn test_remove_vertex() {
    // Test `add_vertex` for different expected graph sizes
    for vert_count in 0..9 {
      println!("Setup for vert_count {}", vert_count);

      let mut g = Graph::new(vert_count); // vertices added here

      // Add edges to graph (non-inclusive on the right hand side)
      for v in 0..vert_count {
        for w in (v + 1)..vert_count {
          if let Err(err) = g.add_edge(v, w) {
            panic!("{}", err);
          }
        }
      }

      // Compute expected edge count
      let expected_edge_count = if vert_count > 1 {
        (vert_count - 1) * vert_count
      } else {
        0
      };

      // Ensure our preliminaries are set
      assert_eq!(
        g.edge_count(),
        expected_edge_count,
        "g.edge_count() == {}",
        expected_edge_count
      );
      assert_eq!(
        g.vert_count(),
        vert_count,
        "g.vert_count() == {}",
        vert_count
      );

      println!("  Beginning vertex removal iterations for {:?}", &g);

      // Remove, and test removal of added vertices
      for i in 0..vert_count {
        let dyn_vert_count = g.vert_count();
        let target_v = if dyn_vert_count > i {
          i
        } else if dyn_vert_count > 0 {
          dyn_vert_count - 1
        } else {
          0
        };
        println!(
          "    Removing vertex {};  Remaining {};  For vert_count: {};",
          target_v,
          g.vert_count(),
          vert_count
        );

        // Capture target_v's adjacency list
        let target_v_adj = g.adj(target_v).unwrap();

        // Capture all adjacency lists related to target_v and ensure they each contain target_v
        let related_adjs: Vec<(usize, Vec<usize>)> = target_v_adj
          .iter()
          .map(|x| {
            let related_adj = g.adj(*x).unwrap();
            assert!(
              related_adj.contains(&target_v),
              "Backward edge adj lists should contain vertex to remove"
            );
            (*x, related_adj.into())
          })
          .collect();

        // Remove vertex
        if let Err(err) = g.remove_vertex(target_v) {
          panic!("{}", err);
        }

        println!("   Ensure references in graph were updated accordingly");
        related_adjs.into_iter().for_each(|(w, adj_w)| {
          let tw = if w >= target_v { w - 1 } else { w };
          let nw = g.adj(tw).unwrap();
          println!("      w: {}; adj_w: {:?}; nw: {:?}", w, &adj_w, &nw);

          // Ensure one relationship was removed
          assert_eq!(
            nw.len(),
            adj_w.len() - 1,
            "adjacency list's (at idx {}) has invalid length",
            w
          );

          // Ensure relationship vertices, upwards of target vertex, were decreased by one
          adj_w.iter().enumerate().for_each(|(i2, w2)| {
            if *w2 != target_v {
              if i2 == target_v {
                return;
              }
              let target_i2 = if i2 < target_v { i2 } else { i2 - 1 };
              assert_eq!(
                nw[target_i2],
                if *w2 < target_v { *w2 } else { *w2 - 1 },
                "vertex {}, wasn't offset in {}'s adjacency list",
                nw[target_i2],
                tw
              );
            }
          });
        });
      }
    }
  }

  #[test]
  pub fn test_remove_edge_and_has_edge() {
    for vert_count in 0..9 {
      println!("Setup for vert_count {}", vert_count);

      let mut g = Graph::new(vert_count); // vertices added here

      // Add edges to graph (non-inclusive on the right hand side)
      for v in 0..vert_count {
        for w in (v + 1)..vert_count {
          if let Err(err) = g.add_edge(v, w) {
            panic!("{}", err);
          }
        }
      }

      // Compute expected edge count
      let expected_edge_count = if vert_count > 1 {
        (vert_count - 1) * vert_count
      } else {
        0
      };

      // Ensure our preliminaries are set
      assert_eq!(
        g.edge_count(),
        expected_edge_count,
        "g.edge_count() == {}",
        expected_edge_count
      );
      assert_eq!(
        g.vert_count(),
        vert_count,
        "g.vert_count() == {}",
        vert_count
      );

      println!("  Beginning edge removal iterations for {:?}", &g);

      // Test removal of edges
      for i in 1..vert_count {
        let prev_edge_count = g.edge_count();
        let v = i - 1;
        let w = i;

        println!(
          "    Removing edge:  {} -> {};  For vert_count:  {};",
          v, w, vert_count
        );

        // Ensure edge exists
        assert!(
          g.adj(v).unwrap().contains(&w),
          "vertex {} should be in vertex {}'s list",
          w,
          v
        );
        assert!(
          g.adj(w).unwrap().contains(&v),
          "vertex {} should be in vertex {}'s list",
          v,
          w
        );
        assert!(
          g.has_edge(v, w),
          "graph should contain edge `{} -> {}`",
          v,
          w
        );
        assert!(
          g.has_edge(w, v),
          "graph should contain edge `{} -> {}`",
          w,
          v
        );

        // Remove edge
        if let Err(err) = g.remove_edge(v, w) {
          panic!("{}", err);
        }

        // Ensure edges were removed
        assert_ne!(
          g.adj(v).unwrap().contains(&w),
          true,
          "vertex {} should not exist in vertex {}'s list",
          w,
          v
        );
        assert_ne!(
          g.adj(w).unwrap().contains(&v),
          true,
          "vertex {} should not exist in vertex {}'s list",
          v,
          w
        );
        assert_ne!(
          g.has_edge(v, w),
          true,
          "graph should not contain edge `{} -> {}`",
          v,
          w
        );
        assert_ne!(
          g.has_edge(w, v),
          true,
          "graph should not contain edge `{} -> {}`",
          w,
          v
        );

        // Ensure edge count was offset correctly
        assert_eq!(
          g.edge_count(),
          prev_edge_count - 2,
          "Expected edge count {} to be decreased by 2",
          prev_edge_count
        );
      }
    }
  }

  #[test]
  pub fn test_validate_vertex() {
    for (graph_size, vert_to_validate, result) in [
      (0, 99, Err(invalid_vertex_msg(99, 0))),
      (3, 99, Err(invalid_vertex_msg(99, 2))),
      (3, 2, Ok(2)),
    ] {
      let g = Graph::new(graph_size);
      assert_eq!(g.validate_vertex(vert_to_validate), result);
    }
  }

  #[test]
  pub fn test_try_from_file_ref() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get graph data
    let f = File::open(&file_path)?;

    // Create graph
    let _: Graph = (&f).try_into().unwrap();

    Ok(())
  }

  #[test]
  pub fn test_try_from_file() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get graph data
    let f = File::open(&file_path)?;

    // Create graph
    let _: Graph = f.try_into().unwrap();

    Ok(())
  }

  #[test]
  pub fn test_try_from_mut_buf_reader_ref() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Seek;
    use crate::graph::shared_utils::extract_vert_and_edge_counts_from_bufreader;

    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get graph data
    let f = File::open(&file_path)?;
    let mut reader = BufReader::new(f);

    // Create graph (impls for `TryFrom<BufReader<R: std::io::Read>>` and `TryFrom<File>` are defined for `Graph` struct
    let g: Graph = (&mut reader).try_into()?;

    // Rewind reader and extract vert and edge count from first lines
    reader.rewind()?;

    let (expected_vert_count, expected_edge_count) =
        extract_vert_and_edge_counts_from_bufreader(&mut reader)?;

    assert_eq!(
      g.vert_count(),
      expected_vert_count,
      "Vert count is invalid"
    );
    // Note: Graph counts edges bidirectionally (2x the logical edge count)
    assert_eq!(
      g.edge_count(),
      expected_edge_count * 2,
      "Edge count is invalid"
    );

    Ok(())
  }

  #[test]
  pub fn test_try_from_buf_reader() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get graph data
    let f = File::open(&file_path)?;

    // Create graph
    let _: Graph = BufReader::new(f).try_into().unwrap();

    Ok(())
  }
}

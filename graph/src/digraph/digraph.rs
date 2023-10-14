use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

use crate::graph::invalid_vertex_msg;
use crate::graph::shared_utils::extract_vert_and_edge_counts_from_bufreader;

#[derive(Clone, Debug)]
pub struct Digraph {
  _adj_lists: Vec<Vec<usize>>,
  _edge_count: usize,
  _in_degree: Vec<usize>,
}

impl Digraph {
  /// Returns a new graph initialized with `vert_count` number of empty adjacency lists (one for each expected vertex).
  pub fn new(vert_count: usize) -> Self {
    Digraph {
      _adj_lists: vec![Vec::new(); vert_count],
      _in_degree: vec![0; vert_count],
      _edge_count: 0,
    }
  }

  /// Returns vertex count
  pub fn vert_count(&self) -> usize {
    self._adj_lists.len()
  }

  /// Returns number of edges in graph.
  pub fn edge_count(&self) -> usize {
    self._edge_count
  }

  /// Returns a `Result` containing given vertex' adjacency list, or an
  ///  'vertex out out bounds' error message..
  /// ```rust
  /// use walrs_graph::digraph::Digraph;
  ///
  /// let mut digraph = Digraph::new(5);
  /// let vowels = "aeiou";
  ///
  /// // Add indices/vertices to graph
  /// for (v, _) in vowels.char_indices() {
  ///   digraph.add_vertex(v);
  /// }
  ///
  /// assert_eq!(digraph.vert_count(), vowels.len());
  ///
  /// // Should not panic!
  /// let adj = match digraph.adj(0) {
  ///   Ok(adj) => adj,
  ///   Err(err) => panic!("{}", err)
  /// };
  ///
  /// // ..
  pub fn adj(&self, v: usize) -> Result<&Vec<usize>, String> {
    if let Err(err) = self.validate_vertex(v) {
      return Err(err);
    }
    Ok(&self._adj_lists[v])
  }

  pub fn outdegree(&self, v: usize) -> Result<usize, String> {
    if let Err(err) = self.validate_vertex(v) {
      return Err(err);
    }
    Ok(self._adj_lists[v].len())
  }

  pub fn indegree(&self, v: usize) -> Result<usize, String> {
    if let Err(err) = self.validate_vertex(v) {
      return Err(err);
    }
    Ok(self._in_degree[v])
  }

  /// Adds vertex to digraph;  **Note:** This method increases the graph's internal adjacency list
  /// representation's length if vertex is greater than contained adjacency's list until index is
  /// valid, allows graph to grow arbitrarily.
  pub fn add_vertex(&mut self, v: usize) -> usize {
    let mut v_len = self._adj_lists.len();
    if v >= v_len {
      loop {
        if v_len > v {
          break;
        }
        self._adj_lists.push(Vec::new());
        self._in_degree.push(0);
        v_len += 1;
      }
    }
    v
  }

  /// Adds an edges from vertex `v` to vertex `w`, to the graph.
  pub fn add_edge(&mut self, v: usize, w: usize) -> Result<&mut Self, String> {
    if let Err(err) = self
      .validate_vertex(v)
      .and_then(|_| self.validate_vertex(w))
    {
      return Err(err);
    }
    let adj = &mut self._adj_lists[v];
    adj.push(w);
    adj.sort_unstable();
    self._edge_count += 1;
    self._in_degree[w] += 1;
    Ok(self)
  }

  /// Checks if contained adjacency list can accommodate given vertex.
  pub fn validate_vertex(&self, v: usize) -> Result<&Self, String> {
    let len = self._adj_lists.len();
    if v >= len {
      return Err(format!(
        "{:}",
        invalid_vertex_msg(v, if len > 0 { len - 1 } else { 0 })
      ));
    }
    Ok(self)
  }

  // Returns a copy of given Digraph "reversed".
  pub fn reverse(&self) -> Result<Self, String> {
    let v_count = self.vert_count();
    let mut out = Digraph::new(v_count);
    if v_count == 0 {
      return Ok(out);
    }
    for v in 0..v_count {
      for w in &self._adj_lists[v] {
        if let Err(err) = out.add_edge(*w, v) {
          return Err(err);
        }
      }
    }
    Ok(out)
  }

  /// Populates digraph instance from incoming lines.  @note Returns Error string if not all
  /// vertices are in graph (use `#.add_vertex(...)` to enter the max vertex (expected vertex count)
  /// into the graph before digesting buffer lines.
  pub fn digest_lines<R: std::io::Read>(
    &mut self,
    lines: Lines<&mut BufReader<R>>,
  ) -> Result<&Self, Box<dyn std::error::Error>> {
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

          // Add edge, and panic if unable to add it
          if let Err(err) = self.add_edge(verts[0], verts[1]) {
            return Err(err.into());
          }
        }

        // Catch "error reading line"
        Err(err) => {
          return Err(Box::new(err));
        }
      }
    }

    Ok(self)
  }
}

impl<R: std::io::Read> From<&mut BufReader<R>> for Digraph {
  ///  Creates a Digraph from a buffer reader representing a text file formatted as;  E.g.,
  ///  ```text
  ///  num_verts
  ///  num_edges
  ///  vertex weight
  ///  vertex weight weight
  ///  ...
  ///  ```
  /// `weight` here is a vertex/weight reachable from vertex `vertex`.
  ///
  fn from(reader: &mut BufReader<R>) -> Self {
    // Extract vert count, and move cursor passed edge count line, for reader
    let vert_count = match extract_vert_and_edge_counts_from_bufreader(reader) {
      Ok((vc, _)) => vc,
      Err(err) => panic!("{:?}", err),
    };

    // Construct digraph
    let mut dg = Digraph::new(vert_count);

    // Populate graph from buffer lines
    if let Err(err) = dg.digest_lines(reader.lines()) {
      panic!("{:?}", err);
    }

    // Return graph
    dg
  }
}

impl<R: std::io::Read> From<BufReader<R>> for Digraph {
  fn from(mut reader: BufReader<R>) -> Self {
    Digraph::from(&mut reader)
  }
}

impl From<&File> for Digraph {
  fn from(file_struct: &File) -> Self {
    (&mut BufReader::new(file_struct)).into()
  }
}

impl From<File> for Digraph {
  fn from(file_struct: File) -> Self {
    (&mut BufReader::new(file_struct)).into()
  }
}

#[cfg(test)]
mod test {
  use crate::digraph::Digraph;

  /// Some usize vec;  Note not meant to be generic;  Defined only this `test` module.
  pub fn usize_vec_sum(us: &Vec<usize>) -> usize {
    let mut us_sum = 0;
    for u in us {
      us_sum += u;
    }
    us_sum
  }

  #[test]
  pub fn test_new() {
    for count in 0..3 {
      let g = Digraph::new(count);
      assert_eq!(g.edge_count(), 0);
      assert_eq!(g._edge_count, g.edge_count());
      assert_eq!(g.vert_count(), count);
      assert_eq!(g.vert_count(), g._adj_lists.len());
      assert_eq!(g._in_degree.len(), count);
    }
  }

  #[test]
  pub fn test_adj() {
    // Test valid
    for num_verts in 1..4 {
      let g = Digraph::new(num_verts);

      // Test for valid adjacency sets
      for i in 0..num_verts {
        match g.adj(i) {
          Ok(rslt) => assert_eq!(rslt.len(), 0),
          Err(err) => panic!("{}", err),
        }
      }
    }
  }

  #[test]
  pub fn test_vert_count() {
    for num_verts in 0..4 {
      let g = Digraph::new(num_verts);
      assert_eq!(g.vert_count(), num_verts);
      assert_eq!(g._adj_lists.len(), num_verts);
      assert_eq!(g._in_degree.len(), num_verts);
    }
  }

  #[test]
  pub fn test_edge_count() {
    for (graph_size, _) in [(0, 0), (1, 0), (2, 1), (3, 3), (9, 9)] {
      let mut g = Digraph::new(graph_size);

      // Add edges for each to vertex each subsequent vertex.
      if graph_size > 1 {
        // For each vertex from `0` to `limit` add edges subsequent vertices.
        for i in 0..graph_size {
          for j in (i + 1)..graph_size {
            if let Err(err) = g.add_edge(i, j) {
              panic!("{}", err);
            }
          }
        }
      }

      // Assert edge count
      assert_eq!(
        g.edge_count(),
        usize_vec_sum(&g._in_degree),
        "#.edge_count is invalid"
      );
    }
  }

  #[test]
  pub fn test_indegree_and_outdegree() -> Result<(), String> {
    for (graph_size, _) in [(0, 3), (0, 0), (1, 0), (2, 1), (3, 3), (9, 9)] {
      let mut g = Digraph::new(graph_size);

      // Add edges for each vertex.
      if graph_size > 1 {
        // For each vertex from `0` to `limit` add edges subsequent vertices.
        for i in 0..graph_size {
          for j in (i + 1)..graph_size {
            g.add_edge(i, j)?;
          }
        }
      }

      // Print intro message for test.
      // println!(
      //   "For {:?} with {:} vertices;  Expecting {:} edges",
      //   &g, graph_size, num_expected_edges
      // );

      // Test `degree` for each vertex
      for i in 0..graph_size {
        let expected_indegree = if i >= 1 { i } else { 0 };
        let expected_outdegree = if graph_size > 1 {
          graph_size - i - 1
        } else {
          0
        };
        assert_eq!(
          g.indegree(i)?,
          expected_indegree,
          "{}",
          format!("`#.indegree({:})` should return {:}", i, expected_indegree)
        );
        assert_eq!(
          g.outdegree(i)?,
          expected_outdegree,
          "{}",
          format!(
            "`#.outdegree({:})` should return {:}",
            i, expected_outdegree
          )
        );
      }
    }

    Ok(())
  }

  #[test]
  #[should_panic(expected = "Vertex 99 is out of index range 0-0")]
  pub fn test_adj_invalid() {
    let g = Digraph::new(0);
    if let Err(err) = g.adj(99) {
      panic!("{}", err);
    }
  }

  #[test]
  pub fn test_add_vertex() {
    // array needs to be mutable to allow `#.add_vertex` method to be called
    for (g, num_verts_to_add) in [
      (Digraph::new(0), 3),
      (Digraph::new(0), 2),
      (Digraph::new(0), 1),
      (Digraph::new(2), 3),
      (Digraph::new(1), 2),
      (Digraph::new(0), 1),
    ]
      .as_mut()
    {
      // Add vertices 0 to num verts to add (non-inclusive on the right hand side)
      for i in 0..*num_verts_to_add {
        g.add_vertex(i);
      }

      assert_eq!(g.edge_count(), 0);
      assert_eq!(g.vert_count(), *num_verts_to_add);
    }
  }

  #[test]
  pub fn test_add_edge() -> Result<(), String> {
    for (graph_size, _) in [(0, 3), (0, 0), (1, 0), (2, 2), (3, 3), (9, 9)] {
      let mut g = Digraph::new(graph_size);

      // Add edges for each vertex.
      if graph_size > 1 {
        // For each vertex from `0` to `limit` add edges subsequent vertices.
        for i in 0..graph_size {
          for j in (i + 1)..graph_size {
            g.add_edge(i, j)?;
          }
        }
      }

      // Print intro message for test.
      // println!(
      //   "For {:?} with {:} vertices;  Expecting {:} edges",
      //   &g, graph_size, num_expected_edges
      // );

      // Test `degree` for each vertex
      for i in 0..graph_size {
        let expected_indegree = if i >= 1 { i } else { 0 };
        // println!("`#.indegree({:})` should return {:}", i, expected_indegree);
        assert_eq!(
          g.indegree(i)?,
          expected_indegree,
          "{}",
          format!("`#.indegree({:})` should return {:}", i, expected_indegree)
        );
      }

      // Ensure messages are separate for each test.
      // println!();

      assert_eq!(
        g.edge_count(),
        usize_vec_sum(&g._in_degree),
        "Edge count is invalid"
      );
      assert_eq!(g.vert_count(), graph_size, "Vert count is invalid");
    }

    Ok(())
  }

  #[test]
  #[should_panic(expected = "Vertex 99 is out of index range 0-0")]
  pub fn test_validate_vertex_invalid() {
    let g = Digraph::new(0);
    if let Err(err) = g.validate_vertex(99) {
      panic!("{}", err);
    }
  }

  #[test]
  #[should_panic(expected = "Vertex 99 is out of index range 0-2")]
  pub fn test_validate_vertex_invalid_2() {
    let g = Digraph::new(3);
    if let Err(err) = g.validate_vertex(99) {
      panic!("{}", err);
    }
  }

  #[test]
  pub fn test_validate_vertex_valid() {
    let g = Digraph::new(3);

    // Call validate_vertex for valid indices;  Shouldn't panic:
    for i in 0..2 {
      if let Err(err) = g.validate_vertex(i) {
        panic!("{}", err);
      }
    }
  }

  #[test]
  pub fn test_reverse() -> Result<(), String> {
    for graph_size in [0, 1, 3, 9] {
      let mut dg = Digraph::new(graph_size);

      // Add edges for each vertex.
      if graph_size > 1 {
        // For each vertex from `0` to `limit` add edges subsequent vertices.
        for i in 0..graph_size {
          for j in (i + 1)..graph_size {
            dg.add_edge(i, j)?;
          }
        }
      }

      // println!("For {:?}", &dg);

      let rev_dg = dg.reverse()?;

      // println!("Result {:?}", &rev_dg);

      assert_eq!(rev_dg.vert_count(), dg.vert_count());
      assert_eq!(rev_dg.edge_count(), dg.edge_count());

      // Verify in/out degrees are reversed as well
      for i in 0..graph_size {
        let rslt = rev_dg.indegree(i)?;
        let expected = dg.indegree(graph_size - i - 1)?;
        assert_eq!(
          rslt, expected,
          "rev_dg.indegree({:?}) should equal {:?}",
          i, expected
        );
      }
    }

    Ok(())
  }
}

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::utils::extract_vert_and_edge_counts_from_bufreader;

/// Returns panic message for invalid vertices;  Exported for use in testing.
pub fn invalid_vertex_msg(v: usize, max_v: usize) -> String {
  format!("Vertex {} is out of index range 0-{}", v, max_v)
}

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

  /// Returns vertex count.
  pub fn vert_count(&self) -> usize {
    self._adj_lists.len()
  }

  /// Returns number of edges in graph.
  pub fn edge_count(&self) -> usize {
    self._edge_count
  }

  /// Returns a `Result` containing given vertex' adjacency list, or an
  ///  'vertex out of bounds' error message..
  /// ```rust
  /// use walrs_digraph::digraph::Digraph;
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
    self.validate_vertex(v)?;
    Ok(&self._adj_lists[v])
  }

  /// Returns outdegree of given vertex `v`.
  /// @todo Should return `Option<usize>` instead.
  /// Example:
  /// ```rust
  /// use walrs_digraph::digraph::Digraph;
  /// use walrs_digraph::invalid_vertex_msg;
  ///
  /// let mut digraph = Digraph::new(3);
  ///
  /// // Add edges
  /// digraph.add_edge(0, 1).unwrap();
  /// digraph.add_edge(0, 2).unwrap();
  ///
  /// // Returns outdegree for valid vertices
  /// assert_eq!(digraph.outdegree(1), Ok(0));
  /// assert_eq!(digraph.outdegree(2), Ok(0));
  /// assert_eq!(digraph.outdegree(0), Ok(2));
  ///
  /// // Returns `Err(String)` for invalid vertex
  /// assert_eq!(digraph.outdegree(3), Err(invalid_vertex_msg(3, 2)));
  /// ```
  pub fn outdegree(&self, v: usize) -> Result<usize, String> {
    self.validate_vertex(v)?;
    Ok(self._adj_lists[v].len())
  }

  /// Returns indegree of given vertex `v`.
  /// @todo Should return `Option<usize>` instead.
  /// Example:
  /// ```rust
  /// use walrs_digraph::digraph::Digraph;
  /// use walrs_digraph::invalid_vertex_msg;
  ///
  /// let mut digraph = Digraph::new(3);
  ///
  /// // Add edges
  /// digraph.add_edge(0, 1).unwrap();
  /// digraph.add_edge(2, 1).unwrap();
  ///
  /// // Returns indegree for valid vertices
  /// assert_eq!(digraph.indegree(0), Ok(0));
  /// assert_eq!(digraph.indegree(2), Ok(0));
  /// assert_eq!(digraph.indegree(1), Ok(2));
  ///
  /// // Returns `Err(String)` for invalid vertex
  /// assert_eq!(digraph.indegree(3), Err(invalid_vertex_msg(3, 2)));
  /// ```
  pub fn indegree(&self, v: usize) -> Result<usize, String> {
    self.validate_vertex(v)?;
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
    self
      .validate_vertex(v)
      .and_then(|_| self.validate_vertex(w))?;

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
      return Err(invalid_vertex_msg(v, if len > 0 { len - 1 } else { 0 }));
    }
    Ok(self)
  }

  /// Returns a copy of given Digraph "reversed".
  pub fn reverse(&self) -> Result<Self, String> {
    let v_count = self.vert_count();
    let mut out = Digraph::new(v_count);
    if v_count == 0 {
      return Ok(out);
    }
    for v in 0..v_count {
      for w in &self._adj_lists[v] {
        out.add_edge(*w, v)?;
      }
    }
    Ok(out)
  }
}

impl<R: std::io::Read> TryFrom<&mut BufReader<R>> for Digraph {
  type Error = Box<dyn std::error::Error>;

  ///  Creates a Digraph from a buffer reader representing a text file formatted as:
  ///
  ///  ```text
  ///  {num_verts}
  ///  {num_edges}
  ///  {vertex} {weight}
  ///  {vertex} {weight} {weight} ...
  ///  ...
  ///  ```
  /// `weight` here is a vertex/weight reachable from vertex `vertex`.
  ///
  fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
    // Extract vert count, and move cursor passed edge count line, for reader
    let vert_count = extract_vert_and_edge_counts_from_bufreader(reader).unwrap().0;

    // Construct digraph
    let mut dg = Digraph::new(vert_count);

    // Populate graph from buffer lines
    for line in  reader.lines() {
      // For each edge definition, enter them into graph
      let _line = line?;
      // Split and parse edge values to integers
      let verts: Vec<usize> = _line
          .split_ascii_whitespace()
          .map(|x| x.parse::<usize>().unwrap())
          .collect();

      // Add edge, and panic if unable to add it
      dg.add_edge(verts[0], verts[1])?;
    }

    // Return graph
    Ok(dg)
  }
}

impl<R: std::io::Read> TryFrom<BufReader<R>> for Digraph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(mut reader: BufReader<R>) -> Result<Self, Self::Error> {
    Digraph::try_from(&mut reader)
  }
}

impl TryFrom<&File> for Digraph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(file_struct: &File) -> Result<Self, Self::Error> {
    Digraph::try_from(&mut BufReader::new(file_struct))
  }
}

impl TryFrom<File> for Digraph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(file_struct: File) -> Result<Self, Self::Error> {
    Digraph::try_from(&mut BufReader::new(file_struct))
  }
}

#[cfg(test)]
mod test {
  use std::fs::File;
  use std::io::{BufReader, Seek};
  use crate::{extract_vert_and_edge_counts_from_bufreader, invalid_vertex_msg, Digraph};

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
        assert_eq!(g.adj(i).unwrap().len(), 0);
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
            g.add_edge(i, j).expect("Error should never occur here");
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
      //   "For {:?} with {} vertices;  Expecting {} edges",
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
          expected_indegree
        );
        assert_eq!(
          g.outdegree(i)?,
          expected_outdegree
        );
      }
    }

    let g = Digraph::new(3);

    // Returns `Err(String)` for invalid vertex
    assert_eq!(g.outdegree(99), Err(invalid_vertex_msg(99, 2)));
    assert_eq!(g.indegree(99), Err(invalid_vertex_msg(99, 2)));

    Ok(())
  }

  #[test]
  #[should_panic(expected = "Vertex 99 is out of index range 0-0")]
  pub fn test_adj_invalid() {
    let g = Digraph::new(0);
    g.adj(99).unwrap();
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
      //   "For {:?} with {} vertices;  Expecting {} edges",
      //   &g, graph_size, num_expected_edges
      // );

      // Test `degree` for each vertex
      for i in 0..graph_size {
        let expected_indegree = if i >= 1 { i } else { 0 };
        // println!("`#.indegree({})` should return {}", i, expected_indegree);
        assert_eq!(
          g.indegree(i)?,
          expected_indegree
        );
      }

      assert_eq!(
        g.edge_count(),
        usize_vec_sum(&g._in_degree),
        "Edge count is invalid"
      );
      assert_eq!(g.vert_count(), graph_size, "Vert count is invalid");
    }

    // Test error cases
    // ----
    let mut dg = Digraph::new(2);
    assert_eq!(
      dg.add_edge(0, 99).unwrap_err(),
      invalid_vertex_msg(99, 1)
    );
    assert_eq!(
      dg.add_edge(99, 0).unwrap_err(),
      invalid_vertex_msg(99, 1)
    );

    Ok(())
  }

  #[test]
  #[should_panic(expected = "Vertex 99 is out of index range 0-0")]
  pub fn test_validate_vertex_invalid() {
    let g = Digraph::new(0);
    g.validate_vertex(99).unwrap();
  }

  #[test]
  #[should_panic(expected = "Vertex 99 is out of index range 0-2")]
  pub fn test_validate_vertex_invalid_2() {
    let g = Digraph::new(3);
    g.validate_vertex(99).unwrap();
  }

  #[test]
  pub fn test_validate_vertex_valid() {
    let g = Digraph::new(3);

    // Call validate_vertex for valid indices;  Shouldn't panic:
    for i in 0..2 {
      g.validate_vertex(i).unwrap();
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

  #[test]
  pub fn test_try_from_file_ref() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get digraph data
    let f = File::open(&file_path)?;

    // Create graph
    let _: Digraph = (&f).try_into().unwrap();

    Ok(())
  }

  #[test]
  pub fn test_try_from_file() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get digraph data
    let f = File::open(&file_path)?;

    // Create graph
    let _: Digraph = f.try_into().unwrap();

    Ok(())
  }

  #[test]
  pub fn test_try_from_mut_buf_reader_ref() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get digraph data
    let f = File::open(&file_path)?;
    let mut reader = BufReader::new(f);

    // Create graph (impls for `From<BufReader<R: std::io::Read>>` and `From<File>` are defined for `Digraph` struct
    let dg: Digraph = (&mut reader).try_into()?;

    // Rewind reader and extract vert and edge count from first lines
    reader.rewind()?;

    let (expected_vert_count, expected_edge_count) =
        extract_vert_and_edge_counts_from_bufreader(&mut reader)?;

    assert_eq!(
      dg.vert_count(),
      expected_vert_count,
      "Vert count is invalid"
    );
    assert_eq!(
      dg.edge_count(),
      expected_edge_count,
      "Edge count is invalid"
    );

    Ok(())
  }

  #[test]
  pub fn test_try_from_buf_reader() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get digraph data
    let f = File::open(&file_path)?;

    // Create graph
    let _: Digraph = BufReader::new(f).try_into().unwrap();

    Ok(())
  }
}

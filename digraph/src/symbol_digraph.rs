use std::io::{BufRead, BufReader};

use crate::Digraph;

/// `DisymGraph` A Directed Acyclic Graph (B-DAG) data structure.
/// @todo Consider renaming struct to `SymbolDigraph`.
///
/// ```rust
/// // @todo
/// ```
#[derive(Debug, Clone)]
pub struct DisymGraph {
  _vertices: Vec<String>,
  _graph: Digraph,
}

impl DisymGraph {
  /// Creates a directed symbol graph.
  pub fn new() -> Self {
    DisymGraph {
      _vertices: Vec::new(),
      _graph: Digraph::new(0),
    }
  }

  /// Returns vertex count
  pub fn vert_count(&self) -> usize {
    self._graph.vert_count()
  }

  /// Returns number of edges in graph.
  pub fn edge_count(&self) -> usize {
    self._graph.edge_count()
  }

  /// Returns the number of edges from this vertex to other vertices.
  pub fn outdegree(&self, n: usize) -> Result<usize, String> {
    self._graph.outdegree(n)
  }

  /// Returns the number edges pointing from other vertices to the given one.
  pub fn indegree(&self, n: usize) -> Result<usize, String> {
    self._graph.indegree(n)
  }

  /// Returns given vertex' symbol adjacency list.
  pub fn adj(&self, symbol_name: &str) -> Option<Vec<&str>> {
    if let Some(indices) = self.adj_indices(symbol_name) {
      return Some(
        indices
          .into_iter()
          .map(|x| self._vertices[*x].as_str())
          .collect(),
      );
    }
    None
  }

  /// Returns given vertex' index adjacency list - A list containing adjacent indexes.
  pub fn adj_indices(&self, symbol_name: &str) -> Option<&Vec<usize>> {
    if let Some(i) = self.index(symbol_name) {
      match &self._graph.adj(i) {
        Ok(list) => Some(list),
        _ => None,
      }
    } else {
      None
    }
  }

  /// Returns a reference to contained vertex index graph.
  pub fn graph(&self) -> &Digraph {
    &self._graph
  }

  /// Returns a boolean indicating whether symbol graph contains given symbol name or not.
  pub fn contains(&self, symbol_name: &str) -> bool {
    self.has_vertex(symbol_name)
  }

  /// Returns the index of the given symbol name.
  pub fn index(&self, symbol_name: &str) -> Option<usize> {
    self._vertices.iter().position(|v| v == symbol_name)
  }

  /// Returns the indices for the given symbol strings.
  pub fn indices(&self, vs: &[&str]) -> Option<Vec<usize>> {
    if vs.is_empty() || self.vert_count() == 0 {
      None
    } else {
      Some(vs.iter().filter_map(|v| self.index(v)).collect())
    }
  }

  /// Returns the name of the given symbol index.
  pub fn name(&self, symbol_idx: usize) -> Option<String> {
    self._vertices.get(symbol_idx).map(|x| x.to_string())
  }

  pub fn name_as_ref(&self, symbol_idx: usize) -> Option<&str> {
    self._vertices.get(symbol_idx).map(|x| x.as_ref())
  }

  /// Returns the symbol names for the given indices.
  pub fn names(&self, indices: &[usize]) -> Option<Vec<String>> {
    if indices.is_empty() || self.vert_count() == 0 {
      None
    } else {
      Some(indices.iter().filter_map(|i| self.name(*i)).collect())
    }
  }

  /// Adds a symbol vertex to the graph.
  pub fn add_vertex(&mut self, v: &str) -> usize {
    // @todo should accept `ToString`
    if let Some(i) = self.index(v) {
      i
    } else {
      let i = self._vertices.len();
      self._vertices.push(v.to_string());
      self._graph.add_vertex(i);
      i
    }
  }

  /// Checks if graph has vertex.
  pub fn has_vertex(&self, value: &str) -> bool {
    self.index(value).is_some()
  }

  /// Checks if graph contains vertex
  pub fn validate_vertex(&self, v: &str) -> Result<&Self, String> {
    if let Some(v) = self.index(v) {
      return match self._graph.validate_vertex(v) {
        Ok(_) => Ok(self),
        Err(err) => Err(err),
      };
    }
    Ok(self)
  }

  /// Adds edge to graph
  /// @todo Method should not return `Result`, it should fail on failure.
  /// @todo Should accept `ToString`.
  pub fn add_edge(&mut self, vertex: &str, weights: &[&str]) -> Result<&mut Self, String> {
    let v1 = self.add_vertex(vertex);

    // Ensure each edge "end" vertex is attached to DAG
    for w in weights {
      let v2 = self.add_vertex(w);

      // Add edges
      if let Err(err) = self._graph.add_edge(v1, v2) {
        // @todo Should `panic!` here, on failure
        return Err(err);
      }
    }

    Ok(self)
  }

  /// Returns a reversed version of symbol graph instance.
  pub fn reverse(&self) -> Result<Self, String> {
    let _graph = self._graph.reverse()?;
    Ok(DisymGraph {
      _vertices: self._vertices.clone(),
      _graph,
    })
  }

  /// Populates digraph instance from incoming lines.  @note Returns Error string if not all
  /// vertices are in graph (use `#.add_vertex(...)` to enter the max vertex (expected vertex count)
  /// into the graph before digesting buffer lines.
  pub fn digest_lines<R: std::io::Read>(
    &mut self,
    lines: std::io::Lines<&mut BufReader<R>>,
  ) -> Result<&Self, Box<dyn std::error::Error>> {
    // Loop through lines
    for line in lines {
      // For each edge definition, enter them into graph
      match line {
        // If line
        Ok(_line) => {
          // Split and parse edge values to integers
          let verts: Vec<&str> = _line.split_ascii_whitespace().collect();

          // Add edge, and panic if unable to add it
          if let Err(err) = self.add_edge(verts[0], &verts[1..]) {
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

impl Default for DisymGraph {
  fn default() -> Self {
    Self::new()
  }
}

/// `From` trait usage example.
///
/// ```rust
/// use walrs_digraph::symbol_digraph::DisymGraph;
/// use std::io::{BufRead, BufReader, Lines};
/// use std::fs::File;
///
/// let file_path = "../test-fixtures/symbol_graph_test_routes.txt";
///
///  // Get graph 'symbol' data
///  let f = match File::open(&file_path) {
///    Ok(f) => f,
///    Err(err) => panic!("{}", err)
///  };
///
///  // Get mutable reader
///  let mut reader = BufReader::new(f);
///
///  // Create graph
///  let dg: DisymGraph = (&mut reader).into();
///
///  println!("{:?}", dg);
/// ```
impl<R: std::io::Read> From<&mut BufReader<R>> for DisymGraph {
  fn from(reader: &mut BufReader<R>) -> Self {
    // Construct graph
    let mut dg = DisymGraph::new();

    // Populate graph from buffer lines
    if let Err(err) = dg.digest_lines(reader.lines()) {
      panic!("{:?}", err);
    }

    // Return graph
    dg
  }
}

#[cfg(test)]
mod test {
  use std::fs::File;
  use std::io::BufReader;

  use crate::symbol_digraph::DisymGraph;

  #[test]
  fn test_new() -> Result<(), String> {
    let mut dsg = DisymGraph::new();

    let guest = "Guest".to_string();
    let user = "User".to_string();
    let admin = "Admin".to_string();

    dsg.add_edge(&user, &[&guest])?;
    dsg.add_edge(&admin, &[&user])?;

    Ok(())
  }

  #[test]
  fn test_vert_count() -> Result<(), String> {
    let mut dsg = DisymGraph::new();
    let symbols: Vec<&str> = "all your base are belong to us"
      .split_ascii_whitespace()
      .collect();
    let limit = symbols.len() - 1;
    for (i, s) in symbols.iter().enumerate() {
      if i == limit {
        break;
      }
      dsg.add_edge(s, &[symbols[i + 1]])?;
    }

    assert_eq!(dsg.vert_count(), symbols.len(), "vert_count is invalid");

    Ok(())
  }

  #[test]
  fn test_edge_count() -> Result<(), String> {
    let mut dsg = DisymGraph::new();
    let symbols: Vec<&str> = "all your base are belong to us"
      .split_ascii_whitespace()
      .collect();
    let limit = symbols.len() - 1;
    for (i, s) in symbols.iter().enumerate() {
      if i == limit {
        break;
      }
      dsg.add_edge(s, &[symbols[i + 1]])?;
    }
    assert_eq!(dsg.edge_count(), symbols.len() - 1, "edge_count is invalid");
    Ok(())
  }

  #[test]
  fn test_indegree() -> Result<(), String> {
    let mut dsg = DisymGraph::new();

    // Add multiple weights when adding edge
    let vowels: Vec<&str> = "a e i o u".split_ascii_whitespace().collect();

    for (i, v) in vowels.iter().enumerate() {
      let weights = &vowels[i + 1..];

      dsg.add_edge(v, weights)?;

      // println!(
      //   "for vowel \"{}\" weights {:?};  indegree: {};",
      //   v, weights, indegree
      // );
      assert_eq!(dsg.indegree(i)?, i, "invalid indegree");
    }

    Ok(())
  }

  #[test]
  fn test_outdegree() -> Result<(), String> {
    let mut dsg = DisymGraph::new();

    // Add multiple weights when adding edge
    let vowels: Vec<&str> = "a e i o u".split_ascii_whitespace().collect();
    let limit = vowels.len() - 1;

    for (i, v) in vowels.iter().enumerate() {
      let weights = &vowels[i + 1..];

      dsg.add_edge(v, weights)?;

      // println!(
      //   "for vowel \"{}\" weights {:?};  outdegree: {};",
      //   v, weights, outdegree
      // );
      assert_eq!(dsg.outdegree(i)?, limit - i, "invalid indegree");
    }

    Ok(())
  }

  #[test]
  fn test_adj_indices() -> Result<(), String> {
    let mut dsg = DisymGraph::new();
    assert_eq!(
      dsg.adj_indices("non-existing-symbol").is_none(),
      true,
      "Should return 'None' on empty graph"
    );

    let vowels: Vec<&str> = "a e i o u".split_ascii_whitespace().collect();
    for (i, v) in vowels.iter().enumerate() {
      let weights = &vowels[i + 1..];
      dsg.add_edge(v, weights)?;

      match dsg.adj_indices(v) {
        Some(adj_indices) => {
          assert_eq!(
            adj_indices.len(),
            weights.len(),
            "vertices adjacent to \"{}\" have an invalid length",
            v
          );

          // Verify vertices adjacent to `v` are found in `weights`
          adj_indices.iter().for_each(|x| {
            let name = dsg.name(*x).unwrap();
            assert!(
              weights.contains(&name.as_str()),
              "vertices adjacent to \"{}\" are invalid",
              v
            );
          });
        }
        None => panic!("Expected vertices adjacent to \"{}\";  Received `None`", v),
      };
    }

    Ok(())
  }

  #[test]
  fn test_contains() {
    let mut dsg = DisymGraph::new();

    assert_eq!(
      dsg.contains("hello"),
      false,
      "Empty graph instances shouldn't contain any vertices."
    );
    let v1 = "abc";
    let v2 = "efg";
    dsg.add_vertex(v1);
    dsg.add_vertex(v2);

    assert_eq!(
      dsg.contains(v1),
      true,
      "graph should contain symbol \"{}\"",
      v1
    );
    assert_eq!(
      dsg.contains(v2),
      true,
      "graph should contain symbol \"{}\"",
      v2
    );
  }

  #[test]
  fn test_index() {
    let mut dsg = DisymGraph::new();
    assert_eq!(
      dsg.index("abc").is_none(),
      true,
      "Empty graphs shouldn't contain indices for non-existent vertices"
    );

    let v1 = "abc";
    let v2 = "efg";
    dsg.add_vertex(v1);
    dsg.add_vertex(v2);

    assert_eq!(
      dsg.index(v1).unwrap(),
      0,
      "graph should contain vertex \"{}\" at index {}",
      v1,
      0
    );
    assert_eq!(
      dsg.index(v2).unwrap(),
      1,
      "graph should contain vertex \"{}\" at index {}",
      v2,
      1
    );
  }

  #[test]
  fn test_indices() {
    let mut dsg = DisymGraph::new();
    let symbols: Vec<&str> = "all your base are belong to us"
      .split_ascii_whitespace()
      .collect();

    assert_eq!(
      dsg.indices(&symbols).is_none(),
      true,
      "Empty graphs shouldn't contain indices for non-existent vertices"
    );

    let v1 = "abc";
    let v2 = "efg";
    let verts = [v1, v2];
    dsg.add_vertex(v1);
    dsg.add_vertex(v2);

    assert_eq!(
      dsg.indices(&verts).unwrap(),
      vec![0, 1],
      "graph should contain indices for existing vertices"
    );
  }

  #[test]
  fn test_name() {
    let mut dsg = DisymGraph::new();
    assert_eq!(
      dsg.name(0).is_none(),
      true,
      "Empty graphs shouldn't contain symbols for non-existent indices"
    );

    let v1 = "abc";
    let v2 = "efg";
    dsg.add_vertex(v1);
    dsg.add_vertex(v2);

    assert_eq!(
      dsg.name(0).unwrap(),
      v1,
      "index for vertex \"{}\" should equal {}",
      v1,
      0
    );
    assert_eq!(
      dsg.name(1).unwrap(),
      v2,
      "index for vertex \"{}\" should equal {}",
      v2,
      1
    );
  }

  #[test]
  fn test_names() {
    let mut dsg = DisymGraph::new();
    let mut symbols: Vec<&str> = "all your base are belong to us"
      .split_ascii_whitespace()
      .collect();
    let indices: Vec<usize> = (0..symbols.len()).collect();

    assert_eq!(
      dsg.names(&indices).is_none(),
      true,
      "Empty graphs shouldn't contain names for non-existent indices"
    );

    let v1 = "abc";
    let v2 = "efg";
    dsg.add_vertex(v1);
    dsg.add_vertex(v2);

    assert_eq!(
      dsg.names(&indices).unwrap().sort(),
      symbols.sort(),
      "graph should contain indices for existing vertices"
    );
  }

  #[test]
  fn test_add_vertex() -> Result<(), String> {
    let mut dsg = DisymGraph::new();
    let symbols: Vec<&str> = "all your base are belong to us"
      .split_ascii_whitespace()
      .collect();
    for s in symbols.iter() {
      dsg.add_vertex(s);
    }
    assert_eq!(dsg.vert_count(), symbols.len(), "vert_count is invalid");

    // Test already vertices aren't added more than once
    dsg.add_vertex(symbols[0]);
    assert_eq!(
      dsg.vert_count(),
      symbols.len(),
      "vert_count should not increase when vertex is already added"
    );

    Ok(())
  }

  #[test]
  fn test_has_vertex() {
    let mut dsg = DisymGraph::new();
    assert_eq!(
      dsg.has_vertex("abc"),
      false,
      "Empty graphs shouldn't contain any vertices"
    );

    let v1 = "abc";
    let v2 = "efg";
    dsg.add_vertex(v1);
    dsg.add_vertex(v2);

    assert_eq!(dsg.has_vertex(v1), true, "should contain vertex \"{}\"", v1);
    assert_eq!(dsg.has_vertex(v2), true, "should contain vertex \"{}\"", v2);
    assert_eq!(
      dsg.has_vertex("non-existent"),
      false,
      "shouldn't contain non-existent vertex"
    );
  }

  #[test]
  fn test_add_edge() -> Result<(), String> {
    let mut dsg = DisymGraph::new();
    let symbols: Vec<&str> = "all your base are belong to us"
      .split_ascii_whitespace()
      .collect();
    let symbol_limit = symbols.len() - 1;

    for (i, s) in symbols.iter().enumerate() {
      if i == symbol_limit {
        break;
      }
      dsg.add_edge(s, &[symbols[i + 1]])?;
    }

    assert_eq!(dsg.edge_count(), symbols.len() - 1, "edge_count is invalid");

    // Add multiple weights when adding edge
    let vowels: Vec<&str> = "a e i o u".split_ascii_whitespace().collect();
    let limit = vowels.len() - 1;

    for (i, v) in vowels.iter().enumerate() {
      let weights = &vowels[i + 1..];
      let index = i + symbol_limit + 1;

      dsg.add_edge(v, weights)?;

      assert_eq!(dsg.indegree(index)?, i, "invalid indegree");
      assert_eq!(dsg.outdegree(index)?, limit - i, "invalid outdegree");
    }

    // println!("{:?}", &dsg);
    assert_eq!(
      dsg.edge_count(),
      symbols.len() - 1 + (vowels.len() * 2),
      "invalid edge_count"
    );
    assert_eq!(
      dsg.vert_count(),
      symbols.len() + vowels.len(),
      "invalid vert_count"
    );

    Ok(())
  }

  #[test]
  fn test_reverse() -> Result<(), String> {
    let mut dsg = DisymGraph::new();
    let symbols: Vec<&str> = "all your base are belong to us"
      .split_ascii_whitespace()
      .collect();
    let symbol_limit = symbols.len() - 1;

    // Add edges from every to the ones that come after it
    for (i, s) in symbols.iter().enumerate() {
      let weights = &symbols[i + 1..];

      dsg.add_edge(s, weights)?;

      assert_eq!(dsg.indegree(i)?, i, "Expected indegree to equal `{}`", i);
      assert_eq!(
        dsg.outdegree(i)?,
        symbol_limit - i,
        "Expected outdegree to equal `{}`",
        symbol_limit - i
      );
    }

    // Get "reversed" instance of digraph
    let dsg_reversed = dsg.reverse()?;

    // Log graphs
    // println!("\nOriginal Graph:\n {:?}\n", &dsg);
    // println!("Reversed Graph:\n {:?}\n", &dsg_reversed);

    // Check graph counts
    assert_eq!(
      dsg_reversed.vert_count(),
      dsg.vert_count(),
      "Reversed graph's vert_count should equal original graphs evert count"
    );
    assert_eq!(
      dsg_reversed.edge_count(),
      dsg.edge_count(),
      "Reversed graph's edge_count should equal original graphs edge count"
    );

    // Test result
    for (i, _) in symbols.iter().enumerate() {
      let v2_name = dsg_reversed.name(i).unwrap();
      let v2_weights = dsg_reversed.adj(&v2_name).unwrap();
      let v2_weights_expected = &symbols[0..i];

      // Log values
      // println!("\nv1 {:?} weights: {:?}", &v1_name, v1_weights);
      // println!("v2 {:?} weights: {:?}", &v2_name, v2_weights);
      // println!(
      //   "v2 {:?} weights expected: {:?}",
      //   &v2_name, v2_weights_expected
      // );

      // Test values
      // ----
      // Weight lengths
      assert_eq!(
        v2_weights.len(),
        v2_weights_expected.len(),
        "v2 weights length are not equal to \"v2 expected weights length\""
      );

      // out/in degrees
      assert_eq!(
        dsg_reversed.outdegree(i),
        dsg.indegree(i),
        "Reversed graph's `outdegree({})` should equal original graph's `indegree({})",
        i,
        i
      );

      // in/out degrees
      assert_eq!(
        dsg_reversed.indegree(i),
        dsg.outdegree(i),
        "Reversed graph's `indegree({})` should equal original graph's `outdegree({})",
        i,
        i
      );

      // Ensure expected weights match received weights
      for (j, w) in v2_weights_expected.into_iter().enumerate() {
        assert_eq!(
          &v2_weights[j], w,
          "v2_weights[{}] should v2_weights_expected[{}]",
          j, j
        );
      }
    }

    Ok(())
  }

  #[test]
  fn test_from_mut_bufreader_impl() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "../test-fixtures/symbol_graph_test_routes.txt";

    // Get graph 'symbol' data
    let f = File::open(&file_path)?;

    // Get mutable reader
    let mut reader = BufReader::new(f);

    // Create graph
    let _: DisymGraph = (&mut reader).into();

    // println!("{:?}", dg);

    Ok(())
  }
}

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::Digraph;

pub fn invalid_vert_symbol_msg(v: &str) -> String {
  format!("Invalid vertex symbol '{}' not found in graph.", v)
}

pub type DisymGraphData = Vec<(String, Option<Vec<String>>)>;

/// `DisymGraph` A Directed Acyclic Graph (B-DAG) data structure.
///
/// ```rust
/// // TODO
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
          .iter()
          .map(|x| self._vertices[*x].as_str())
          .collect(),
      );
    }
    None
  }

  /// Returns given vertex' index adjacency list - A list containing adjacent indexes.
  pub fn adj_indices(&self, symbol_name: &str) -> Option<&Vec<usize>> {
    if let Some(i) = self.index(symbol_name) {
      self._graph.adj(i).unwrap().into()
    } else {
      None
    }
  }

  /// Returns a reference to contained vertex index graph.
  /// @todo Consider returning a copy here.
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
    match self.index(v) {
      Some(_) => Ok(self),
      None => Err(invalid_vert_symbol_msg(v)),
    }
  }

  /// Adds edge to graph
  pub fn add_edge(&mut self, vertex: &str, weights: &[&str]) -> Result<&mut Self, String> {
    let v1 = self.add_vertex(vertex);

    // Ensure each edge "end" vertex is attached to DAG
    for w in weights {
      let v2 = self.add_vertex(w);

      // Add edges
      self._graph.add_edge(v1, v2)?;
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
}

impl Default for DisymGraph {
  fn default() -> Self {
    Self::new()
  }
}

impl TryFrom<&DisymGraphData> for DisymGraph {
  type Error = String;

  fn try_from(data: &DisymGraphData) -> Result<Self, Self::Error> {
    let mut graph = DisymGraph::new();

    for (vertex, edges) in data.iter() {
      // Handle Option<Vec<String>> - if None, add vertex with no edges
      if let Some(edge_list) = edges {
        let edge_refs: Vec<&str> = edge_list.iter().map(|s| s.as_str()).collect();
        graph.add_edge(vertex.as_str(), &edge_refs)?;
      } else {
        // Just add the vertex without any edges
        graph.add_vertex(vertex.as_str());
      }
    }

    Ok(graph)
  }
}

impl TryFrom<DisymGraphData> for DisymGraph {
  type Error = String;

  fn try_from(data: DisymGraphData) -> Result<Self, Self::Error> {
    DisymGraph::try_from(&data)
  }
}

/// `From` trait usage example.
///
/// ```rust
/// use walrs_digraph::disymgraph::DisymGraph;
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
///  let dg: DisymGraph = (&mut reader).try_into().unwrap();
///
///  println!("{:?}", dg);
/// ```
impl<R: std::io::Read> TryFrom<&mut BufReader<R>> for DisymGraph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(reader: &mut BufReader<R>) -> Result<DisymGraph, Self::Error> {
    // Construct graph
    let mut dg = DisymGraph::new();
    let lines = reader.lines();
    // Populate graph from buffer lines
    for line in lines {
      // For each edge definition, enter them into graph
      let _line = line?;

      // Split and parse edge values to integers
      let verts: Vec<&str> = _line.split_ascii_whitespace().collect();

      // Add edge, and panic if unable to add it
      dg.add_edge(verts[0], &verts[1..])?;
    }

    // Return graph
    Ok(dg)
  }
}

impl<R: std::io::Read> TryFrom<BufReader<R>> for DisymGraph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(mut reader: BufReader<R>) -> Result<Self, Self::Error> {
    DisymGraph::try_from(&mut reader)
  }
}

impl TryFrom<&File> for DisymGraph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(file_struct: &File) -> Result<Self, Self::Error> {
    DisymGraph::try_from(&mut BufReader::new(file_struct))
  }
}

impl TryFrom<File> for DisymGraph {
  type Error = Box<dyn std::error::Error>;

  fn try_from(file_struct: File) -> Result<Self, Self::Error> {
    DisymGraph::try_from(&mut BufReader::new(file_struct))
  }
}

#[cfg(test)]
mod test {
  use std::fs::File;
  use std::io::{BufReader};

  use crate::disymgraph::DisymGraph;
  use crate::{invalid_vert_symbol_msg};

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
  fn test_default() {
    let dsg = DisymGraph::default();

    assert_eq!(dsg.vert_count(), 0);
    assert_eq!(dsg.edge_count(), 0);
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
  fn test_adj() -> Result<(), String> {
    let mut dsg = DisymGraph::new();
    assert!(dsg.adj("unknonwn-symbol").is_none());

    // Add edges and assert [adjacency] lists
    // ----
    let vowels: Vec<&str> = "a e i o u".split_ascii_whitespace().collect();

    for (i, v) in vowels.iter().enumerate() {
      let weights = &vowels[i + 1..];
      dsg.add_edge(v, weights)?;

      let adj = dsg.adj(v).unwrap();
      assert_eq!(
        adj.len(),
        weights.len(),
        "vertices adjacent to \"{}\" have an invalid length",
        v
      );

      // Verify vertices adjacent to `v` are found in `weights`
      adj.iter().for_each(|name| {
        assert!(
          weights.contains(name),
          "vertices adjacent to \"{}\" are invalid",
          v
        );
      });
    }

    assert!(dsg.adj("unknonwn-symbol").is_none());

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

      let adj_indices = dsg.adj_indices(v).unwrap();
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
  fn test_name_as_ref() {
    let mut dsg = DisymGraph::new();
    assert_eq!(
      dsg.name_as_ref(0).is_none(),
      true,
      "Empty graphs shouldn't contain symbols for non-existent indices"
    );

    let v1 = "abc";
    let v2 = "efg";
    dsg.add_vertex(v1);
    dsg.add_vertex(v2);

    assert_eq!(
      dsg.name_as_ref(0).unwrap(),
      v1,
      "index for vertex \"{}\" should equal {}",
      v1,
      0
    );
    assert_eq!(
      dsg.name_as_ref(1).unwrap(),
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
      let outdegree_idx = symbol_limit - i;
      assert_eq!(
        dsg.outdegree(i)?,
        outdegree_idx,
        "Expected outdegree to equal `{}`",
        outdegree_idx
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
  fn test_validate_vertex() -> Result<(), String> {
    let mut dsg = DisymGraph::new();
    let v1 = "abc";
    let v2 = "efg";
    dsg.add_vertex(v1);
    dsg.add_vertex(v2);

    // Validate existing vertices
    dsg.validate_vertex(v1)?;
    dsg.validate_vertex(v2)?;

    // Validate non-existing vertex
    assert_eq!(
      dsg.validate_vertex("non-existent").unwrap_err(),
      invalid_vert_symbol_msg("non-existent")
    );

    Ok(())
  }

  #[test]
  fn test_from_mut_bufreader() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "../test-fixtures/symbol_graph_test_routes.txt";

    // Get graph 'symbol' data
    let f = File::open(&file_path)?;

    // Get mutable reader
    let mut reader = BufReader::new(f);

    // Create graph
    let _: DisymGraph = (&mut reader).try_into()?;

    // println!("{:?}", dg);

    Ok(())
  }

  #[test]
  fn test_try_from_file_ref() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/symbol_graph_test_routes.txt";

    // Get digraph data
    let f = File::open(file_path)?;

    // Create graph
    let _: DisymGraph = (&f).try_into().unwrap();

    Ok(())
  }

  #[test]
  fn test_try_from_file() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/symbol_graph_test_routes.txt";

    // Get digraph data
    let f = File::open(file_path)?;

    // Create graph
    let _: DisymGraph = f.try_into().unwrap();

    Ok(())
  }

  #[test]
  fn test_try_from_mut_buf_reader() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "../test-fixtures/symbol_graph_test_routes.txt";

    // Get digraph data
    let f = File::open(file_path)?;
    let mut reader = BufReader::new(f);

    // Create graph (impls for `From<BufReader<R: std::io::Read>>` and `From<File>` are defined for `DisymGraph` struct
    let dg: DisymGraph = (&mut reader).try_into()?;

    assert!(dg.vert_count() > 0,
      "Vert count is invalid"
    );
    assert!(
      dg.edge_count() > 0,
      "Edge count is invalid"
    );

    Ok(())
  }

  #[test]
  fn test_try_from_buf_reader() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/symbol_graph_test_routes.txt";

    // Get digraph data
    let f = File::open(file_path)?;

    // Create graph
    let _: DisymGraph = BufReader::new(f).try_into().unwrap();

    Ok(())
  }

  #[test]
  fn test_try_from_disymgraph_data() -> Result<(), String> {
    use crate::disymgraph::DisymGraphData;

    // Create test data: a simple graph with roles hierarchy
    // Note: Using Option<Vec<String>> to represent edges - None means no edges
    let data: DisymGraphData = vec![
      ("admin".to_string(), Some(vec!["user".to_string(), "moderator".to_string()])),
      ("user".to_string(), Some(vec!["guest".to_string()])),
      ("moderator".to_string(), Some(vec!["user".to_string()])),
      ("guest".to_string(), None), // No edges - leaf node
    ];

    // Convert to DisymGraph
    let graph = DisymGraph::try_from(&data)?;

    // Verify the graph structure
    assert_eq!(graph.vert_count(), 4, "Should have 4 vertices");
    assert!(graph.has_vertex("admin"), "Should have admin vertex");
    assert!(graph.has_vertex("user"), "Should have user vertex");
    assert!(graph.has_vertex("moderator"), "Should have moderator vertex");
    assert!(graph.has_vertex("guest"), "Should have guest vertex");

    // Verify edges
    let admin_adj = graph.adj("admin").expect("Admin should have adjacencies");
    assert_eq!(admin_adj.len(), 2, "Admin should have 2 edges");
    assert!(admin_adj.contains(&"user"), "Admin should be connected to user");
    assert!(admin_adj.contains(&"moderator"), "Admin should be connected to moderator");

    let user_adj = graph.adj("user").expect("User should have adjacencies");
    assert_eq!(user_adj.len(), 1, "User should have 1 edge");
    assert!(user_adj.contains(&"guest"), "User should be connected to guest");

    let moderator_adj = graph.adj("moderator").expect("Moderator should have adjacencies");
    assert_eq!(moderator_adj.len(), 1, "Moderator should have 1 edge");
    assert!(moderator_adj.contains(&"user"), "Moderator should be connected to user");

    let guest_adj = graph.adj("guest");
    assert!(guest_adj.is_none() || guest_adj.unwrap().is_empty(), "Guest should have no edges");

    Ok(())
  }

  #[test]
  fn test_try_from_disymgraph_data_with_empty_graph() -> Result<(), String> {
    use crate::disymgraph::DisymGraphData;

    // Create empty graph data
    let data: DisymGraphData = vec![];

    // Convert to DisymGraph
    let graph = DisymGraph::try_from(&data)?;

    // Verify empty graph
    assert_eq!(graph.vert_count(), 0, "Should have 0 vertices");
    assert_eq!(graph.edge_count(), 0, "Should have 0 edges");

    Ok(())
  }

  #[test]
  fn test_try_from_disymgraph_data_with_only_isolated_vertices() -> Result<(), String> {
    use crate::disymgraph::DisymGraphData;

    // Create graph with isolated vertices (all None edges)
    let data: DisymGraphData = vec![
      ("vertex1".to_string(), None),
      ("vertex2".to_string(), None),
      ("vertex3".to_string(), None),
    ];

    // Convert to DisymGraph
    let graph = DisymGraph::try_from(&data)?;

    // Verify the graph structure
    assert_eq!(graph.vert_count(), 3, "Should have 3 vertices");
    assert_eq!(graph.edge_count(), 0, "Should have 0 edges");
    assert!(graph.has_vertex("vertex1"), "Should have vertex1");
    assert!(graph.has_vertex("vertex2"), "Should have vertex2");
    assert!(graph.has_vertex("vertex3"), "Should have vertex3");

    // Verify no edges
    assert!(graph.adj("vertex1").is_none() || graph.adj("vertex1").unwrap().is_empty());
    assert!(graph.adj("vertex2").is_none() || graph.adj("vertex2").unwrap().is_empty());
    assert!(graph.adj("vertex3").is_none() || graph.adj("vertex3").unwrap().is_empty());

    Ok(())
  }

  #[test]
  fn test_try_from_disymgraph_data_owned() -> Result<(), String> {
    use crate::disymgraph::DisymGraphData;

    // Create test data: a simple graph with roles hierarchy
    let data: DisymGraphData = vec![
      ("root".to_string(), Some(vec!["branch_a".to_string(), "branch_b".to_string()])),
      ("branch_a".to_string(), Some(vec!["leaf".to_string()])),
      ("branch_b".to_string(), Some(vec!["leaf".to_string()])),
      ("leaf".to_string(), None),
    ];

    // Convert to DisymGraph (consuming the data)
    let graph = DisymGraph::try_from(data)?;

    // Verify the graph structure
    assert_eq!(graph.vert_count(), 4, "Should have 4 vertices");
    assert!(graph.has_vertex("root"), "Should have root vertex");
    assert!(graph.has_vertex("branch_a"), "Should have branch_a vertex");
    assert!(graph.has_vertex("branch_b"), "Should have branch_b vertex");
    assert!(graph.has_vertex("leaf"), "Should have leaf vertex");

    // Verify edges
    let root_adj = graph.adj("root").expect("Root should have adjacencies");
    assert_eq!(root_adj.len(), 2, "Root should have 2 edges");
    assert!(root_adj.contains(&"branch_a"), "Root should be connected to branch_a");
    assert!(root_adj.contains(&"branch_b"), "Root should be connected to branch_b");

    let branch_a_adj = graph.adj("branch_a").expect("Branch_a should have adjacencies");
    assert_eq!(branch_a_adj.len(), 1, "Branch_a should have 1 edge");
    assert!(branch_a_adj.contains(&"leaf"), "Branch_a should be connected to leaf");

    let branch_b_adj = graph.adj("branch_b").expect("Branch_b should have adjacencies");
    assert_eq!(branch_b_adj.len(), 1, "Branch_b should have 1 edge");
    assert!(branch_b_adj.contains(&"leaf"), "Branch_b should be connected to leaf");

    let leaf_adj = graph.adj("leaf");
    assert!(leaf_adj.is_none() || leaf_adj.unwrap().is_empty(), "Leaf should have no edges");

    Ok(())
  }
}

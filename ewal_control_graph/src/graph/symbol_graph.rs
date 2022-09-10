use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::graph::Graph;

/// `SymbolGraph` A Directed Acyclic Graph (B-DAG) data structure.
/// ```rust
/// // @todo
/// ```
#[derive(Debug)]
pub struct SymbolGraph {
  _vertices: Vec<String>,
  _graph: Graph,
}

impl SymbolGraph {
  /// Instantiates a new SymbolGraph and returns it.
  pub fn new() -> Self {
    SymbolGraph {
      _vertices: Vec::new(),
      _graph: Graph::new(0),
    }
  }

  /// Returns number of edges in graph.
  pub fn edge_count(&self) -> usize {
    self._graph.edge_count()
  }

  /// Returns vertex count
  pub fn vert_count(&self) -> usize {
    self._graph.vert_count()
  }

  /// Returns a `Result` containing given vertex' index adjacency list - A list containing adjacent indices;
  /// Else, returns an 'index is out of bounds' error string.
  pub fn adj(&self, symbol_name: &str) -> Result<&Vec<usize>, String> {
    if let Some(i) = self.index(symbol_name) {
      self._graph.adj(i)
    } else {
      Err(format!(
        "Symbol \"{}\" doesn't exist in symbol graph",
        symbol_name
      ))
    }
  }

  /// Returns contained graph.
  pub fn graph(&self) -> &Graph {
    &self._graph
  }

  /// Returns the number of edges to given vertex
  pub fn degree(&self, v: &str) -> Result<usize, String> {
    if let Some(idx) = self.index(v) {
      self._graph.degree(idx)
    } else {
      Err(format!("Vertex {} is not in graph", v)) // @todo messaging should come from reusable fn/method.
    }
  }

  /// Returns a boolean indicating whether symbol graph contains given symbol name or not.
  pub fn contains(&self, symbol_name: &str) -> bool {
    self.has_vertex(symbol_name)
  }

  /// Returns an option of "the index of the given symbol", or `None`.
  pub fn index(&self, symbol_name: &str) -> Option<usize> {
    self._vertices.iter().position(|v| v == symbol_name)
  }

  /// Returns the indices for the given symbol strings.
  pub fn indices(&self, vs: &[&str]) -> Vec<usize> {
    vs.iter().filter_map(|v| self.index(v)).collect()
  }

  /// Returns the name of the given symbol index.
  pub fn name(&self, symbol_idx: usize) -> Option<String> {
    self._vertices.get(symbol_idx).map(|x| x.to_string())
  }

  /// Returns the symbol names for the given indices.
  pub fn names(&self, indices: &[usize]) -> Vec<String> {
    indices.iter().filter_map(|i| self.name(*i)).collect()
  }

  /// Adds a symbol vertex to the graph.
  pub fn add_vertex(&mut self, v: &str) -> usize {
    if let Some(i) = self.index(v) {
      i
    } else {
      let i = self.vert_count();
      self._vertices.push(v.to_string());
      self._graph.add_vertex(i);
      i
    }
  }

  /// Checks if graph has vertex.
  pub fn has_vertex(&self, value: &str) -> bool {
    self.index(value).is_some()
  }

  /// Adds edge to graph
  pub fn add_edge(&mut self, vertex: &str, weights: Option<&[&str]>) -> Result<&mut Self, String> {
    let v1 = self.add_vertex(vertex);

    if let Some(_ws) = weights {
      // Ensure each edge "end" vertex is attached to DAG
      for w in _ws {
        let v2 = self.add_vertex(w);

        // Add edges
        if let Err(err) = self._graph.add_edge(v1, v2) {
          return Err(err);
        }
      }
    }

    Ok(self)
  }

  /*
  /// Removes a given edge (`value` -> `weight` etc.)
  pub fn remove_edge(&mut self, value: &str, weight: &str) -> &mut Self {
    if let Some(adj) = self._vert_to_index_map.get_mut(value) {
      adj.retain(|x| !(*x == weight));
    }
    self
  }

  /// Removes related edges and given vertex symbol from DAG.
  pub fn remove_vertex(&mut self, value: &str) -> &mut Self {
    // Remove vert entry from `adjacency_list`
    self._vert_to_index_map.remove(&value);

    // Remove references to vertex to remove
    for (_, v) in self._vert_to_index_map.iter_mut() {
      // Find references to `value` and remove them
      v.retain(|x| !(*x == value));
    }

    // Remove vert from `vertices`
    if let Some(i) =
      self
        ._vertices
        .iter()
        .enumerate()
        .find_map(|(i, v)| if *v == value { Some(i) } else { None })
    {
      self._vertices.remove(i);
    }

    self
  }*/
}

impl TryFrom<&mut BufReader<File>> for SymbolGraph {
  type Error = String;

  fn try_from(reader: &mut BufReader<File>) -> Result<Self, Self::Error> {
    let mut g: SymbolGraph = SymbolGraph::new();

    for (line_num, line) in reader.lines().enumerate() {
      match line {
        Ok(_line) => {
          let vs: Vec<&str> = _line.split_ascii_whitespace().collect();

          if vs.len() == 0 {
            return Err(format!(
              "Malformed symbol graph buffer at buffer line {:}",
              line_num
            ));
          }

          g.add_vertex(vs[0]);

          if vs.len() >= 2 {
            g.add_vertex(vs[1]);
            if let Err(err) = g.add_edge(vs[0], Some(&vs[1..])) {
              return Err(err);
            }
          }
        }
        Err(err) => {
          return Err(format!(
            "Malformed symbol graph buffer at buffer line {:}: {:?}",
            line_num, err
          ));
        }
      }
    }

    Ok(g)
  }
}

#[cfg(test)]
mod test {
  use crate::graph::symbol_graph::SymbolGraph;

  #[test]
  pub fn test_symbol_graph_builder_from_buf_reader() {}

  #[test]
  pub fn test_add_edge() -> Result<(), String> {
    let mut graph = SymbolGraph::new();

    // Construct vertices list to add edges from
    // ----
    let seed: &'static str = "a e i o u";
    let values: Vec<&str> = seed.split_ascii_whitespace().collect();

    // Call `#.add_edges` per vertex and test side effects
    // ----
    for (i, v) in values.iter().enumerate() {
      // Craft vertex' adjacency list
      let adjacency_list = if i > 0 { Some(&values[0..i]) } else { None };

      // Add edges
      if let Err(err) = graph.add_edge(v, adjacency_list) {
        panic!("{}", err);
      }

      // String reference to vertex
      let v_as_string = v.to_string();

      // Assert `v` is in `_vertices`
      assert!(
        graph._vertices.contains(&v_as_string),
        "SymbolGraph should contain \"{:}\" in it's vertices list.",
        &v_as_string
      );
    }

    println!("{:?}", &graph);

    // Test vertices length
    assert_eq!(
      graph.vert_count(),
      values.len(),
      "`SymbolGraph` should contain {:?} vertices",
      values.len().to_string()
    );

    let vert_count = graph.vert_count();

    // Test edges count
    assert_eq!(
      graph.edge_count(),
      // All verts have edges to other verts (vert_count - 1 = x) for each vert (x * vert_count = y) to each other vert (y * 2)
      (vert_count - 1) * vert_count,
      "`SymbolGraph` should contain {:?} edges",
      vert_count.to_string()
    );

    // println!("{:?}", &graph);

    Ok(())
  }
}

use std::fmt::Debug;

use crate::graph::Graph;

#[derive(Debug)]
pub struct DFS<'a> {
  _marked: Vec<bool>,
  _count: usize,
  _edge_to: Vec<usize>,
  _source_vertex: usize,
  _graph: &'a Graph,
}

impl<'a> DFS<'a> {
  /// Creates a populated, ready to be queried, depth-first-search struct.
  /// @note Panics when `source_vertex` is greater than given graph's current `#.vert_count()` value.
  pub fn new(g: &'a Graph, source_vertex: usize) -> Self {
    let mut out = DFS {
      _marked: vec![false; g.vert_count()],
      _edge_to: vec![0; g.vert_count()],
      _source_vertex: source_vertex,
      _graph: g,
      _count: 0,
    };
    if let Err(err) = g.validate_vertex(source_vertex) {
      panic!("{}", err);
    }
    out.dfs(out._source_vertex);
    out
  }

  /// Runs 'depth-first-search' algorithm on contained graph and stores results on `self`.
  pub fn dfs(&mut self, v: usize) -> &Self {
    if let Err(err) = self._graph.validate_vertex(v) {
      panic!("{}", err);
    }
    self._count += 1;
    self._marked[v] = true;
    if let Ok(adj) = self._graph.adj(v) {
      for w in adj {
        if !self._marked[*w] {
          self._edge_to[*w] = v;
          self.dfs(*w);
        }
      }
    }
    self
  }

  pub fn count(&self) -> usize {
    self._count
  }

  pub fn marked(&self, i: usize) -> bool {
    if i >= self._marked.len() {
      // @todo shouldn't 'panic!' here
      panic!("{:} is out of range", i);
    }
    self._marked[i]
  }

  pub fn graph(&self) -> &'a Graph {
    self._graph
  }

  pub fn source_vertex(&self) -> usize {
    self._source_vertex
  }

  pub fn has_path_to(&self, i: usize) -> bool {
    self.marked(i)
  }

  pub fn path_to(&self, v: usize) -> Option<Vec<usize>> {
    if self.has_path_to(v) {
      return None;
    }
    let s = self._source_vertex;
    let mut path: Vec<usize> = vec![];
    let mut x = v;

    loop {
      if x == s {
        break;
      }
      x = self._edge_to[x];
      path.push(x);
    }

    path.push(s);
    Some(path)
  }
}

#[cfg(test)]
mod test {
  use crate::graph::symbol_graph::SymbolGraph;
  use std::fs::File;
  use std::io::BufReader;
  use crate::graph::GenericSymbol;

  use super::*;

  #[test]
  pub fn test_dfs() -> std::io::Result<()> {
    // Get representation of graph
    let f = File::open("../test-fixtures/acl_roles_symbol_graph.txt")?;

    // Graph vertex, and edge, sizes
    let mut reader = BufReader::new(f);
    let sg: SymbolGraph<GenericSymbol> = (&mut reader).try_into().unwrap();
    let _dfs = DFS::new(&sg.graph(), 3);
    // println!("{:?}", &sg);
    // println!("{:?}", &dfs.has_path_to(3));

    Ok(())
  }
}

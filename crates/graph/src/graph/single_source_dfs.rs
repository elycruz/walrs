use std::fmt::Debug;

use crate::graph::Graph;

/// Depth-first search to find paths from a single source vertex to all reachable vertices
/// in an undirected graph.
///
/// # Examples
///
/// ```
/// use walrs_graph::{Graph, DFS};
///
/// let mut g = Graph::new(6);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
/// g.add_edge(3, 4).unwrap();
/// // Vertex 5 is isolated
///
/// let dfs = DFS::new(&g, 0);
///
/// // Check reachability
/// assert!(dfs.marked(4)); // Reachable from 0
/// assert!(!dfs.marked(5)); // Not reachable from 0
///
/// // Get path from source to vertex
/// if let Some(path) = dfs.path_to(4) {
///     println!("Path from 0 to 4: {:?}", path);
/// }
/// ```
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
  fn dfs(&mut self, v: usize) -> &Self {
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

  /// Returns the number of vertices reachable from the source vertex.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_graph::{Graph, DFS};
  ///
  /// let mut g = Graph::new(5);
  /// g.add_edge(0, 1).unwrap();
  /// g.add_edge(1, 2).unwrap();
  /// // Vertices 3 and 4 are isolated
  ///
  /// let dfs = DFS::new(&g, 0);
  /// assert_eq!(dfs.count(), 3); // 0, 1, 2 are reachable
  /// ```
  pub fn count(&self) -> usize {
    self._count
  }

  /// Returns `true` if vertex `i` is reachable from the source vertex,
  /// or `false` if `i` is out of range or not reachable.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_graph::{Graph, DFS};
  ///
  /// let mut g = Graph::new(3);
  /// g.add_edge(0, 1).unwrap();
  ///
  /// let dfs = DFS::new(&g, 0);
  /// assert!(dfs.marked(0));
  /// assert!(dfs.marked(1));
  /// assert!(!dfs.marked(2));
  /// assert!(!dfs.marked(99)); // Out-of-range returns false
  /// ```
  pub fn marked(&self, i: usize) -> bool {
    if i >= self._marked.len() {
      return false;
    }
    self._marked[i]
  }

  pub fn graph(&self) -> &'a Graph {
    self._graph
  }

  pub fn source_vertex(&self) -> usize {
    self._source_vertex
  }

  /// Returns `true` if there is a path from the source vertex to vertex `i`.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_graph::{Graph, DFS};
  ///
  /// let mut g = Graph::new(4);
  /// g.add_edge(0, 1).unwrap();
  /// g.add_edge(1, 2).unwrap();
  ///
  /// let dfs = DFS::new(&g, 0);
  /// assert!(dfs.has_path_to(2));
  /// assert!(!dfs.has_path_to(3));
  /// ```
  pub fn has_path_to(&self, i: usize) -> bool {
    self.marked(i)
  }

  /// Returns a path from the source vertex to vertex `v`, or `None` if no path exists.
  ///
  /// The path is returned in reverse order (from `v` back to source).
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_graph::{Graph, DFS};
  ///
  /// let mut g = Graph::new(4);
  /// g.add_edge(0, 1).unwrap();
  /// g.add_edge(1, 2).unwrap();
  /// g.add_edge(2, 3).unwrap();
  ///
  /// let dfs = DFS::new(&g, 0);
  ///
  /// if let Some(path) = dfs.path_to(3) {
  ///     // Path is returned in reverse order
  ///     assert_eq!(path.last(), Some(&0));
  /// }
  ///
  /// // No path to isolated vertex
  /// let mut g2 = Graph::new(5);
  /// g2.add_edge(0, 1).unwrap();
  /// let dfs2 = DFS::new(&g2, 0);
  /// assert_eq!(dfs2.path_to(4), None);
  /// ```
  pub fn path_to(&self, v: usize) -> Option<Vec<usize>> {
    if !self.has_path_to(v) {
      return None;
    }
    let s = self._source_vertex;
    let mut path: Vec<usize> = vec![v];
    let mut x = v;

    while x != s {
      x = self._edge_to[x];
      path.push(x);
    }

    Some(path)
  }
}

#[cfg(test)]
mod test {
  use crate::graph::GenericSymbol;
  use crate::graph::symbol_graph::SymbolGraph;
  use std::fs::File;
  use std::io::BufReader;

  use super::*;

  #[test]
  pub fn test_dfs() -> std::io::Result<()> {
    // Get representation of graph
    let f = File::open("./test-fixtures/acl_roles_symbol_graph.txt")?;

    // Graph vertex, and edge, sizes
    let mut reader = BufReader::new(f);
    let sg: SymbolGraph<GenericSymbol> = (&mut reader).try_into().unwrap();
    let _dfs = DFS::new(&sg.graph(), 3);
    println!("{:?}", &sg);
    println!("{:?}", &_dfs.has_path_to(3));

    Ok(())
  }

  #[test]
  pub fn test_dfs_disconnected_graph() {
    // Two components: {0-1-2} and {3-4}
    let mut g = Graph::new(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(3, 4).unwrap();

    let dfs = DFS::new(&g, 0);
    assert!(dfs.marked(0));
    assert!(dfs.marked(1));
    assert!(dfs.marked(2));
    assert!(!dfs.marked(3), "Vertex 3 is in a different component");
    assert!(!dfs.marked(4), "Vertex 4 is in a different component");
    assert_eq!(dfs.count(), 3);
  }

  #[test]
  pub fn test_dfs_single_vertex() {
    let g = Graph::new(1);
    let dfs = DFS::new(&g, 0);
    assert!(dfs.marked(0));
    assert_eq!(dfs.count(), 1);
    assert_eq!(dfs.source_vertex(), 0);
  }

  #[test]
  pub fn test_dfs_isolated_vertices() {
    let g = Graph::new(5);
    // No edges — all vertices isolated
    let dfs = DFS::new(&g, 2);
    assert!(dfs.marked(2));
    assert!(!dfs.marked(0));
    assert!(!dfs.marked(1));
    assert!(!dfs.marked(3));
    assert!(!dfs.marked(4));
    assert_eq!(dfs.count(), 1);
  }

  #[test]
  pub fn test_dfs_marked_out_of_range() {
    let g = Graph::new(3);
    let dfs = DFS::new(&g, 0);
    // Out-of-range should return false, not panic
    assert!(!dfs.marked(3));
    assert!(!dfs.marked(99));
    assert!(!dfs.marked(usize::MAX));
  }

  #[test]
  pub fn test_dfs_path_to_direct() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();

    let dfs = DFS::new(&g, 0);

    // Path to source is just the source
    let path_to_source = dfs.path_to(0).unwrap();
    assert_eq!(path_to_source, vec![0]);

    // Path to 3 should end at source 0
    let path = dfs.path_to(3).unwrap();
    assert_eq!(*path.last().unwrap(), 0);
    assert_eq!(*path.first().unwrap(), 3);
  }

  #[test]
  pub fn test_dfs_path_to_unreachable() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1).unwrap();
    // Vertices 2 and 3 are isolated

    let dfs = DFS::new(&g, 0);
    assert_eq!(dfs.path_to(2), None);
    assert_eq!(dfs.path_to(3), None);
  }

  #[test]
  pub fn test_dfs_has_path_to() {
    let mut g = Graph::new(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();

    let dfs = DFS::new(&g, 0);
    assert!(dfs.has_path_to(0));
    assert!(dfs.has_path_to(1));
    assert!(dfs.has_path_to(2));
    assert!(!dfs.has_path_to(3));
    assert!(!dfs.has_path_to(4));
    // Out-of-range
    assert!(!dfs.has_path_to(99));
  }

  #[test]
  pub fn test_dfs_graph_accessor() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1).unwrap();
    let dfs = DFS::new(&g, 0);
    assert_eq!(dfs.graph().vert_count(), 3);
  }

  #[test]
  pub fn test_dfs_count() {
    let mut g = Graph::new(5);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    // Vertices 3 and 4 are isolated

    let dfs = DFS::new(&g, 0);
    assert_eq!(dfs.count(), 3); // 0, 1, 2 are reachable
  }
}

use crate::Digraph;
use crate::dfs::{DigraphDFSShape, vertex_marked};

pub struct DigraphDipathsDFS {
  _marked: Vec<bool>,
  _count: usize,
  _edge_to: Vec<Option<usize>>,
  _source_vertex: usize,
}

impl DigraphDFSShape for DigraphDipathsDFS {
  /// Returns a `Result` indicating whether  a path from 'source vertex' to 'i' exists.
  fn marked(&self, i: usize) -> Result<bool, String> {
    vertex_marked(&self._marked, i)
  }
}

impl DigraphDipathsDFS {
  pub fn new(g: &Digraph, source_vertex: usize) -> Result<Self, String> {
    let mut out = DigraphDipathsDFS {
      _marked: vec![false; g.vert_count()],
      _edge_to: vec![None; g.vert_count()],
      _source_vertex: source_vertex,
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
        self._edge_to[*w] = Some(v);
        self.dfs(g, *w)?;
      }
    }
    Ok(self)
  }

  /// Result indicating whether there is a path from `source_vertex` to vertex `i`.
  pub fn has_path_to(&self, i: usize) -> Result<bool, String> {
    self.marked(i)
  }

  /// Returns an `Option` indicating path to vertex `v`;  Returns `None` if `v` is equal to
  /// `source_path` (initially passed to struct)..
  /// @note - Panics if `v` is out of bounds.
  pub fn path_to(&self, v: usize) -> Option<Vec<usize>> {
    if !self.has_path_to(v).unwrap() {
      return None;
    }
    let s = self._source_vertex;
    let mut path: Vec<usize> = vec![];
    let mut x = v;
    loop {
      if x == s {
        break;
      }
      path.push(x);
      x = match self._edge_to[x] {
        Some(index) => index,
        _ => s,
      }
    }
    path.push(s);
    Some(path)
  }

  /// Returns the number of vertices reachable from `source_vertex`.
  pub fn count(&self) -> usize {
    self._count
  }
}

#[cfg(test)]
mod test {
  use crate::math::triangular_num;
  use crate::symbol_digraph::DisymGraph;

  use super::*;

  #[test]
  pub fn test_dipaths_dfs_with_symbol_dag() -> Result<(), Box<dyn std::error::Error>> {
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
      sym_graph.add_edge(v, &edges).expect("No panic");
      sym_graph_2.add_edge(v, &edges_2).expect("No panic");
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
      let dfs_rslt = DigraphDipathsDFS::new(sym_graph.graph(), i)?;
      let dfs_rslt_2 = DigraphDipathsDFS::new(sym_graph_2.graph(), i)?;

      for j in i + 1..v_len {
        // println!("j: {}", i);
        // `#.marked()`
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

        // `#.has_path_to()`
        assert_eq!(
          dfs_rslt.has_path_to(j)?,
          dfs_rslt.marked(j)?,
          "`#.has_path_to({})` should equal `#.marked({})` (1)",
          j,
          j
        );
        assert_eq!(
          dfs_rslt_2.has_path_to(j)?,
          dfs_rslt_2.marked(j)?,
          "`#.has_path_to({})` should equal `#.marked({})` (1)",
          j,
          j
        );

        // `#.path_to()`
        // println!("{:?}", &dfs_rslt.path_to(j).unwrap());
        assert_eq!(
          dfs_rslt.path_to(j).is_some(),
          true,
          "`#.path_to({})` should return `true`",
          j
        );
        assert_eq!(
          dfs_rslt.path_to(j).unwrap().sort_unstable(),
          (i..j + 1).collect::<Vec<usize>>().sort_unstable(),
          "found path doesn't match expected"
        );

        // println!("dfs_rslt.path_to({}) == {:?}", i, dfs_rslt.path_to(i));
        // `#.path_to(dfs_rslt.source_path) == [dfs_rslt.source_path]`
        assert_eq!(
          dfs_rslt.path_to(i).unwrap(),
          vec![i],
          "`#.path_to({})`, for source_vert `{}`, should return `[{}]`",
          i,
          i,
          i
        );
        assert_eq!(
          dfs_rslt_2.path_to(i).unwrap(),
          vec![i],
          "`#.path_to({})`, for source_vert `{}`, should return `[{}]`",
          i,
          i,
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

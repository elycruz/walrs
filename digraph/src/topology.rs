use crate::{invalid_vertex_msg, DepthFirstOrder, Digraph, DirectedCycle};

/// The `Topological` struct represents a data type for determining a
/// topological order of a *directed acyclic graph* (DAG).
/// A digraph has a topological order if and only if it is a DAG.
/// The `has_order` method determines whether the digraph has a topological order,
/// and if so, the `order` method returns one.
///
/// This implementation uses depth-first search.
/// The constructor takes Θ(V + E) time in the worst case, where V is the
/// number of vertices and E is the number of edges.
/// Each instance method takes Θ(1) time.
/// It uses Θ(V) extra space (not including the digraph).
///
/// Based on the Topological implementation from Algorithms, 4th Edition
/// by Robert Sedgewick and Kevin Wayne.
///
/// # Examples
///
/// ```rust
/// use walrs_digraph::Digraph;
/// use walrs_digraph::Topology;
///
/// // Create a simple DAG
/// let mut g = Digraph::new(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let topo = Topology::new(&g);
/// assert!(topo.has_order());
///
/// // Get the topological order
/// if let Some(order) = topo.order() {
///     // Vertex 0 must come before 1 and 2
///     // Vertices 1 and 2 must come before 3
///     let order_vec: Vec<usize> = order.iter().cloned().collect();
///     assert_eq!(order_vec.len(), 4);
/// }
/// ```
pub struct Topology {
    /// topological order (None if digraph has a cycle)
    _order: Option<Vec<usize>>,
    /// rank[v] = rank of vertex v in order (None if digraph has a cycle)
    _rank: Option<Vec<usize>>,
}

impl Topology {
    /// Determines whether the digraph has a topological order and, if so,
    /// finds such a topological order.
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::Topology;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let topo = Topology::new(&g);
    /// assert!(topo.has_order());
    /// ```
    pub fn new(g: &Digraph) -> Self {
        let finder = DirectedCycle::new(g);

        if !finder.has_cycle() {
            let dfs = DepthFirstOrder::new(g);
            let order = dfs.reverse_post();

            // Build rank array
            let mut rank = vec![0; g.vert_count()];
            for (i, &v) in order.iter().enumerate() {
                rank[v] = i;
            }

            Topology {
                _order: Some(order),
                _rank: Some(rank),
            }
        } else {
            Topology {
                _order: None,
                _rank: None,
            }
        }
    }

    /// Returns a topological order if the digraph has a topological order,
    /// and `None` otherwise.
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::Topology;
    ///
    /// // DAG
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let topo = Topology::new(&g);
    /// assert!(topo.order().is_some());
    ///
    /// // Graph with cycle
    /// let mut cyclic = Digraph::new(3);
    /// cyclic.add_edge(0, 1).unwrap();
    /// cyclic.add_edge(1, 2).unwrap();
    /// cyclic.add_edge(2, 0).unwrap();
    ///
    /// let topo_cyclic = Topology::new(&cyclic);
    /// assert!(topo_cyclic.order().is_none());
    /// ```
    pub fn order(&self) -> Option<&[usize]> {
        self._order.as_deref()
    }

    /// Returns bool which signals whether the digraph has a topological order or not.
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::Topology;
    ///
    /// // DAG
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let topo = Topology::new(&g);
    /// assert!(topo.has_order());
    ///
    /// // Graph with cycle
    /// let mut cyclic = Digraph::new(3);
    /// cyclic.add_edge(0, 1).unwrap();
    /// cyclic.add_edge(1, 2).unwrap();
    /// cyclic.add_edge(2, 0).unwrap();
    ///
    /// let topo_cyclic = Topology::new(&cyclic);
    /// assert!(!topo_cyclic.has_order());
    /// ```
    pub fn has_order(&self) -> bool {
        self._order.is_some()
    }

    /// Returns whether the digraph is a DAG (directed acyclic graph).
    ///
    /// This is equivalent to `has_order()`.
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::Topology;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let topo = Topology::new(&g);
    /// assert!(topo.is_dag());
    /// ```
    pub fn is_dag(&self) -> bool {
        self.has_order()
    }

    /// Returns the rank of vertex `v` in the topological order.
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::Topology;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let topo = Topology::new(&g);
    ///
    /// // For linear chain, rank should reflect topological order
    /// assert_eq!(topo.rank(0), Ok(Some(0)));
    /// assert_eq!(topo.rank(1), Ok(Some(1)));
    /// assert_eq!(topo.rank(2), Ok(Some(2)));
    ///
    /// // Invalid vertex returns error
    /// assert!(topo.rank(5).is_err());
    /// ```
    pub fn rank(&self, v: usize) -> Result<Option<usize>, String> {
        match &self._rank {
            Some(rank) => {
                if v >= rank.len() {
                    Err(invalid_vertex_msg(v, rank.len()))
                } else {
                    Ok(Some(rank[v]))
                }
            }
            None => Ok(None),
        }
    }

    /// Returns an iterator over the vertices in topological order.
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::Topology;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let topo = Topology::new(&g);
    ///
    /// if let Some(order) = topo.order() {
    ///     for v in order {
    ///         println!("{}", v);
    ///     }
    /// }
    /// ```
    pub fn order_iter(&self) -> Option<impl Iterator<Item = &usize>> {
        self._order.as_ref().map(|order| order.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates the tinyDAG digraph from the Sedgewick/Wayne test files.
    fn create_tiny_dag() -> Digraph {
        let mut g = Digraph::new(13);
        g.add_edge(2, 3).unwrap();
        g.add_edge(0, 6).unwrap();
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 0).unwrap();
        g.add_edge(11, 12).unwrap();
        g.add_edge(9, 12).unwrap();
        g.add_edge(9, 10).unwrap();
        g.add_edge(9, 11).unwrap();
        g.add_edge(3, 5).unwrap();
        g.add_edge(8, 7).unwrap();
        g.add_edge(5, 4).unwrap();
        g.add_edge(0, 5).unwrap();
        g.add_edge(6, 4).unwrap();
        g.add_edge(6, 9).unwrap();
        g.add_edge(7, 6).unwrap();
        g
    }

    #[test]
    fn test_topological_dag() {
        let g = create_tiny_dag();
        let topo = Topology::new(&g);

        assert!(topo.has_order());
        assert!(topo.is_dag());
        assert!(topo.order().is_some());

        let order = topo.order().unwrap();
        assert_eq!(order.len(), 13);
    }

    #[test]
    fn test_topological_cyclic() {
        let mut g = Digraph::new(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let topo = Topology::new(&g);

        assert!(!topo.has_order());
        assert!(!topo.is_dag());
        assert!(topo.order().is_none());
    }

    #[test]
    fn test_topological_order_respects_edges() {
        // Create a simple DAG: 0 -> 1 -> 2, 0 -> 2
        let mut g = Digraph::new(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(0, 2).unwrap();

        let topo = Topology::new(&g);
        assert!(topo.has_order());

        let order = topo.order().unwrap();

        // Find positions in order
        let pos_0 = order.iter().position(|&v| v == 0).unwrap();
        let pos_1 = order.iter().position(|&v| v == 1).unwrap();
        let pos_2 = order.iter().position(|&v| v == 2).unwrap();

        // 0 must come before 1 and 2
        assert!(pos_0 < pos_1);
        assert!(pos_0 < pos_2);
        // 1 must come before 2
        assert!(pos_1 < pos_2);
    }

    #[test]
    fn test_rank() {
        let mut g = Digraph::new(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let topo = Topology::new(&g);

        // For linear chain, topological order is 0, 1, 2
        assert_eq!(topo.rank(0), Ok(Some(0)));
        assert_eq!(topo.rank(1), Ok(Some(1)));
        assert_eq!(topo.rank(2), Ok(Some(2)));
    }

    #[test]
    fn test_rank_cyclic() {
        let mut g = Digraph::new(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let topo = Topology::new(&g);

        // For cyclic graph, rank returns Ok(None)
        assert_eq!(topo.rank(0), Ok(None));
        assert_eq!(topo.rank(1), Ok(None));
        assert_eq!(topo.rank(2), Ok(None));
    }

    #[test]
    fn test_rank_invalid_vertex() {
        let mut g = Digraph::new(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let topo = Topology::new(&g);

        // Invalid vertex should return error
        assert!(topo.rank(5).is_err());
    }

    #[test]
    fn test_empty_graph() {
        let g = Digraph::new(0);
        let topo = Topology::new(&g);

        assert!(topo.has_order());
        assert!(topo.is_dag());
        assert!(topo.order().unwrap().is_empty());
    }

    #[test]
    fn test_single_vertex() {
        let g = Digraph::new(1);
        let topo = Topology::new(&g);

        assert!(topo.has_order());
        assert_eq!(topo.order().unwrap(), &[0]);
        assert_eq!(topo.rank(0), Ok(Some(0)));
    }

    #[test]
    fn test_disconnected_dag() {
        // Two disconnected components: 0->1 and 2->3
        let mut g = Digraph::new(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();

        let topo = Topology::new(&g);

        assert!(topo.has_order());
        let order = topo.order().unwrap();
        assert_eq!(order.len(), 4);

        // Check that edges are respected
        let pos_0 = order.iter().position(|&v| v == 0).unwrap();
        let pos_1 = order.iter().position(|&v| v == 1).unwrap();
        let pos_2 = order.iter().position(|&v| v == 2).unwrap();
        let pos_3 = order.iter().position(|&v| v == 3).unwrap();

        assert!(pos_0 < pos_1);
        assert!(pos_2 < pos_3);
    }

    #[test]
    fn test_order_iter() {
        let mut g = Digraph::new(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let topo = Topology::new(&g);

        let order_vec: Vec<usize> = topo.order_iter().unwrap().cloned().collect();
        assert_eq!(order_vec, topo.order().unwrap().to_vec());
    }

    #[test]
    fn test_order_iter_cyclic() {
        let mut g = Digraph::new(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();
        g.add_edge(2, 0).unwrap();

        let topo = Topology::new(&g);

        assert!(topo.order_iter().is_none());
    }

    #[test]
    fn test_rank_consistency_with_order() {
        let g = create_tiny_dag();
        let topo = Topology::new(&g);

        let order = topo.order().unwrap();

        // Verify that rank[v] gives the correct position in order
        for (i, &v) in order.iter().enumerate() {
            assert_eq!(topo.rank(v), Ok(Some(i)));
        }
    }
}

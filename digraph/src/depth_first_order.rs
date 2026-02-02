use crate::Digraph;

/// Returns panic message for invalid vertices in `DepthFirstOrder`.
pub fn depth_first_order_invalid_vertex_msg(v: usize, max_v: usize) -> String {
    format!(
        "vertex {} is not between 0 and {}",
        v,
        if max_v > 0 { max_v - 1 } else { 0 }
    )
}

/// The `DepthFirstOrder` struct represents a data type for determining
/// depth-first search ordering of the vertices in a digraph, including
/// preorder, postorder, and reverse postorder.
///
/// This implementation uses depth-first search.
/// The constructor takes Θ(V + E) time, where V is the number of vertices
/// and E is the number of edges.
/// Each instance method takes Θ(1) time.
/// It uses Θ(V) extra space (not including the digraph).
///
/// Based on the DepthFirstOrder implementation from Algorithms, 4th Edition
/// by Robert Sedgewick and Kevin Wayne.
///
/// # Examples
///
/// ```rust
/// use walrs_digraph::Digraph;
/// use walrs_digraph::DepthFirstOrder;
///
/// // Create a simple DAG
/// let mut g = Digraph::new(4);
/// g.add_edge(0, 1).unwrap();
/// g.add_edge(0, 2).unwrap();
/// g.add_edge(1, 3).unwrap();
/// g.add_edge(2, 3).unwrap();
///
/// let dfo = DepthFirstOrder::new(&g);
///
/// // Get preorder and postorder traversals
/// let preorder: Vec<usize> = dfo.pre().iter().cloned().collect();
/// let postorder: Vec<usize> = dfo.post().iter().cloned().collect();
///
/// assert_eq!(preorder.len(), 4);
/// assert_eq!(postorder.len(), 4);
/// ```
pub struct DepthFirstOrder {
    /// marked[v] = has v been marked in dfs?
    _marked: Vec<bool>,
    /// pre[v] = preorder number of v
    _pre: Vec<usize>,
    /// post[v] = postorder number of v
    _post: Vec<usize>,
    /// vertices in preorder
    _preorder: Vec<usize>,
    /// vertices in postorder
    _postorder: Vec<usize>,
    /// counter for preorder numbering
    _pre_counter: usize,
    /// counter for postorder numbering
    _post_counter: usize,
}

impl DepthFirstOrder {
    /// Determines a depth-first order for the digraph.
    ///
    /// # Arguments
    ///
    /// * `g` - The digraph
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::DepthFirstOrder;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let dfo = DepthFirstOrder::new(&g);
    /// assert!(dfo.pre_order(0).is_ok());
    /// ```
    pub fn new(g: &Digraph) -> Self {
        let vert_count = g.vert_count();
        let mut out = DepthFirstOrder {
            _marked: vec![false; vert_count],
            _pre: vec![0; vert_count],
            _post: vec![0; vert_count],
            _preorder: Vec::with_capacity(vert_count),
            _postorder: Vec::with_capacity(vert_count),
            _pre_counter: 0,
            _post_counter: 0,
        };

        // Run DFS from each unmarked vertex
        for v in 0..vert_count {
            if !out._marked[v] {
                out.dfs(g, v);
            }
        }

        debug_assert!(out.check());

        out
    }

    /// Runs DFS in the digraph from vertex v and computes preorder/postorder.
    fn dfs(&mut self, g: &Digraph, v: usize) {
        self._marked[v] = true;
        self._pre[v] = self._pre_counter;
        self._pre_counter += 1;
        self._preorder.push(v);

        if let Ok(adj) = g.adj(v) {
            for &w in adj {
                if !self._marked[w] {
                    self.dfs(g, w);
                }
            }
        }

        self._postorder.push(v);
        self._post[v] = self._post_counter;
        self._post_counter += 1;
    }

    /// Returns the preorder number of vertex `v`.
    ///
    /// # Arguments
    ///
    /// * `v` - The vertex
    ///
    /// # Returns
    ///
    /// `Ok(usize)` - The preorder number of vertex `v`
    /// `Err(String)` - Error message if `v` is not a valid vertex
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::DepthFirstOrder;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let dfo = DepthFirstOrder::new(&g);
    /// assert_eq!(dfo.pre_order(0), Ok(0));
    /// ```
    pub fn pre_order(&self, v: usize) -> Result<usize, String> {
        self.validate_vertex(v)?;
        Ok(self._pre[v])
    }

    /// Returns the postorder number of vertex `v`.
    ///
    /// # Arguments
    ///
    /// * `v` - The vertex
    ///
    /// # Returns
    ///
    /// `Ok(usize)` - The postorder number of vertex `v`
    /// `Err(String)` - Error message if `v` is not a valid vertex
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::DepthFirstOrder;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let dfo = DepthFirstOrder::new(&g);
    /// assert!(dfo.post_order(2).is_ok());
    /// ```
    pub fn post_order(&self, v: usize) -> Result<usize, String> {
        self.validate_vertex(v)?;
        Ok(self._post[v])
    }

    /// Returns the vertices in postorder.
    ///
    /// # Returns
    ///
    /// A slice of vertices in postorder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::DepthFirstOrder;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let dfo = DepthFirstOrder::new(&g);
    /// let postorder = dfo.post();
    /// assert_eq!(postorder.len(), 3);
    /// ```
    pub fn post(&self) -> &[usize] {
        &self._postorder
    }

    /// Returns the vertices in preorder.
    ///
    /// # Returns
    ///
    /// A slice of vertices in preorder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::DepthFirstOrder;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let dfo = DepthFirstOrder::new(&g);
    /// let preorder = dfo.pre();
    /// assert_eq!(preorder.len(), 3);
    /// ```
    pub fn pre(&self) -> &[usize] {
        &self._preorder
    }

    /// Returns the vertices in reverse postorder.
    ///
    /// Reverse postorder is commonly used for topological sorting of DAGs.
    ///
    /// # Returns
    ///
    /// A `Vec<usize>` containing the vertices in reverse postorder
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::DepthFirstOrder;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let dfo = DepthFirstOrder::new(&g);
    /// let reverse_post = dfo.reverse_post();
    /// assert_eq!(reverse_post.len(), 3);
    /// // For a linear chain 0->1->2, reverse postorder gives topological order: [0, 1, 2]
    /// assert_eq!(reverse_post, vec![0, 1, 2]);
    /// ```
    pub fn reverse_post(&self) -> Vec<usize> {
        self._postorder.iter().rev().cloned().collect()
    }

    /// Returns an iterator over the vertices in reverse postorder.
    ///
    /// This is more memory-efficient than `reverse_post()` when you only need
    /// to iterate once.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walrs_digraph::Digraph;
    /// use walrs_digraph::DepthFirstOrder;
    ///
    /// let mut g = Digraph::new(3);
    /// g.add_edge(0, 1).unwrap();
    /// g.add_edge(1, 2).unwrap();
    ///
    /// let dfo = DepthFirstOrder::new(&g);
    /// for v in dfo.reverse_post_iter() {
    ///     println!("{}", v);
    /// }
    /// ```
    pub fn reverse_post_iter(&self) -> impl Iterator<Item = &usize> {
        self._postorder.iter().rev()
    }

    /// Validates that vertex v is within bounds.
    fn validate_vertex(&self, v: usize) -> Result<(), String> {
        let n = self._marked.len();
        if v >= n {
            return Err(depth_first_order_invalid_vertex_msg(v, n));
        }
        Ok(())
    }

    /// Check that pre() and post() are consistent with pre_order(v) and post_order(v).
    #[allow(dead_code)]
    fn check(&self) -> bool {
        // Check that post_order(v) is consistent with post()
        for (r, &v) in self._postorder.iter().enumerate() {
            if self._post[v] != r {
                eprintln!("post_order(v) and post() inconsistent");
                return false;
            }
        }

        // Check that pre_order(v) is consistent with pre()
        for (r, &v) in self._preorder.iter().enumerate() {
            if self._pre[v] != r {
                eprintln!("pre_order(v) and pre() inconsistent");
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates the tinyDAG digraph from the Sedgewick/Wayne test files.
    /// The graph has 13 vertices and the following edges:
    ///  2->3, 0->6, 0->1, 2->0, 11->12, 9->12, 9->10, 9->11, 3->5, 8->7, 5->4,
    ///  0->5, 6->4, 6->9, 7->6
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
    fn test_depth_first_order_new() {
        let g = create_tiny_dag();
        let dfo = DepthFirstOrder::new(&g);

        // All vertices should be visited
        assert_eq!(dfo.pre().len(), 13);
        assert_eq!(dfo.post().len(), 13);
    }

    #[test]
    fn test_pre_order() {
        let g = create_tiny_dag();
        let dfo = DepthFirstOrder::new(&g);

        // First vertex visited should have pre_order 0
        let first_vertex = dfo.pre()[0];
        assert_eq!(dfo.pre_order(first_vertex), Ok(0));

        // Invalid vertex should return error
        assert!(dfo.pre_order(13).is_err());
    }

    #[test]
    fn test_post_order() {
        let g = create_tiny_dag();
        let dfo = DepthFirstOrder::new(&g);

        // First vertex in postorder should have post_order 0
        let first_vertex = dfo.post()[0];
        assert_eq!(dfo.post_order(first_vertex), Ok(0));

        // Invalid vertex should return error
        assert!(dfo.post_order(13).is_err());
    }

    #[test]
    fn test_reverse_post() {
        let g = create_tiny_dag();
        let dfo = DepthFirstOrder::new(&g);

        let reverse_post = dfo.reverse_post();

        // Should have all vertices
        assert_eq!(reverse_post.len(), 13);

        // reverse_post should be the reverse of post
        let post: Vec<usize> = dfo.post().iter().cloned().collect();
        let mut expected: Vec<usize> = post;
        expected.reverse();
        assert_eq!(reverse_post, expected);
    }

    #[test]
    fn test_reverse_post_iter() {
        let g = create_tiny_dag();
        let dfo = DepthFirstOrder::new(&g);

        let reverse_post_vec: Vec<usize> = dfo.reverse_post_iter().cloned().collect();
        let reverse_post = dfo.reverse_post();

        assert_eq!(reverse_post_vec, reverse_post);
    }

    #[test]
    fn test_simple_linear_chain() {
        // Simple linear chain: 0 -> 1 -> 2
        let mut g = Digraph::new(3);
        g.add_edge(0, 1).unwrap();
        g.add_edge(1, 2).unwrap();

        let dfo = DepthFirstOrder::new(&g);

        // Preorder: [0, 1, 2]
        assert_eq!(dfo.pre(), &[0, 1, 2]);

        // Postorder: [2, 1, 0]
        assert_eq!(dfo.post(), &[2, 1, 0]);

        // Reverse postorder (topological order): [0, 1, 2]
        assert_eq!(dfo.reverse_post(), vec![0, 1, 2]);
    }

    #[test]
    fn test_empty_graph() {
        let g = Digraph::new(0);
        let dfo = DepthFirstOrder::new(&g);

        assert_eq!(dfo.pre().len(), 0);
        assert_eq!(dfo.post().len(), 0);
        assert_eq!(dfo.reverse_post().len(), 0);
    }

    #[test]
    fn test_single_vertex() {
        let g = Digraph::new(1);
        let dfo = DepthFirstOrder::new(&g);

        assert_eq!(dfo.pre(), &[0]);
        assert_eq!(dfo.post(), &[0]);
        assert_eq!(dfo.pre_order(0), Ok(0));
        assert_eq!(dfo.post_order(0), Ok(0));
    }

    #[test]
    fn test_disconnected_components() {
        // Two disconnected components: 0->1 and 2->3
        let mut g = Digraph::new(4);
        g.add_edge(0, 1).unwrap();
        g.add_edge(2, 3).unwrap();

        let dfo = DepthFirstOrder::new(&g);

        // All vertices should be visited
        assert_eq!(dfo.pre().len(), 4);
        assert_eq!(dfo.post().len(), 4);
    }

    #[test]
    fn test_check_consistency() {
        let g = create_tiny_dag();
        let dfo = DepthFirstOrder::new(&g);

        // The check method should return true for consistent pre/post orders
        assert!(dfo.check());
    }

    #[test]
    fn test_validate_vertex_error_message() {
        let g = Digraph::new(5);
        let dfo = DepthFirstOrder::new(&g);

        let result = dfo.pre_order(10);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "vertex 10 is not between 0 and 4"
        );
    }
}

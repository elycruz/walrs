# Digraph

Reference implementation: https://algs4.cs.princeton.edu/42digraph/

## Implementation:

- [x] `Digraph`
  - [x] `add_edge()`
  - [x] `add_vertex()`
  - [x] `adj()` - Adjacency list getter - Returns `Result<...>`.
  - [x] ~~`digest_lines()`~~ - Handled by trait now (see `TryFrom` usages).
  - [x] `edge_count()`
  - [x] `indegree()`
  - [x] ~~`indegree_safe()` - Returns `Result`.~~
  - [x] `new()`
  - [x] `outdegree()`
  - [x] ~~`outdegree_safe()` - Returns `Result`.~~
  - [x] `reverse()`
  - [x] `validate_vertex()` - Should be settable.
  - [x] ~~`validate_vertex_safe()` - Returns `Result`.~~
  - [x] `vert_count()`
  - [x] Simplify API - we shouldn't have '*_safe' variant methods - If methods can 'panic' they should just return `Result<...>`;  E.g., instead of `#.validate_vertex`, and `#.validate_vertex_safe` we need only `#.validate_vertex()` - Better overall.
  - [x] `TryFrom<&mut BuffReader<R>>`
  - [x] `TryFrom<BuffReader<R>>`
  - [x] `TryFrom<&File>`
  - [x] `TryFrom<File>`

- [x] `Symbol` trait (`id() -> &str`; implemented for `String`, shared with `walrs_graph`)

- [x] `DigraphDFSShape` trait (`marked()`)

- [x] `DisymGraph<T: Symbol = String>` - "Directed Symbol Graph"
  - [x] `add_edge()` (requires `T: From<String>`)
  - [x] `add_symbol()`
  - [x] `add_vertex()` (requires `T: From<String>`)
  - [x] `adj()`
  - [x] `adj_indices()`
  - [x] `adj_symbols()`
  - [x] `connect()`
  - [x] `contains()` - Same as `has_vertex()`.
  - [x] ~~`digest_lines()`~~ - Handled by trait now (see `TryFrom` usages).
  - [x] `edge_count()`
  - [x] `graph()`
  - [x] `has_vertex()`
  - [x] `indegree()`
  - [x] `index()`
  - [x] `indices()`
  - [x] `name()`
  - [x] `name_as_ref()`
  - [x] `names()`
  - [x] `new()` - `String` payload only; other payloads use `Default`.
  - [x] `outdegree()`
  - [x] `reverse()`
  - [x] `symbol()`
  - [x] `symbols()`
  - [x] `validate_vertex()` - Should be settable.
  - [x] `vert_count()`
  - [x] `Default`
  - [x] `TryFrom<&DisymGraphData<T>>` / `TryFrom<DisymGraphData<T>>` (strict: adjacent ids must be listed)
  - [x] `TryFrom<&DisymGraph<T>> for DisymGraphData<T>` / owned
  - [x] `TryFrom<&mut BuffReader<R>>` (requires `T: From<String>`)
  - [x] `TryFrom<BuffReader<R>>` (requires `T: From<String>`)
  - [x] `TryFrom<&File>` (requires `T: From<String>`)
  - [x] `TryFrom<File>` (requires `T: From<String>`)

- [x] `DirectedPathsDFS`
  - [x] `new()`
  - [x] `marked()` - Via `DigraphDFSShape`.
  - [x] `count()`
  - [x] `has_path_to()`
  - [x] `path_to()`
  - [x] `vertex_marked()` - Free function.

- [x] `DirectedCycle`
  - [x] `new()`
  - [x] `has_cycle()`
  - [x] `cycle()`

- [x] `DepthFirstOrder`
  - [x] `new()`
  - [x] `pre_order()`
  - [x] `post_order()`
  - [x] `pre()`
  - [x] `post()`
  - [x] `reverse_post()`
  - [x] `reverse_post_iter()`

- [x] `Topology`
  - [x] `new()`
  - [x] `order()`
  - [x] `order_iter()`
  - [x] `has_order()`
  - [x] `is_dag()`
  - [x] `rank()`

- [x] Utilities
  - [x] `invalid_vertex_msg()`
  - [x] `invalid_vert_symbol_msg()`
  - [x] `extract_vert_and_edge_counts_from_bufreader()`

- [x] ~~`DigraphMultiSourceDFS`  + `DigraphMultiSourceDirectedPathsDFS` (`DigraphMultiSourceDFS`).~~

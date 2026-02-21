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

- [x] `DisymGraph` - "Directed Symbol Graph"
  - [x] `add_edge()`
  - [x] `add_vertex()`
  - [x] `adj()`
  - [x] `adj_indices()`
  - [x] `digest_lines()`
  - [x] `edge_count()`
  - [x] `graph()`
  - [x] `has_vertex()`
  - [x] `indegree()`
  - [x] `index()`
  - [x] `indices()`
  - [x] `name()`
  - [x] `names()`
  - [x] `new()`
  - [x] `outdegree()`
  - [x] `reverse()`
  - [x] `validate_vertex()` - Should be settable.
  - [x] `vert_count()`
  - [x] `TryFrom<&mut BuffReader<R>>`
  - [x] `TryFrom<BuffReader<R>>`
  - [x] `TryFrom<&File>`
  - [x] `TryFrom<File>`

- [x] `DirectedPathsDFS`
  - [x] `dfs()`
  - [x] `marked()`
  - [x] `count()`
  - [x] `has_path_to()`
  - [x] `path_to()`

- [x] `DirectedCycle` impl.

- [x] `DepthFirstOrder`.

- [x] `Topology`

- [x] ~~`DigraphMultiSourceDFS`  + `DigraphMultiSourceDirectedPathsDFS` (`DigraphMultiSourceDFS`).~~

# Digraph

Reference implementations: Algorithms 4th Ed. - Chapter on digraphs: https://algs4.cs.princeton.edu/42digraph/

## Implementations:

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
  - [x] `validate_vertex()`
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
  - [x] `validate_vertex()` - @todo Should be settable.
  - [x] `vert_count()`

- [ ] Consider making `DisymGraph` generic - Will allow arbitrary types to function with the structure.

- [x] `DigraphDFS`
  - [x] `dfs()`
  - [x] `marked()`
  - [x] `count()`
  
- [x] `DigraphDipathsDFS`
  - [x] `dfs()`
  - [x] `marked()`
  - [x] `count()`
  - [x] `has_path_to()`
  - [x] `path_to()`
  
- [ ] `DigraphMultiSourceDFS`  + `DigraphMultiSourceDirectedPathsDFS` (`DigraphMultiSourceDFS`).

## General Todos:

- [x] ~~Struct should employ 'safe'/'result' variant methods, for `Result<...>` return types.~~ Digraph structs now contain 'safe' methods by default (instead of a variant that can panic and one that cannot ('safe' version)) - Less API surface for our purposes. 
- [x] Digraph impl change to use `Vec<String>`, instead of `HashSet<String>` for adjacency lists representations.
- [x] Digraph, "safe", methods should only return ~~`Option<>`~~ `Result<>`, unless some external (including stdlib) library call returns `Result<>`, or can panic, then returning `Result<>` is ok.
- [x] Decide whether methods that take `usize` values should validate said values. - These should validate given vertices due to allowing code panics to propagate higher up in the  code execution chain.
- [-] ~~Update control to store `Box<str>` instead of `String` - More memory efficient.~~

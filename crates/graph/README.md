# walrs_graph

Undirected graph data structures and algorithms for the walrs project. This crate provides efficient implementations of graph data structures and classic graph algorithms based on *Algorithms, 4th Edition* by Robert Sedgewick and Kevin Wayne.

## Overview

- **`Graph`** — undirected adjacency-list graph over `usize` vertex indices. Each undirected edge is stored in both endpoints' adjacency lists, so `edge_count()` reports twice the number of logical edges. Adjacency lists are kept sorted, enabling O(log deg(v)) `has_edge` via binary search.
- **`DFS`** — single-source depth-first search with reachability (`marked` / `has_path_to`), reachability count (`count`), and `path_to` reconstruction.
- **`SymbolGraph<T: Symbol>`** — graph keyed by typed symbols. Wraps a `Graph` and deduplicates vertices on insert. Comes with `GenericSymbol` (a `String`-backed implementation of `Symbol`) and a `TryFrom<&mut BufReader<File>>` impl that ingests whitespace-delimited adjacency files.

## Public API surface

Top-level re-exports from `walrs_graph` (everything in `src/graph/` is glob-re-exported via `src/lib.rs`):

- **Core**: `Graph`, `invalid_vertex_msg`
- **Search**: `DFS`
- **Symbol graph**: `SymbolGraph<T>`, `GenericSymbol`, `Symbol` (trait)
- **Utilities**: `extract_vert_and_edge_counts_from_bufreader`

Submodules (`graph`, `single_source_dfs`, `symbol_graph`, `traits`, `shared_utils`) are also `pub` if you'd rather refer to a type by its full path.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
walrs_graph = { path = "../graph" }
```

This crate has no Cargo features.

## Usage

### Basic Graph Operations

```rust
use walrs_graph::Graph;

// Create a graph with 5 vertices
let mut graph = Graph::new(5);

// Add edges (undirected)
graph.add_edge(0, 1)?;
graph.add_edge(0, 2)?;
graph.add_edge(1, 3)?;
graph.add_edge(2, 3)?;
graph.add_edge(3, 4)?;

// Query the graph
println!("Vertices: {}", graph.vert_count()); // 5
println!("Edges: {}", graph.edge_count() / 2); // 5 (divided by 2 for undirected)

// Get adjacent vertices
let adj = graph.adj(0)?;
println!("Vertices adjacent to 0: {:?}", adj); // [1, 2]

// Get degree (number of edges touching vertex)
let degree = graph.degree(3)?;
println!("Degree of vertex 3: {}", degree); // 3
```

### Loading Graph from File

Graph files should be formatted as:
```text
<number of vertices>
<number of edges>
<vertex> <vertex>
<vertex> <vertex>
...
```

Example:
```rust
use std::fs::File;
use walrs_graph::Graph;

let file = File::open("graph.txt")?;
let graph = Graph::try_from(&file)?;
```

### Depth-First Search

```rust
use walrs_graph::{Graph, DFS};

let mut graph = Graph::new(6);
graph.add_edge(0, 1)?;
graph.add_edge(0, 2)?;
graph.add_edge(1, 3)?;
graph.add_edge(2, 3)?;
graph.add_edge(3, 4)?;
// Note: vertex 5 is not connected

// Search from vertex 0
let dfs = DFS::new(&graph, 0);

// Check connectivity
assert!(dfs.marked(0));  // reachable
assert!(dfs.marked(4));  // reachable
assert!(!dfs.marked(5)); // not reachable

// Get path from source to vertex
if let Some(path) = dfs.path_to(4) {
    println!("Path from 0 to 4: {:?}", path);
}
```

### Symbol Graph

Use string names instead of numeric indices:

```rust
use std::fs::File;
use std::io::BufReader;
use walrs_graph::{SymbolGraph, GenericSymbol};

let file = File::open("routes.txt")?;
let mut reader = BufReader::new(file);
let sg: SymbolGraph<GenericSymbol> = (&mut reader).try_into()?;

// Query by symbol name
if sg.contains("JFK") {
    let adjacent = sg.adj("JFK")?;
    println!("Routes from JFK:");
    for dest in adjacent {
        println!("  {}", dest.id());
    }
}

// Get degree
let degree = sg.degree("JFK")?;
println!("Number of routes from JFK: {}", degree);
```

## API Conventions

This crate follows the same conventions as the `walrs_digraph` crate:

- Vertices are represented as `usize` indices (0-based)
- Methods that may fail return `Result<T, String>`
- Adjacency lists are kept sorted for predictable iteration
- Edge count for undirected graphs counts each edge twice (once per direction)
- Private fields prefixed with `_`

## Examples

Runnable examples live in [`examples/`](./examples/).

| Example             | Demonstrates                                                                  | Run                                                                                                                  |
| ------------------- | ----------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `graph_traversal`   | Loading a `Graph` from a file and running `DFS` from a chosen source vertex   | `cargo run -p walrs_graph --example graph_traversal -- ./crates/graph/test-fixtures/graph_test_tinyG.txt 0`          |
| `symbol_graph_demo` | Loading a `SymbolGraph<GenericSymbol>` and interactively querying neighbors via stdin | `cargo run -p walrs_graph --example symbol_graph_demo -- ./crates/graph/test-fixtures/acl_roles_symbol_graph.txt`    |

## Benchmarks

Run benchmarks to measure performance:

```bash
cargo bench -p walrs_graph
```

Benchmarks cover:
- Graph creation
- Adding edges
- Adjacency list queries
- Degree calculations
- Edge existence checks

## Testing

Run the test suite:

```bash
cargo test -p walrs_graph
```

Many public APIs include doc tests and unit tests.

## Performance Characteristics

| Operation | Time Complexity | Space |
|-----------|----------------|-------|
| `new(V)` | O(V) | O(V) |
| `add_edge(v, w)` | O(deg(v) log deg(v) + deg(w) log deg(w)) | O(E) |
| `adj(v)` | O(1) | - |
| `degree(v)` | O(1) | - |
| `has_edge(v, w)` | O(log deg(v)) | - |

Where V = number of vertices, E = number of edges

## Reference Implementation

Based on *Algorithms, 4th Edition* by Robert Sedgewick and Kevin Wayne:
- Chapter 4.1: Undirected Graphs
- Reference: https://algs4.cs.princeton.edu/41graph/

## Related Crates

- **walrs_digraph** — directed counterpart with `Digraph`, `DirectedCycle`, `Topology`, `DirectedPathsDFS`, and `DisymGraph`.

## License

Elastic-2.0. See the [LICENSE](./LICENSE) file alongside this crate.

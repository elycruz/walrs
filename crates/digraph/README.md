# walrs_digraph

Directed graph data structures and algorithms for the walrs project. Adjacency-list-backed `Digraph` plus a small set of classic DFS-based algorithms (cycle detection, depth-first ordering, topological sort, single-source reachability) and a string-keyed symbol-graph wrapper, all based on *Algorithms, 4th Edition* by Robert Sedgewick and Kevin Wayne.

## Overview

`walrs_digraph` provides:

- **`Digraph`** — directed graph with `usize`-indexed vertices, separate in-degree tracking, and `TryFrom` impls for files / `BufReader` so a graph can be loaded from a Sedgewick-format text file.
- **`DirectedCycle`** — depth-first cycle finder. Reports whether a cycle exists and returns one cycle when present. Handles self-loops and multiple components.
- **`DepthFirstOrder`** — DFS preorder, postorder, and reverse-postorder traversals (the latter is the basis of topological sort).
- **`Topology`** — topological order for a DAG, plus `is_dag()` / per-vertex `rank(v)`.
- **`DirectedPathsDFS`** — single-source reachability and path reconstruction from a given source vertex.
- **`DisymGraph`** — directed symbol graph that maps `String` names onto an underlying `Digraph` and supports `TryFrom<&File>` / `TryFrom<&mut BufReader<R>>` plus `TryFrom<DisymGraphData>` round-trips.

## Public API surface

Top-level re-exports from `walrs_digraph` (see `src/lib.rs`):

- **Core**: `Digraph`
- **Traversals & algorithms**: `DepthFirstOrder`, `DirectedCycle`, `DirectedPathsDFS`, `Topology`
- **Symbol graph**: `DisymGraph`, `DisymGraphData`, `invalid_vert_symbol_msg`
- **Traits**: `DigraphDFSShape` (shared marker for DFS structs)
- **Utilities**: `extract_vert_and_edge_counts_from_bufreader`, `invalid_vertex_msg`, `vertex_marked`

Submodules (`digraph`, `directed_cycle`, `depth_first_order`, `topology`, `directed_paths_dfs`, `disymgraph`, `traits`, `utils`) are also `pub` if you want to refer to a type by its full path.

## Installation

```toml
[dependencies]
walrs_digraph = { path = "../digraph" }
```

This crate has no Cargo features and no runtime dependencies.

## Usage

### Building a digraph

```rust
use walrs_digraph::Digraph;

let mut g = Digraph::new(4);
g.add_edge(0, 1).unwrap()
 .add_edge(0, 2).unwrap()
 .add_edge(1, 3).unwrap()
 .add_edge(2, 3).unwrap();

assert_eq!(g.vert_count(), 4);
assert_eq!(g.edge_count(), 4);
assert_eq!(g.outdegree(0).unwrap(), 2);
assert_eq!(g.indegree(3).unwrap(), 2);

// Reverse all edges
let r = g.reverse().unwrap();
assert_eq!(r.outdegree(3).unwrap(), 2);
```

### Loading from a file

`Digraph` parses the Sedgewick text format:

```text
<num_vertices>
<num_edges>
<from> <to>
<from> <to>
...
```

```rust
use std::fs::File;
use walrs_digraph::Digraph;

let f = File::open("./test-fixtures/digraph_test_tinyDG.txt")?;
let g: Digraph = (&f).try_into()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Cycle detection

```rust
use walrs_digraph::{Digraph, DirectedCycle};

let mut g = Digraph::new(3);
g.add_edge(0, 1).unwrap();
g.add_edge(1, 2).unwrap();
g.add_edge(2, 0).unwrap(); // closes the cycle

let finder = DirectedCycle::new(&g);
assert!(finder.has_cycle());
assert!(finder.cycle().is_some());
```

### Topological sort

```rust
use walrs_digraph::{Digraph, Topology};

let mut g = Digraph::new(4);
g.add_edge(0, 1).unwrap();
g.add_edge(0, 2).unwrap();
g.add_edge(1, 3).unwrap();
g.add_edge(2, 3).unwrap();

let topo = Topology::new(&g);
assert!(topo.is_dag());
let order = topo.order().unwrap();
assert_eq!(order.len(), 4);
```

`Topology::new` returns a no-order instance when the digraph contains a cycle; check with `has_order()` / `is_dag()`.

### Single-source paths

```rust
use walrs_digraph::{Digraph, DirectedPathsDFS, DigraphDFSShape};

let mut g = Digraph::new(4);
g.add_edge(0, 1).unwrap();
g.add_edge(1, 2).unwrap();

let dfs = DirectedPathsDFS::new(&g, 0).unwrap();
assert!(dfs.marked(2).unwrap());
assert!(dfs.path_to(2).is_some());
assert_eq!(dfs.count(), 3);
```

### Symbol graph

```rust
use walrs_digraph::DisymGraph;

let mut sg = DisymGraph::new();
sg.add_edge("admin", &["user", "moderator"]).unwrap();
sg.add_edge("user", &["guest"]).unwrap();

assert!(sg.contains("admin"));
assert_eq!(sg.adj("admin").unwrap().len(), 2);
```

`DisymGraph` also implements `TryFrom<&File>` / `TryFrom<&mut BufReader<R>>` for whitespace-delimited adjacency files (one `<vertex> <neighbor>...` line per row), and `TryFrom<DisymGraphData>` (`Vec<(String, Option<Vec<String>>)>`) for in-memory construction.

## API conventions

- Vertices are `usize` indices (0-based).
- Methods that may fail return `Result<T, String>` with messages produced via `invalid_vertex_msg` / `invalid_vert_symbol_msg`.
- `add_edge(v, w)` records `v -> w` and increments `indegree(w)`. Self-loops and duplicate edges are allowed.
- Private fields are prefixed with `_`.

## Examples

Runnable examples live in [`examples/`](./examples/).

| Example            | Demonstrates                                                            | Run                                                                                           |
| ------------------ | ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `directed_cycle`   | Loading a `Digraph` from a file and reporting any cycle via `DirectedCycle` | `cargo run -p walrs_digraph --example directed_cycle -- ./crates/digraph/test-fixtures/digraph_test_tinyDG.txt` |

## Testing

```sh
cargo test -p walrs_digraph
```

Doc tests on most public methods exercise the documented behaviour, alongside per-module unit tests under `src/`.

## Reference

- *Algorithms, 4th Edition* by Robert Sedgewick and Kevin Wayne — chapter 4.2 ("Directed Graphs"): https://algs4.cs.princeton.edu/42digraph/
- Reference Java implementations: `Digraph.java`, `DirectedCycle.java`, `DepthFirstOrder.java`, `Topological.java`.

## Related crates

- **walrs_graph** — undirected counterpart with `Graph`, `DFS`, and `SymbolGraph<T>`.

## License

Elastic-2.0. See the [LICENSE](./LICENSE) file alongside this crate.

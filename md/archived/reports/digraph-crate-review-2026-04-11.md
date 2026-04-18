# Digraph Crate (`walrs_digraph`) — Code Review

**Date:** 2026-04-11
**Issue:** #164
**Scope:** `crates/digraph/src/` — all source files
**Focus:** Correctness, soundness, edge cases, error handling, performance, trait design, test coverage, documentation

---

## Summary

| Severity | Count |
|---|---|
| 🟠 High | 2 |
| 🟡 Medium | 3 |
| 🔵 Low | 7 |
| ✅ Clean | 3 files (`depth_first_order.rs`, `directed_cycle.rs`, `topology.rs` — minor issues only) |

All 82 unit tests and 23 doc tests pass. No `unsafe` code is present.

---

## 🟠 High (2)

### 1. `digraph.rs:136` — `add_edge` sorts adjacency list on every insertion

```rust
let adj = &mut self._adj_lists[v];
adj.push(w);
adj.sort_unstable(); // ← called every time
```

Each call to `add_edge` pushes a vertex then sorts the entire adjacency list.
For a vertex with degree *d*, this is *O(d log d)* per insertion. Building a
graph with *E* edges becomes *O(E × d_max × log d_max)* instead of *O(E)*
for a plain push, or *O(V + E)* with a single post-construction sort.

On the Sedgewick tinyDG test fixture (13 vertices, 22 edges) this is negligible,
but on real-world graphs with thousands of edges it becomes a bottleneck.

**Additionally**, duplicate edges are silently accepted — two calls to
`add_edge(0, 1)` produce two entries in the adjacency list and increment
`_edge_count` and `_in_degree[1]` twice. Whether duplicates should be
allowed is a design choice, but it should be documented either way.

**Suggested fix:** Remove the `sort_unstable()` from `add_edge`. If sorted
adjacency lists are required for algorithms, provide a `sort_adj_lists()`
method or sort once after construction.

---

### 2. `digraph.rs:183–201` & `utils.rs:27–37` — `unwrap()`/`expect()` in `Result`-returning paths cause panics on malformed input

The `TryFrom<&mut BufReader<R>> for Digraph` implementation returns
`Result<Self, Box<dyn Error>>`, but **panics** on several malformed-input cases
rather than returning `Err`:

**a) `digraph.rs:183–185` — `unwrap()` on helper call:**
```rust
let vert_count = extract_vert_and_edge_counts_from_bufreader(reader)
    .unwrap()  // panics if Err
    .0;
```

**b) `utils.rs:29–30` — `expect()` on I/O + `unwrap()` on parse:**
```rust
reader
    .read_line(&mut s)
    .expect("Unable to read \"vertex count\" line from buffer");
let vertices_count = s.trim().parse::<usize>().unwrap(); // panics on "abc"
```

**c) `digraph.rs:197` — `unwrap()` on parse in edge lines:**
```rust
.map(|x| x.parse::<usize>().unwrap()) // panics on non-numeric
```

**d) `digraph.rs:201` — Indexing assumes ≥2 values per line:**
```rust
dg.add_edge(verts[0], verts[1])?; // panics if line has <2 values
```

**Suggested fix:** Replace all `unwrap()`/`expect()` calls with `?` or
`.map_err(...)` to propagate errors through the `Result` return type.
The same pattern applies to `extract_vert_and_edge_counts_from_bufreader`.

---

## 🟡 Medium (3)

### 3. `topology.rs:187` — Off-by-one in `rank()` error message

```rust
pub fn rank(&self, v: usize) -> Result<Option<usize>, String> {
    match &self._rank {
        Some(rank) => {
            if v >= rank.len() {
                Err(invalid_vertex_msg(v, rank.len())) // ← should be rank.len() - 1
            }
            ...
        }
    }
}
```

`invalid_vertex_msg(v, max_v)` formats as *"Vertex {v} is outside defined range
0-{max_v}"*. Passing `rank.len()` (e.g., 3 for a 3-vertex graph) produces
*"range 0-3"*, implying vertex 3 is valid. `Digraph::validate_vertex` correctly
passes `len - 1`; this should do the same.

**Suggested fix:**
```rust
Err(invalid_vertex_msg(v, if rank.len() > 0 { rank.len() - 1 } else { 0 }))
```

---

### 4. `disymgraph.rs:12` — Doc comment incorrectly calls DisymGraph a "DAG"

```rust
/// `DisymGraph` A Directed Acyclic Graph (B-DAG) data structure.
```

`DisymGraph` is a **directed symbol graph** — it does not enforce acyclicity.
Cycles can be added freely (e.g., `add_edge("a", &["b"])` then
`add_edge("b", &["a"])`). The doc comment is misleading.

**Suggested fix:** Change to:
```rust
/// `DisymGraph` — A directed symbol graph data structure that maps
/// string labels to vertices in an underlying `Digraph`.
```

---

### 5. `disymgraph.rs:68–69` — `adj_indices` uses `unwrap().into()` with confusing type coercion

```rust
pub fn adj_indices(&self, symbol_name: &str) -> Option<&Vec<usize>> {
    if let Some(i) = self.index(symbol_name) {
        self._graph.adj(i).unwrap().into()
    } else {
        None
    }
}
```

`Digraph::adj()` returns `Result<&Vec<usize>, String>`. The `unwrap()` is safe
here (since `index()` already validated the vertex), but the `.into()` to convert
`&Vec<usize>` → `Option<&Vec<usize>>` is non-obvious. A reader would expect
`Some(...)` wrapping.

**Suggested fix:**
```rust
Some(self._graph.adj(i).unwrap())
```

---

## 🔵 Low (7)

### 6. `disymgraph.rs:87` — O(n) linear scan for vertex lookup

```rust
pub fn index(&self, symbol_name: &str) -> Option<usize> {
    self._vertices.iter().position(|v| v == symbol_name)
}
```

Every call to `index()` scans the entire vertex list. Since `index()` is called
by `add_vertex()`, `add_edge()`, `contains()`, `adj()`, `adj_indices()`, and
`validate_vertex()`, building a graph with *V* vertices and *E* edges is
*O(V × (V + E))*. A `HashMap<String, usize>` alongside the `Vec` would give
*O(1)* lookups.

---

### 7. `digraph.rs:116–123` — `add_vertex` loop can use `Vec::resize`

```rust
loop {
    if v_len > v { break; }
    self._adj_lists.push(Vec::new());
    self._in_degree.push(0);
    v_len += 1;
}
```

**Suggested fix:**
```rust
if v >= self._adj_lists.len() {
    self._adj_lists.resize_with(v + 1, Vec::new);
    self._in_degree.resize(v + 1, 0);
}
```

---

### 8. `directed_paths_dfs.rs:62–64` — Verbose conditional pattern

```rust
let path_exists = self.has_path_to(v);
if path_exists.is_err() || (path_exists.is_ok() && !path_exists.unwrap()) {
    return None;
}
```

**Suggested fix:**
```rust
if !self.has_path_to(v).unwrap_or(false) {
    return None;
}
```

---

### 9. `directed_paths_dfs.rs:74` — `unwrap()` without context message

```rust
x = self._edge_to[x].unwrap();
```

The `unwrap()` is safe here (invariant: all vertices on the path have a parent
set by DFS), but `expect("_edge_to invariant violated: vertex on path has no parent")`
would document the invariant and aid debugging if ever triggered.

---

### 10. `lib.rs:10–17` — Glob re-exports from all modules

```rust
pub use depth_first_order::*;
pub use digraph::*;
// ... (all modules)
```

Wildcard re-exports can lead to name collisions as the crate grows and make the
public API surface unclear. Consider explicit re-exports:
```rust
pub use depth_first_order::DepthFirstOrder;
pub use digraph::Digraph;
// etc.
```

---

### 11. `traits.rs` — Minimal trait with single implementor

```rust
pub trait DigraphDFSShape {
    fn marked(&self, i: usize) -> Result<bool, String>;
}
```

`DigraphDFSShape` has one method and one implementor (`DirectedPathsDFS`).
If there are no plans for additional implementors, this trait adds indirection
without benefit. Consider removing it or keeping it as a private implementation
detail.

---

### 12. `digraph.rs:146` — `validate_vertex` error message edge case for empty graph

```rust
Err(invalid_vertex_msg(v, if len > 0 { len - 1 } else { 0 }))
```

For an empty graph (`len == 0`), vertex 5 produces *"Vertex 5 is outside defined
range 0-0"*. This implies there is a vertex 0, which there isn't. Consider
a separate message for empty graphs: *"Vertex 5 is invalid: graph has no vertices"*.

---

## ✅ Positive Observations

1. **No `unsafe` code** — the entire crate is safe Rust.
2. **No external dependencies** — standalone crate with only `std`.
3. **Solid test coverage** — 82 unit tests + 23 doc tests covering all major algorithms (cycle detection, DFS, topological sort, path finding) and edge cases (empty graphs, single vertex, disconnected components, self-loops).
4. **Algorithm correctness** — cycle detection, DFS ordering, and topological sort algorithms are faithful implementations of Sedgewick/Wayne's Algorithms 4th Ed. and produce correct results.
5. **Doc comments with examples** — most public API methods have doc examples that are tested as doctests.
6. **`debug_assert!(out.check())` in `DepthFirstOrder::new`** — good use of debug assertions for internal consistency checking.
7. **`TryFrom` impls for multiple reader types** — ergonomic API for constructing graphs from files.

---

## Missing Test Coverage

While test coverage is generally good, the following edge cases lack tests:

1. **`Digraph::try_from` with malformed input** — no tests for files with non-numeric data, missing lines, or invalid edge references.
2. **Duplicate edges** — no tests verifying behavior when the same edge is added twice.
3. **`DirectedPathsDFS::new` with invalid source vertex** — no test for the `Err` path.
4. **`Digraph::validate_vertex` on empty graph** — message correctness not asserted.
5. **Self-loops in `Digraph`** — only tested via `DirectedCycle`; not tested for degree counts.

---

## Recommendations (Priority Order)

1. **Fix panicking paths** (High #2) — Replace `unwrap()`/`expect()` with `?` in all `Result`-returning functions.
2. **Remove per-insertion sort** (High #1) — Defer sorting or remove it entirely.
3. **Fix `Topology::rank` off-by-one** (Medium #3) — Align error message with `Digraph::validate_vertex`.
4. **Fix `DisymGraph` doc comment** (Medium #4) — Correct "DAG" → "directed symbol graph".
5. **Add `HashMap` index to `DisymGraph`** (Low #6) — For O(1) vertex lookups.
6. **Add tests for malformed input and edge cases** — Cover the missing scenarios listed above.

# DigraphDicycle - Directed Cycle Detection

The `DigraphDicycle` struct is a Rust implementation of the DirectedCycle algorithm from *Algorithms, 4th Edition* by Robert Sedgewick and Kevin Wayne.

## Overview

This module determines whether a directed graph (digraph) has a directed cycle. If a cycle exists, it can retrieve one such cycle.

## Algorithm

- **Implementation**: Depth-first search (DFS)
- **Time Complexity**: Θ(V + E) where V is the number of vertices and E is the number of edges
- **Space Complexity**: Θ(V) extra space (not including the digraph)
- **Instance Methods**: Θ(1) time

## Usage

```rust
use walrs_digraph::{Digraph, DigraphDicycle};

fn main() {
    // Create a digraph with a cycle
    let mut g = Digraph::new(3);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 0).unwrap(); // Creates cycle: 0 -> 1 -> 2 -> 0
    
    // Detect the cycle
    let finder = DigraphDicycle::new(&g);
    
    if finder.has_cycle() {
        println!("Directed cycle found!");
        if let Some(cycle) = finder.cycle() {
            println!("Cycle: {:?}", cycle);
        }
    } else {
        println!("No directed cycle");
    }
}

```

## Example with File

```bash
# Run the example with a test file
cargo run --example directed_cycle ../test-fixtures/digraph_test_tinyDG.txt
```

## API

### `new(g: &Digraph) -> Self`
Creates a new directed cycle finder for the given digraph. Runs DFS from each unmarked vertex to find cycles.

### `has_cycle(&self) -> bool`
Returns `true` if the digraph has a directed cycle, `false` otherwise.

### `cycle(&self) -> Option<&Vec<usize>>`
Returns a directed cycle if one exists, or `None` otherwise. The cycle is represented as a vector of vertex indices where the first and last elements are the same vertex.

## Implementation Details

The algorithm uses three main data structures:
- `_marked`: Tracks visited vertices during DFS
- `_on_stack`: Tracks vertices currently on the recursion stack
- `_edge_to`: Stores the path taken to reach each vertex

When a back edge is detected (an edge to a vertex currently on the stack), a cycle has been found. The algorithm then traces back through `_edge_to` to reconstruct the cycle.

## References

- [Algorithms, 4th Edition - Section 4.2](https://algs4.cs.princeton.edu/42digraph)
- [DirectedCycle.java](https://algs4.cs.princeton.edu/42digraph/DirectedCycle.java)

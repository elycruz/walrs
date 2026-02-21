use std::env;
use std::fs::File;
use walrs_graph::{DFS, Graph};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();

  if args.len() < 3 {
    eprintln!("Usage: {} <input_file> <source_vertex>", args[0]);
    eprintln!("Example: {} tinyG.txt 0", args[0]);
    std::process::exit(1);
  }

  let filename = &args[1];
  let source_vertex: usize = args[2]
    .parse()
    .expect("Source vertex must be a valid number");

  let file = File::open(filename)?;
  let graph = Graph::try_from(&file)?;

  println!(
    "Graph with {} vertices and {} edges",
    graph.vert_count(),
    graph.edge_count() / 2
  ); // Divide by 2 for undirected graph

  // Perform depth-first search from source vertex
  let dfs = DFS::new(&graph, source_vertex);

  println!("\nVertices reachable from {}:", source_vertex);
  for v in 0..graph.vert_count() {
    if dfs.marked(v) {
      print!("{} ", v);
    }
  }
  println!("\n");

  // Show adjacency lists
  println!("Adjacency lists:");
  for v in 0..graph.vert_count() {
    print!("{}: ", v);
    if let Ok(adj) = graph.adj(v) {
      for w in adj {
        print!("{} ", w);
      }
    }
    println!();
  }

  Ok(())
}

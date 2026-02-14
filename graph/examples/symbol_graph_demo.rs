use std::env;
use std::fs::File;
use std::io::BufReader;
use walrs_graph::{SymbolGraph, GenericSymbol, Symbol};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        eprintln!("Example: {} routes.txt", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    let symbol_graph: SymbolGraph<GenericSymbol> = (&mut reader).try_into()?;

    println!("Symbol Graph with {} vertices and {} edges",
             symbol_graph.vert_count(),
             symbol_graph.edge_count() / 2); // Divide by 2 for undirected

    // Interactive query mode
    println!("\nEnter a vertex name (or 'quit' to exit):");

    let stdin = std::io::stdin();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        stdin.read_line(&mut buffer)?;
        let vertex_name = buffer.trim();

        if vertex_name == "quit" || vertex_name.is_empty() {
            break;
        }

        if symbol_graph.contains(vertex_name) {
            println!("  Adjacent vertices:");
            match symbol_graph.adj(vertex_name) {
                Ok(adjacent) => {
                    for v in adjacent {
                        println!("    {}", v.id());
                    }
                }
                Err(e) => println!("  Error: {}", e),
            }
        } else {
            println!("  '{}' is not in the graph", vertex_name);
        }

        println!("\nEnter another vertex name (or 'quit' to exit):");
    }

    Ok(())
}

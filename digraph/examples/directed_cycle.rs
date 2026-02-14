use std::env;
use std::fs::File;
use walrs_digraph::{Digraph, DirectedCycle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        eprintln!("Example: {} tinyDG.txt", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let file = File::open(filename)?;
    let digraph = Digraph::try_from(&file)?;

    let finder = DirectedCycle::new(&digraph);

    if finder.has_cycle() {
        print!("Directed cycle: ");
        if let Some(cycle) = finder.cycle() {
            for v in cycle {
                print!("{} ", v);
            }
        }
        println!();
    } else {
        println!("No directed cycle");
    }

    println!();
    Ok(())
}

pub mod digraph_test;

use std::fs::File;
use std::io::{BufRead, BufReader, Seek};

use walrs_graph::graph::shared_utils::extract_vert_and_edge_counts_from_bufreader;
use walrs_graph::graph::symbol_graph::SymbolGraph;
use walrs_graph::graph::{GenericSymbol, Graph};

#[test]
pub fn test_graph_tiny_text_undirected() -> std::io::Result<()> {
  let file_path = "../../test-fixtures/graph_test_tinyG.txt";

  // Get representation of graph
  let f = File::open(&file_path)?;

  // Graph vertex, and edge, sizes
  let mut reader = BufReader::new(f);
  let g1: Graph = (&mut reader).try_into().unwrap();

  // Rewind bufreader for reuse
  if let Err(err) = reader.rewind() {
    panic!("{:?}", err);
  }

  // Reuse bufreader to create an additional graph we can use to test 'From<...>' impl.
  match extract_vert_and_edge_counts_from_bufreader(&mut reader) {
    Ok((vert_count, edge_count)) => {
      // Create graph
      let mut g = Graph::new(vert_count);
      if let Err(err) = g.digest_lines((&mut reader).lines()) {
        panic!("{:?}", err);
      }

      // Test edges length
      assert_eq!(
        g.edge_count(),
        edge_count * 2, // @todo text file, loaded, should contain correct edge count
        "Should have expected edges length"
      );

      // Test vertices length
      assert_eq!(
        g.vert_count(),
        vert_count,
        "Should have expected vertices length"
      );

      assert_eq!(
        g1.edge_count(),
        g.edge_count(),
        "both graphs should contain same edge count"
      );

      assert_eq!(
        g1.vert_count(),
        g.vert_count(),
        "both graphs should contain same vert count"
      );

      // @todo Test all contained edges, and/or, adjacency lists.
      // Print graph
      // println!("{:?}", &g);

      Ok(())
    }
    Err(err) => panic!("{:?}", err),
  }
}

#[test]
pub fn test_symbol_graph() -> std::io::Result<()> {
  // Get representation of graph
  let f = File::open("../../test-fixtures/symbol_graph_test_routes.txt")?;

  // Graph vertex, and edge, sizes
  let mut reader = BufReader::new(f);

  let g: SymbolGraph<GenericSymbol> = (&mut reader).try_into().unwrap();

  println!("{:?}", &g);

  Ok(())
}

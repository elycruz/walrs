#[cfg(test)]
mod test {
  use std::fs::File;
  use std::io::{BufReader, Seek};
  use walrs_graph::digraph::Digraph;
  use walrs_graph::graph::shared_utils::extract_vert_and_edge_counts_from_bufreader;

  #[test]
  pub fn test_from_file_ref_impl() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get digraph data
    let f = File::open(&file_path)?;

    // Create graph
    let _: Digraph = (&f).into();

    // println!("{:?}", dg);

    Ok(())
  }

  #[test]
  pub fn test_from_mut_ref_buf_reader_impl() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get digraph data
    let f = File::open(&file_path)?;
    let mut reader = BufReader::new(f);

    // Create graph (impls for `From<BufReader<R: std::io::Read>>` and `From<File>` are defined for `Digraph` struct
    let dg: Digraph = (&mut reader).into();

    // println!("{:?}", dg);

    // Rewind reader and extract vert and edge count from first lines
    reader.rewind()?;

    let (expected_vert_count, expected_edge_count) =
      extract_vert_and_edge_counts_from_bufreader(&mut reader)?;

    assert_eq!(
      dg.vert_count(),
      expected_vert_count,
      "Vert count is invalid"
    );
    assert_eq!(
      dg.edge_count(),
      expected_edge_count,
      "Edge count is invalid"
    );

    Ok(())
  }

  #[test]
  pub fn test_try_from_mut_ref_buf_reader_impl() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get digraph data
    let f = File::open(&file_path)?;
    let mut reader = BufReader::new(f);

    // Create graph (impls for `From<BufReader<R: std::io::Read>>` and `From<File>` are defined for `Digraph` struct
    let dg: Digraph = (&mut reader).try_into()?;

    // println!("{:?}", dg);

    // Rewind reader and extract vert and edge count from first lines
    reader.rewind()?;

    let (expected_vert_count, expected_edge_count) =
      extract_vert_and_edge_counts_from_bufreader(&mut reader)?;

    assert_eq!(
      dg.vert_count(),
      expected_vert_count,
      "Vert count is invalid"
    );
    assert_eq!(
      dg.edge_count(),
      expected_edge_count,
      "Edge count is invalid"
    );

    Ok(())
  }

  #[test]
  pub fn test_from_buf_reader_impl() -> Result<(), std::io::Error> {
    let file_path = "../test-fixtures/graph_test_tinyG.txt";

    // Get digraph data
    let f = File::open(&file_path)?;

    // Create graph
    let _: Digraph = BufReader::new(f).into();

    // println!("{:?}", dg);

    Ok(())
  }
}

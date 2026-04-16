use std::io::{BufRead, BufReader};

/// Extracts vertex, and, edge counts from top (first two lines) of text file containing
/// vertices, and their edges;  E.g.,
/// **note:** annotations are only for example here - only numbers are allowed in the file;
///  control errors out on 'parse error' otherwise..
///
/// ```text
///  3      // Num. vertices
///  6      // Num. edges
///  0 1 2  // Edges from `0` to ...
///  1 0 2  // Edges from `1` to ...
///  2 1 0  // ...
/// ```
pub fn extract_vert_and_edge_counts_from_bufreader<R: std::io::Read>(
  reader: &mut BufReader<R>,
) -> Result<(usize, usize), String> {
  // Extract vertex, and edge, counts from buffer
  let mut s = String::new();

  // Extract vertices count
  reader
    .read_line(&mut s)
    .map_err(|e| format!("Unable to read \"vertex count\" line from buffer: {}", e))?;
  let vertices_count = s
    .trim()
    .parse::<usize>()
    .map_err(|e| format!("Failed to parse vertex count \"{}\": {}", s.trim(), e))?;
  s.clear();

  // Edge count currently, not required
  reader
    .read_line(&mut s)
    .map_err(|e| format!("Unable to read \"edge count\" line from buffer: {}", e))?;
  let edges_count = s
    .trim()
    .parse::<usize>()
    .map_err(|e| format!("Failed to parse edge count \"{}\": {}", s.trim(), e))?;

  Ok((vertices_count, edges_count))
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_extract_valid_counts() {
    let data = b"13\n13\n0 5\n";
    let mut reader = BufReader::new(&data[..]);
    let (verts, edges) = extract_vert_and_edge_counts_from_bufreader(&mut reader).unwrap();
    assert_eq!(verts, 13);
    assert_eq!(edges, 13);
  }

  #[test]
  fn test_extract_malformed_vertex_line() {
    let data = b"abc\n10\n";
    let mut reader = BufReader::new(&data[..]);
    let result = extract_vert_and_edge_counts_from_bufreader(&mut reader);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse vertex count"));
  }

  #[test]
  fn test_extract_malformed_edge_line() {
    let data = b"10\nxyz\n";
    let mut reader = BufReader::new(&data[..]);
    let result = extract_vert_and_edge_counts_from_bufreader(&mut reader);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse edge count"));
  }

  #[test]
  fn test_extract_empty_input() {
    let data = b"";
    let mut reader = BufReader::new(&data[..]);
    let result = extract_vert_and_edge_counts_from_bufreader(&mut reader);
    assert!(result.is_err());
  }

  #[test]
  fn test_extract_single_line_only() {
    let data = b"5\n";
    let mut reader = BufReader::new(&data[..]);
    let result = extract_vert_and_edge_counts_from_bufreader(&mut reader);
    // Second line is empty → parse error
    assert!(result.is_err());
  }

  #[test]
  fn test_extract_with_whitespace() {
    let data = b"  7  \n  3  \n";
    let mut reader = BufReader::new(&data[..]);
    let (verts, edges) = extract_vert_and_edge_counts_from_bufreader(&mut reader).unwrap();
    assert_eq!(verts, 7);
    assert_eq!(edges, 3);
  }
}

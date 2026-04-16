use std::io::{BufRead, BufReader};

/// Returns panic message for invalid vertices;  Exported for use in testing.
pub fn invalid_vertex_msg(v: usize, max_v: usize) -> String {
  format!("Vertex {} is outside defined range 0-{}", v, max_v)
}

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
    .map_err(|e| format!("Failed to parse vertex count '{}': {}", s.trim(), e))?;
  s.clear();

  // Extract edge count
  reader
    .read_line(&mut s)
    .map_err(|e| format!("Unable to read \"edge count\" line from buffer: {}", e))?;
  let edges_count = s
    .trim()
    .parse::<usize>()
    .map_err(|e| format!("Failed to parse edge count '{}': {}", s.trim(), e))?;

  Ok((vertices_count, edges_count))
}

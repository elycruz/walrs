/// Simple trait to contain shared definitions for Digraph*DFS structs.
pub trait DigraphDFSShape {
  /// Returns a `Result` indicating whether  a path from 'source vertex' to 'i' exists.
  fn marked(&self, i: usize) -> Result<bool, String>;
}

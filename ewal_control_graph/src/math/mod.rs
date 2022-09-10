/// Calculates nth triangular number, where n is a natural number.
///
/// ```rust
/// use ewal_control_graph::math::triangular_num;
///
/// assert_eq!(triangular_num(2), 3);
/// ```
///
pub fn triangular_num(n: usize) -> usize {
  n * (n + 1) / 2
}

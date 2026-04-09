/// General filter trait for transforming values.
///
/// The `Output` associated type defaults to `T`, allowing filters to transform
/// values to different types if needed.
///
/// # Examples
///
/// ```rust
/// use walrs_filter::Filter;
///
/// struct UppercaseFilter;
///
/// impl Filter<String> for UppercaseFilter {
///     type Output = String;
///     
///     fn filter(&self, value: String) -> Self::Output {
///         value.to_uppercase()
///     }
/// }
///
/// let filter = UppercaseFilter;
/// assert_eq!(filter.filter("hello".to_string()), "HELLO");
/// ```
pub trait Filter<T> {
  type Output;

  fn filter(&self, value: T) -> Self::Output;
}

/// Fallible filter trait for transformations that can fail.
///
/// This is the fallible counterpart to [`Filter`], mirroring the
/// `Validate`/`ValidateRef` pattern. Use this for filters that can
/// legitimately fail (e.g., base64 decode, JSON parse, URL decode).
///
/// # Examples
///
/// ```rust
/// use walrs_filter::{TryFilter, FilterError};
///
/// struct ParseIntFilter;
///
/// impl TryFilter<String> for ParseIntFilter {
///     type Output = i64;
///     
///     fn try_filter(&self, value: String) -> Result<Self::Output, FilterError> {
///         value.trim().parse::<i64>().map_err(|e|
///             FilterError::new(e.to_string()).with_name("ParseInt")
///         )
///     }
/// }
///
/// let filter = ParseIntFilter;
/// assert_eq!(filter.try_filter("42".to_string()).unwrap(), 42);
/// assert!(filter.try_filter("not_a_number".to_string()).is_err());
/// ```
pub trait TryFilter<T> {
  type Output;

  fn try_filter(&self, value: T) -> Result<Self::Output, crate::FilterError>;
}

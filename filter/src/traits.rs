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

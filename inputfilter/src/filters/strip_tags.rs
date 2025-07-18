use std::borrow::Cow;
use std::sync::OnceLock;

static DEFAULT_AMMONIA_BUILDER: OnceLock<ammonia::Builder> = OnceLock::new();

/// Sanitizes incoming HTML using the [Ammonia](https://docs.rs/ammonia/1.0.0/ammonia/) crate.
///
/// ```rust
/// use walrs_inputfilter::filters::StripTagsFilter;
/// use std::borrow::Cow;
///
/// let filter = StripTagsFilter::new();
///
/// for (i, (incoming_src, expected_src)) in [
///   ("", ""),
///   ("Socrates'", "Socrates'"),
///   ("\"Hello\"", "\"Hello\""),
///   ("Hello", "Hello"),
///   ("<script>alert(\"Hello World\");</script>", ""),        // Removes `script` tags, by default
///   ("<p>The quick brown fox</p><style>p { font-weight: bold; }</style>",
///    "<p>The quick brown fox</p>"),                          // Removes `style` tags, by default
///   ("<p>The quick brown fox", "<p>The quick brown fox</p>") // Fixes erroneous markup, by default
///  ]
///  .into_iter().enumerate() {
///     println!("Filter test {}: filter({}) == {}", i, incoming_src, expected_src);
///     let result = filter.filter(incoming_src.into());
///
///     assert_eq!(result, expected_src.to_string());
///     assert_eq!(filter(incoming_src.into()), result);
///  }
/// ```
///
pub struct StripTagsFilter<'a> {
  /// Ammonia builder used to sanitize incoming HTML.
  ///
  /// If `None`, a default builder is used when `filter`/instance is called.
  pub ammonia: Option<ammonia::Builder<'a>>,
}

impl StripTagsFilter<'_> {
  /// Constructs a new `StripTagsFilter` instance.
  pub fn new() -> Self {
    Self { ammonia: None }
  }

  /// Filters incoming HTML using the contained `ammonia::Builder` instance.
  ///  If no instance is set gets/(and/or) initializes a new (default, and singleton) instance.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use std::sync::OnceLock;
  /// use ammonia::Builder as AmmoniaBuilder;
  /// use walrs_inputfilter::filters::StripTagsFilter;
  ///
  /// // Using default settings:
  /// let filter = StripTagsFilter::new();
  ///
  /// let subject = r#"<p>Hello</p><script>alert('hello');</script>
  ///        <style>p { font-weight: bold; }</style>"#;
  ///
  /// // Ammonia removes `script`, and `style` tags by default.
  /// assert_eq!(filter.filter(subject.into()).trim(),
  ///   "<p>Hello</p>"
  /// );
  ///
  /// // Using custom settings:
  /// // Instantiate a custom sanitizer instance.
  /// let mut sanitizer = AmmoniaBuilder::default();
  /// let additional_allowed_tags = vec!["style"];
  ///
  /// sanitizer
  ///   .add_tags(&additional_allowed_tags) // Add 'style' tag to "tags-whitelist"
  ///
  ///   // Remove 'style' tag from "tags-blacklist"
  ///   .rm_clean_content_tags(&additional_allowed_tags);
  ///
  /// let filter = StripTagsFilter {
  ///   ammonia: Some(sanitizer)
  /// };
  ///
  /// // Notice `style` tags are no longer removed.
  /// assert_eq!(filter.filter(
  ///     "<script>alert('hello');</script><style>p { font-weight: bold; }</style>".into()
  ///   ),
  ///   "<style>p { font-weight: bold; }</style>"
  /// );
  ///
  /// // Can also be called as an function trait (has `FN*` traits implemented).
  /// assert_eq!(filter(
  ///     "<script>alert('hello');</script><style>p { font-weight: bold; }</style>".into()
  ///   ),
  ///   "<style>p { font-weight: bold; }</style>"
  /// );
  ///
  /// ```
  ///
  pub fn filter<'b>(&self, input: Cow<'b, str>) -> Cow<'b, str> {
    match self.ammonia {
      None => Cow::Owned(
        DEFAULT_AMMONIA_BUILDER
          .get_or_init(ammonia::Builder::default)
          .clean(&input)
          .to_string(),
      ),
      Some(ref sanitizer) => Cow::Owned(sanitizer.clean(&input).to_string()),
    }
  }
}

impl Default for StripTagsFilter<'_> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'b> FnOnce<(Cow<'b, str>,)> for StripTagsFilter<'_> {
  type Output = Cow<'b, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'b, str>,)) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'b> FnMut<(Cow<'b, str>,)> for StripTagsFilter<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'b, str>,)) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'b> Fn<(Cow<'b, str>,)> for StripTagsFilter<'_> {
  extern "rust-call" fn call(&self, args: (Cow<'b, str>,)) -> Self::Output {
    self.filter(args.0)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_construction() {
    let _ = StripTagsFilter::new();
    let _ = StripTagsFilter {
      ammonia: Some(ammonia::Builder::default()),
    };
  }

  #[test]
  fn test_filter() {
    let filter = StripTagsFilter::new();

    for (i, (incoming_src, expected_src)) in [
      ("", ""),
      ("Socrates'", "Socrates'"),
      ("\"Hello\"", "\"Hello\""),
      ("Hello", "Hello"),
      ("<script>alert(\"Hello World\");</script>", ""), // Removes `script` tags, by default
      (
        "<p>The quick brown fox</p><style>p { font-weight: bold; }</style>",
        "<p>The quick brown fox</p>",
      ), // Removes `style` tags, by default
      ("<p>The quick brown fox", "<p>The quick brown fox</p>"), // Fixes erroneous markup
    ]
    .into_iter()
    .enumerate()
    {
      println!(
        "Filter test {}: filter({}) == {}",
        i, incoming_src, expected_src
      );

      let result = filter.filter(incoming_src.into());

      assert_eq!(result, expected_src.to_string());
      assert_eq!(filter(incoming_src.into()), result);
    }
  }
}

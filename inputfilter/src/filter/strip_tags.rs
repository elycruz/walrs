use std::borrow::Cow;
use std::sync::OnceLock;
use ammonia;

static DEFAULT_AMMONIA_BUILDER: OnceLock<ammonia::Builder> = OnceLock::new();

/// Sanitizes incoming HTML using the [Ammonia](https://docs.rs/ammonia/1.0.0/ammonia/) crate.
///
/// ```rust
/// use walrs_inputfilter::filter::StripTags;
/// use std::borrow::Cow;
///
/// let filter = StripTags::new();
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
pub struct StripTags<'a> {
  /// Ammonia builder used to sanitize incoming HTML.
  ///
  /// If `None`, a default builder is used when `filter`/instance is called.
  pub ammonia: Option<ammonia::Builder<'a>>,
}

impl<'a> StripTags<'a> {
  /// Constructs a new `StripTags` instance.
  pub fn new() -> Self {
    Self {
      ammonia: None,
    }
  }

  /// Filters incoming HTML using the contained `ammonia::Builder` instance.
  ///  If no instance is set gets/(and/or) initializes a new (default, and singleton) instance.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use std::sync::OnceLock;
  /// use ammonia::Builder as AmmoniaBuilder;
  /// use walrs_inputfilter::filter::StripTags;
  ///
  /// // Using default settings:
  /// let filter = StripTags::new();
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
  /// let filter = StripTags {
  ///   ammonia: Some(sanitizer)
  /// };
  ///
  /// // Notice `style` tags are no longer removed.
  /// assert_eq!(filter.filter(
  ///     "<script>alert('hello');</script><style>p { font-weight: bold; }</style>".into()
  ///   ),
  ///   "<style>p { font-weight: bold; }</style>"
  /// );
  /// ```
  ///
  pub fn filter<'b>(&self, input: Cow<'b, str>) -> Cow<'b, str> {
    match self.ammonia {
      None => Cow::Owned(
        DEFAULT_AMMONIA_BUILDER.get_or_init(ammonia::Builder::default)
          .clean(&input).to_string()
      ),
      Some(ref sanitizer) => Cow::Owned(
        sanitizer.clean(&input).to_string()
      ),
    }
  }
}

impl<'a> Default for StripTags<'a> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'a, 'b> FnOnce<(Cow<'b, str>, )> for StripTags<'a> {
  type Output = Cow<'b, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'b, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a, 'b> FnMut<(Cow<'b, str>, )> for StripTags<'a> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'b, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a, 'b> Fn<(Cow<'b, str>, )> for StripTags<'a> {
  extern "rust-call" fn call(&self, args: (Cow<'b, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_construction() {
    let _ = StripTags::new();
    let _ = StripTags {
      ammonia: Some(ammonia::Builder::default()),
    };
  }

  #[test]
  fn test_filter() {
    let filter = StripTags::new();

    for (i, (incoming_src, expected_src)) in [
      ("", ""),
      ("Socrates'", "Socrates'"),
      ("\"Hello\"", "\"Hello\""),
      ("Hello", "Hello"),
      ("<script>alert(\"Hello World\");</script>", ""),        // Removes `script` tags, by default
      ("<p>The quick brown fox</p><style>p { font-weight: bold; }</style>",
       "<p>The quick brown fox</p>"),                          // Removes `style` tags, by default
      ("<p>The quick brown fox", "<p>The quick brown fox</p>") // Fixes erroneous markup
    ]
      .into_iter().enumerate() {
      println!("Filter test {}: filter({}) == {}", i, incoming_src, expected_src);

      let result = filter.filter(incoming_src.into());

      assert_eq!(result, expected_src.to_string());
      assert_eq!(filter(incoming_src.into()), result);
    }
  }
}

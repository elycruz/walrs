use std::borrow::Cow;
use std::sync::OnceLock;

use crate::Filter;

static DEFAULT_AMMONIA_BUILDER: OnceLock<ammonia::Builder> = OnceLock::new();

/// Sanitizes incoming HTML using the [Ammonia](https://docs.rs/ammonia/1.0.0/ammonia/) crate.
///
/// ```rust
/// use walrs_inputfilter::filters::{Filter, StripTagsFilter};
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
///  }
/// ```
///
#[must_use]
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
}

impl Filter<Cow<'_, str>> for StripTagsFilter<'_> {
  type Output = Cow<'static, str>;

  /// Filters incoming HTML using the contained `ammonia::Builder` instance.
  ///  If no instance is set gets/(and/or) initializes a new (default, and singleton) instance.
  ///
  /// ```rust
  /// use std::borrow::Cow;
  /// use std::sync::OnceLock;
  /// use ammonia::Builder as AmmoniaBuilder;
  /// use walrs_inputfilter::filters::{Filter, StripTagsFilter};
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
  /// ```
  ///
  fn filter(&self, input: Cow<'_, str>) -> Self::Output {
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

#[cfg(feature = "fn_traits")]
impl FnOnce<(Cow<'_, str>,)> for StripTagsFilter<'_> {
  type Output = Cow<'static, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'_, str>,)) -> Self::Output {
    Filter::filter(&self, args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl FnMut<(Cow<'_, str>,)> for StripTagsFilter<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'_, str>,)) -> Self::Output {
    Filter::filter(self, args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl Fn<(Cow<'_, str>,)> for StripTagsFilter<'_> {
  extern "rust-call" fn call(&self, args: (Cow<'_, str>,)) -> Self::Output {
    Filter::filter(self, args.0)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::thread;

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
    }
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_fn_traits() {
    let filter = StripTagsFilter::new();
    assert_eq!(filter("Hello".into()), "Hello".to_string());
  }

  #[test]
  fn test_filter_in_threaded_context() {
    let filter = StripTagsFilter::new();

    thread::scope(|scope| {
      scope.spawn(|| {
        assert_eq!(filter.filter("Hello".into()), "Hello");
        assert_eq!(
          filter.filter("<script>alert('hello');</script>".into()),
          ""
        );
        assert_eq!(
          filter.filter(
            "<p>The quick brown fox</p><style>p { font-weight: bold; }</style>".into()
          ),
          "<p>The quick brown fox</p>"
        );
      });
    });
  }

  #[test]
  fn test_filter_with_custom_ammonia_in_threaded_context() {
    let mut sanitizer = ammonia::Builder::default();
    let additional_allowed_tags = vec!["style"];

    sanitizer
      .add_tags(&additional_allowed_tags)
      .rm_clean_content_tags(&additional_allowed_tags);

    let filter = StripTagsFilter {
      ammonia: Some(sanitizer),
    };

    thread::scope(|scope| {
      scope.spawn(|| {
        assert_eq!(
          filter.filter(
            "<script>alert('hello');</script><style>p { font-weight: bold; }</style>".into()
          ),
          "<style>p { font-weight: bold; }</style>"
        );
      });
    });
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_fn_traits_in_threaded_context() {
    let filter = StripTagsFilter::new();

    thread::scope(|scope| {
      scope.spawn(move || {
        assert_eq!(filter("Hello".into()), "Hello".to_string());
        assert_eq!(
          filter("<script>alert('hello');</script>".into()),
          "".to_string()
        );
      });
    });
  }
}

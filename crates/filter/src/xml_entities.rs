use crate::Filter;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

static DEFAULT_CHARS_ASSOC_MAP: OnceLock<HashMap<char, &'static str>> = OnceLock::new();

/// Encodes >, <, &, ', and " as XML entities.
///
/// Note: This filter does not skip already (XML) encoded characters;
///   E.g., the `&` in `&amp;` will get encoded as well resulting in the value `&amp;amp;`.
///
/// @todo Update algorithm to skip over existing XML entity declarations, or use a third-party lib.;
///   E.g., ignore results like `&amp;amp;` for string `&amp;`, etc.
///
/// ```rust
/// use walrs_filter::{Filter, XmlEntitiesFilter};
///
/// let filter = XmlEntitiesFilter::new();
///
/// for (incoming_src, expected_src) in [
///  ("", ""),
///  ("Socrates'", "Socrates&apos;"),
///  ("\"Hello\"", "&quot;Hello&quot;"),
///  ("Hello", "Hello"),
///  ("S & P", "S &amp; P"),
///  ("S &amp; P", "S &amp;amp; P"),
///  ("<script>alert('hello');</script>", "&lt;script&gt;alert(&apos;hello&apos;);&lt;/script&gt;"),
/// ] {
///  assert_eq!(filter.filter(incoming_src.into()), expected_src.to_string());
/// }
/// ```
#[derive(Clone, Debug)]
#[must_use]
pub struct XmlEntitiesFilter<'a> {
  pub chars_assoc_map: &'a HashMap<char, &'static str>,
}

impl XmlEntitiesFilter<'_> {
  pub fn new() -> Self {
    Self {
      chars_assoc_map: DEFAULT_CHARS_ASSOC_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert('<', "&lt;");
        map.insert('>', "&gt;");
        map.insert('"', "&quot;");
        map.insert('\'', "&apos;");
        map.insert('&', "&amp;");
        map
      }),
    }
  }
}

impl<'a> Filter<Cow<'a, str>> for XmlEntitiesFilter<'_> {
  type Output = Cow<'a, str>;

  /// Uses contained character association map to encode characters matching contained characters as
  /// xml entities.
  ///
  /// ```rust
  /// use walrs_filter::{Filter, XmlEntitiesFilter};
  ///
  /// let filter = XmlEntitiesFilter::new();
  ///
  /// for (incoming_src, expected_src) in [
  ///   ("", ""),
  ///   (" ", " "),
  ///   ("Socrates'", "Socrates&apos;"),
  ///   ("\"Hello\"", "&quot;Hello&quot;"),
  ///   ("Hello", "Hello"),
  ///   ("<", "&lt;"),
  ///   (">", "&gt;"),
  ///   ("&", "&amp;"),
  ///   ("<script></script>", "&lt;script&gt;&lt;/script&gt;"),
  /// ] {
  ///   assert_eq!(filter.filter(incoming_src.into()), expected_src.to_string());
  /// }
  ///```
  fn filter(&self, input: Cow<'a, str>) -> Self::Output {
    // Fast path: if no characters need encoding, return input as-is (zero-copy)
    if !input.chars().any(|c| self.chars_assoc_map.contains_key(&c)) {
      return input;
    }

    let mut output = String::with_capacity(input.len() + input.len() / 5 * 3);
    for c in input.chars() {
      match self.chars_assoc_map.get(&c) {
        Some(entity) => output.push_str(entity),
        None => output.push(c),
      }
    }

    Cow::Owned(output)
  }
}

impl Default for XmlEntitiesFilter<'_> {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature = "fn_traits")]
impl<'a> FnOnce<(Cow<'a, str>,)> for XmlEntitiesFilter<'_> {
  type Output = Cow<'a, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'a, str>,)) -> Self::Output {
    Filter::filter(&self, args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<'a> FnMut<(Cow<'a, str>,)> for XmlEntitiesFilter<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'a, str>,)) -> Self::Output {
    Filter::filter(self, args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<'a> Fn<(Cow<'a, str>,)> for XmlEntitiesFilter<'_> {
  extern "rust-call" fn call(&self, args: (Cow<'a, str>,)) -> Self::Output {
    Filter::filter(self, args.0)
  }
}

#[cfg(test)]
mod test {
  use super::super::traits::Filter;

  #[test]
  fn test_construction() {
    let _ = super::XmlEntitiesFilter::new();
  }

  #[test]
  fn test_filter() {
    let filter = super::XmlEntitiesFilter::new();

    for (incoming_src, expected_src) in [
      ("", ""),
      ("Socrates'", "Socrates&apos;"),
      ("\"Hello\"", "&quot;Hello&quot;"),
      ("Hello", "Hello"),
      ("<", "&lt;"),
      (">", "&gt;"),
      ("&", "&amp;"),
      (
        "<script>alert('hello');</script>",
        "&lt;script&gt;alert(&apos;hello&apos;);&lt;/script&gt;",
      ),
    ] {
      assert_eq!(filter.filter(incoming_src.into()), expected_src.to_string());
    }
  }

  #[test]
  fn test_noop_no_encodable_chars() {
    let filter = super::XmlEntitiesFilter::new();

    // These inputs have no encodable characters — should be zero-copy no-op
    for input in ["Hello", "Hello World", "abc123", "", " "] {
      let cow_input = std::borrow::Cow::Borrowed(input);
      let result = filter.filter(cow_input);
      assert_eq!(result, input);
      assert!(
        matches!(result, std::borrow::Cow::Borrowed(_)),
        "Expected Cow::Borrowed for no-op input {:?}",
        input
      );
    }
  }

  #[test]
  fn test_noop_reuses_owned_input() {
    let filter = super::XmlEntitiesFilter::new();

    // When input is Cow::Owned and no-op, should reuse the owned String
    let input = "Hello World".to_string();
    let result = filter.filter(std::borrow::Cow::Owned(input));
    assert_eq!(result, "Hello World");
    assert!(matches!(result, std::borrow::Cow::Owned(_)));
  }

  #[test]
  fn test_entity_heavy_string() {
    let filter = super::XmlEntitiesFilter::new();

    // Every character requires entity expansion
    let input = "<>&'\"<>&'\"";
    let expected = "&lt;&gt;&amp;&apos;&quot;&lt;&gt;&amp;&apos;&quot;";
    assert_eq!(filter.filter(input.into()), expected);

    // All ampersands
    let input = "&&&&&";
    let expected = "&amp;&amp;&amp;&amp;&amp;";
    assert_eq!(filter.filter(input.into()), expected);
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_fn_traits() {
    let filter = super::XmlEntitiesFilter::new();
    assert_eq!(filter("Hello".into()), "Hello".to_string());
  }
}

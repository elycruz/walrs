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
/// use walrs_inputfilter::filters::{Filter, XmlEntitiesFilter};
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
///  assert_eq!(filter(incoming_src.into()), expected_src.to_string());
/// }
/// ```
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

impl Filter<Cow<'_, str>> for XmlEntitiesFilter<'_> {
  /// Uses contained character association map to encode characters matching contained characters as
  /// xml entities.
  ///
  /// ```rust
  /// use walrs_inputfilter::filters::{Filter, XmlEntitiesFilter};
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
  fn filter<'b>(&self, input: Cow<'b, str>) -> Cow<'b, str> {
    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
      match self.chars_assoc_map.get(&c) {
        Some(entity) => output.push_str(entity),
        None => output.push(c),
      }
    }

    Cow::Owned(output.to_string())
  }
}

impl Default for XmlEntitiesFilter<'_> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'b> FnOnce<(Cow<'b, str>,)> for XmlEntitiesFilter<'_> {
  type Output = Cow<'b, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'b, str>,)) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'b> FnMut<(Cow<'b, str>,)> for XmlEntitiesFilter<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'b, str>,)) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'b> Fn<(Cow<'b, str>,)> for XmlEntitiesFilter<'_> {
  extern "rust-call" fn call(&self, args: (Cow<'b, str>,)) -> Self::Output {
    self.filter(args.0)
  }
}

#[cfg(test)]
mod test {
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
      assert_eq!(filter(incoming_src.into()), expected_src.to_string());
    }
  }
}

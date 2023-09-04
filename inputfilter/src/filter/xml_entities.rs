use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

static DEFAULT_CHARS_ASSOC_MAP: OnceLock<HashMap<char, &'static str>> = OnceLock::new();

/// Encodes >, <, &, ', and " as XML entities.
pub struct XmlEntitiesFilter<'a> {
  pub chars_assoc_map: &'a HashMap<char, &'static str>,
}

impl<'a> XmlEntitiesFilter<'a> {
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
      })
    }
  }

  /// Uses contained character association map to encode characters matching contained characters as
  /// xml entities.
  ///
  /// ```rust
  /// use walrs_inputfilter::filter::XmlEntitiesFilter;
  ///
  /// let filter = XmlEntitiesFilter::new();
  ///
  /// for (incoming_src, expected_src) in [
  ///   ("", ""),
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
  pub fn filter<'b>(&self, input: Cow<'b, str>) -> Cow<'b, str> {
    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
      match self.chars_assoc_map.get(&c) {
        Some(entity) => output.push_str(entity),
        None => output.push(c),
      }
    }

    Cow::Owned(output.trim().to_string())
  }
}

impl<'a, 'b> FnOnce<(Cow<'b, str>, )> for XmlEntitiesFilter<'a> {
  type Output = Cow<'b, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'b, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl <'a, 'b> FnMut<(Cow<'b, str>, )> for XmlEntitiesFilter<'a> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'b, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl <'a, 'b> Fn<(Cow<'b, str>, )> for XmlEntitiesFilter<'a> {
  extern "rust-call" fn call(&self, args: (Cow<'b, str>, )) -> Self::Output {
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
      ("<script></script>", "&lt;script&gt;&lt;/script&gt;"),
    ] {
      assert_eq!(filter(incoming_src.into()), expected_src.to_string());
    }
  }
}

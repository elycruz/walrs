use crate::Filter;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

static DEFAULT_CHARS_ASSOC_MAP: OnceLock<HashMap<char, &'static str>> = OnceLock::new();

/// Maximum length of the name portion of a named entity (e.g. `amp`, `CounterClockwiseContourIntegral`).
const MAX_NAMED_ENTITY_LEN: usize = 32;

/// Maximum digits recognized in a decimal numeric character reference (`&#NNN;`).
const MAX_DECIMAL_ENTITY_DIGITS: usize = 10;

/// Maximum digits recognized in a hexadecimal numeric character reference (`&#xHHH;`).
const MAX_HEX_ENTITY_DIGITS: usize = 8;

/// Detects a well-formed HTML/XML entity reference starting at `bytes[start]`.
///
/// Returns the byte index immediately after the terminating `;` when a valid entity
/// is found, otherwise `None`. Only lexical validation is performed; the entity name
/// itself is not checked against a dictionary of known entities. This is intentional —
/// the filter's goal is to avoid double-encoding syntactically valid entity references
/// regardless of whether they resolve to a named character.
fn scan_entity(bytes: &[u8], start: usize) -> Option<usize> {
  if bytes.get(start).copied() != Some(b'&') {
    return None;
  }
  let mut j = start + 1;
  let first = *bytes.get(j)?;

  if first == b'#' {
    j += 1;
    let next = *bytes.get(j)?;
    let is_hex = next == b'x' || next == b'X';
    if is_hex {
      j += 1;
    }
    let digits_start = j;

    if is_hex {
      while j < bytes.len()
        && bytes[j].is_ascii_hexdigit()
        && (j - digits_start) < MAX_HEX_ENTITY_DIGITS
      {
        j += 1;
      }
    } else {
      while j < bytes.len()
        && bytes[j].is_ascii_digit()
        && (j - digits_start) < MAX_DECIMAL_ENTITY_DIGITS
      {
        j += 1;
      }
    }

    if j > digits_start && bytes.get(j).copied() == Some(b';') {
      return Some(j + 1);
    }
    None
  } else if first.is_ascii_alphabetic() {
    let name_start = j;
    while j < bytes.len()
      && bytes[j].is_ascii_alphanumeric()
      && (j - name_start) < MAX_NAMED_ENTITY_LEN
    {
      j += 1;
    }
    if j > name_start && bytes.get(j).copied() == Some(b';') {
      return Some(j + 1);
    }
    None
  } else {
    None
  }
}

/// Encodes `<`, `>`, `&`, `'`, and `"` as XML/HTML entities.
///
/// Existing entity references in the input are preserved verbatim so the filter never
/// double-encodes. Recognized forms:
/// - Named entities: `&name;` where `name` is ASCII alphanumeric (e.g. `&amp;`, `&lt;`, `&copy;`).
/// - Decimal numeric: `&#NNNN;` (e.g. `&#39;`).
/// - Hexadecimal numeric: `&#xHHHH;` / `&#XHHHH;` (e.g. `&#x2F;`).
///
/// Malformed entity-like sequences (e.g. `&foo`, `&;`, `&#;`) are treated as raw
/// characters and the leading `&` is encoded as `&amp;`.
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
///  // Already-encoded entities are preserved (no double-encoding).
///  ("S &amp; P", "S &amp; P"),
///  ("Tom &#38; Jerry", "Tom &#38; Jerry"),
///  ("AT&T", "AT&amp;T"),
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

  /// Returns true when any character in `input` requires encoding. Existing entity
  /// references are skipped so inputs composed entirely of already-encoded text
  /// take the zero-copy fast path.
  fn needs_encoding(&self, input: &str) -> bool {
    let bytes = input.as_bytes();
    let encodes_ampersand = self.chars_assoc_map.contains_key(&'&');
    let mut i = 0;
    while i < bytes.len() {
      let b = bytes[i];
      if b == b'&' {
        if let Some(end) = scan_entity(bytes, i) {
          i = end;
          continue;
        }
        if encodes_ampersand {
          return true;
        }
        i += 1;
        continue;
      }
      if b < 0x80 {
        if self.chars_assoc_map.contains_key(&(b as char)) {
          return true;
        }
        i += 1;
      } else {
        let ch = input[i..].chars().next().unwrap();
        if self.chars_assoc_map.contains_key(&ch) {
          return true;
        }
        i += ch.len_utf8();
      }
    }
    false
  }
}

impl<'a> Filter<Cow<'a, str>> for XmlEntitiesFilter<'_> {
  type Output = Cow<'a, str>;

  /// Encodes special characters as XML/HTML entities while preserving any existing
  /// entity references in the input so the filter never double-encodes.
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
  ///   ("&amp;", "&amp;"),
  ///   ("&#x2F;", "&#x2F;"),
  ///   ("<script></script>", "&lt;script&gt;&lt;/script&gt;"),
  /// ] {
  ///   assert_eq!(filter.filter(incoming_src.into()), expected_src.to_string());
  /// }
  ///```
  fn filter(&self, input: Cow<'a, str>) -> Self::Output {
    if !self.needs_encoding(&input) {
      return input;
    }

    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len() + input.len() / 5 * 3);
    let mut i = 0;
    while i < bytes.len() {
      let b = bytes[i];
      if b == b'&' {
        if let Some(end) = scan_entity(bytes, i) {
          output.push_str(&input[i..end]);
          i = end;
          continue;
        }
        match self.chars_assoc_map.get(&'&') {
          Some(entity) => output.push_str(entity),
          None => output.push('&'),
        }
        i += 1;
      } else if b < 0x80 {
        let ch = b as char;
        match self.chars_assoc_map.get(&ch) {
          Some(entity) => output.push_str(entity),
          None => output.push(ch),
        }
        i += 1;
      } else {
        let ch = input[i..].chars().next().unwrap();
        let ch_len = ch.len_utf8();
        match self.chars_assoc_map.get(&ch) {
          Some(entity) => output.push_str(entity),
          None => output.push_str(&input[i..i + ch_len]),
        }
        i += ch_len;
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

  // ---- Entity-preserving behavior (issue #233) ----

  #[test]
  fn test_preserves_existing_named_entities() {
    let filter = super::XmlEntitiesFilter::new();

    for (input, expected) in [
      ("&amp;", "&amp;"),
      ("&lt;", "&lt;"),
      ("&gt;", "&gt;"),
      ("&quot;", "&quot;"),
      ("&apos;", "&apos;"),
      ("&copy;", "&copy;"),
      ("&nbsp;", "&nbsp;"),
      ("Tom &amp; Jerry", "Tom &amp; Jerry"),
      ("&amp;&lt;&gt;&quot;&apos;", "&amp;&lt;&gt;&quot;&apos;"),
    ] {
      assert_eq!(
        filter.filter(input.into()),
        expected,
        "input was {:?}",
        input
      );
    }
  }

  #[test]
  fn test_preserves_numeric_decimal_entities() {
    let filter = super::XmlEntitiesFilter::new();

    for (input, expected) in [
      ("&#38;", "&#38;"),
      ("&#39;", "&#39;"),
      ("&#60;", "&#60;"),
      ("&#169;", "&#169;"),
      ("Tom &#38; Jerry", "Tom &#38; Jerry"),
    ] {
      assert_eq!(filter.filter(input.into()), expected);
    }
  }

  #[test]
  fn test_preserves_numeric_hex_entities() {
    let filter = super::XmlEntitiesFilter::new();

    for (input, expected) in [
      ("&#x2F;", "&#x2F;"),
      ("&#X2F;", "&#X2F;"),
      ("&#x26;", "&#x26;"),
      ("&#xA9;", "&#xA9;"),
      ("path &#x2F; segment", "path &#x2F; segment"),
    ] {
      assert_eq!(filter.filter(input.into()), expected);
    }
  }

  #[test]
  fn test_mixed_raw_and_encoded() {
    let filter = super::XmlEntitiesFilter::new();

    // Raw specials around an existing entity — only the raw ones get encoded.
    assert_eq!(
      filter.filter("<b>Tom &amp; Jerry</b>".into()),
      "&lt;b&gt;Tom &amp; Jerry&lt;/b&gt;"
    );

    // Ampersand followed by a space: bare `&`, should be encoded.
    assert_eq!(filter.filter("AT&T".into()), "AT&amp;T");
    assert_eq!(filter.filter("foo & bar".into()), "foo &amp; bar");

    // Mix of encoded and raw ampersands.
    assert_eq!(
      filter.filter("&amp; AT&T &#38;".into()),
      "&amp; AT&amp;T &#38;"
    );
  }

  #[test]
  fn test_malformed_entity_like_sequences_are_encoded() {
    let filter = super::XmlEntitiesFilter::new();

    // Missing terminating `;`.
    assert_eq!(filter.filter("&amp".into()), "&amp;amp");
    // Empty name.
    assert_eq!(filter.filter("&;".into()), "&amp;;");
    // `#` with no digits.
    assert_eq!(filter.filter("&#;".into()), "&amp;#;");
    // Hex marker with no digits.
    assert_eq!(filter.filter("&#x;".into()), "&amp;#x;");
    // Digits interrupted by non-digit.
    assert_eq!(filter.filter("&#3A;".into()), "&amp;#3A;");
    // Trailing lone `&`.
    assert_eq!(filter.filter("a & b &".into()), "a &amp; b &amp;");
    // `&` followed by non-alphabetic.
    assert_eq!(filter.filter("& foo".into()), "&amp; foo");
  }

  #[test]
  fn test_already_encoded_input_is_noop() {
    let filter = super::XmlEntitiesFilter::new();

    // Fully encoded input should take the zero-copy fast path.
    for input in [
      "&amp;",
      "Tom &amp; Jerry",
      "&lt;script&gt;&lt;/script&gt;",
      "&#39;hello&#39;",
      "&#x2F;path&#x2F;",
    ] {
      let cow_input = std::borrow::Cow::Borrowed(input);
      let result = filter.filter(cow_input);
      assert_eq!(result, input);
      assert!(
        matches!(result, std::borrow::Cow::Borrowed(_)),
        "Expected Cow::Borrowed for already-encoded input {:?}",
        input
      );
    }
  }

  #[test]
  fn test_unicode_passthrough() {
    let filter = super::XmlEntitiesFilter::new();

    // Non-ASCII characters with no encodable specials — zero-copy.
    let input = "héllo wörld — naïve";
    let result = filter.filter(std::borrow::Cow::Borrowed(input));
    assert_eq!(result, input);
    assert!(matches!(result, std::borrow::Cow::Borrowed(_)));

    // Non-ASCII mixed with encodable characters.
    assert_eq!(
      filter.filter("héllo <b>wörld</b>".into()),
      "héllo &lt;b&gt;wörld&lt;/b&gt;"
    );
  }

  #[test]
  fn test_entity_at_string_boundary() {
    let filter = super::XmlEntitiesFilter::new();

    // Bare `&` at end of input must be encoded.
    assert_eq!(filter.filter("foo&".into()), "foo&amp;");
    // `&` followed by `#` at end of input — malformed.
    assert_eq!(filter.filter("foo&#".into()), "foo&amp;#");
    assert_eq!(filter.filter("foo&#x".into()), "foo&amp;#x");
  }

  #[test]
  fn test_scan_entity_rejects_overlong_names() {
    use super::scan_entity;

    // Name longer than MAX_NAMED_ENTITY_LEN (32) — should not match.
    let long = format!("&{};", "a".repeat(40));
    assert_eq!(scan_entity(long.as_bytes(), 0), None);

    // Name of exactly 32 chars — accepted.
    let ok = format!("&{};", "a".repeat(32));
    assert_eq!(scan_entity(ok.as_bytes(), 0), Some(ok.len()));
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_fn_traits() {
    let filter = super::XmlEntitiesFilter::new();
    assert_eq!(filter("Hello".into()), "Hello".to_string());
  }
}

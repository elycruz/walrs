use crate::Filter;
use regex::Regex;
use std::borrow::Cow;
use std::sync::OnceLock;

static SLUG_FILTER_REGEX: OnceLock<Regex> = OnceLock::new();
static SLUG_FILTER_REGEX_STR: &str = r"(?i)[^\w\-]";
static DASH_FILTER_REGEX: OnceLock<Regex> = OnceLock::new();
static DASH_FILTER_REGEX_STR: &str = r"(?i)\-{2,}";

/// Returns the static regex used for filtering a string to slug.
pub fn get_slug_filter_regex() -> &'static Regex {
  SLUG_FILTER_REGEX.get_or_init(|| Regex::new(SLUG_FILTER_REGEX_STR).unwrap())
}

/// Returns the static regex used for filtering out multiple dashes for one dash.
pub fn get_dash_filter_regex() -> &'static Regex {
  DASH_FILTER_REGEX.get_or_init(|| Regex::new(DASH_FILTER_REGEX_STR).unwrap())
}

/// Normalizes given string into a slug - e.g., a string matching /^\w[\w\-]{0,198}\w?$/
///
/// ```rust
/// use std::borrow::Cow;
/// use walrs_filter::slug::to_slug;
///
/// assert_eq!(to_slug(Cow::Borrowed("Hello World")), "hello-world");
/// ```
pub fn to_slug(xs: Cow<'_, str>) -> Cow<'static, str> {
  _to_slug(get_slug_filter_regex(), 200, xs)
}

/// Same as `to_slug` method but removes duplicate '-' symbols.
///
/// ```rust
/// use std::borrow::Cow;
/// use walrs_filter::slug::to_pretty_slug;
///
/// assert_eq!(to_pretty_slug(Cow::Borrowed("%$Hello@#$@#!(World$$")), "hello-world");
/// ```
pub fn to_pretty_slug(xs: Cow<'_, str>) -> Cow<'static, str> {
  _to_pretty_slug(get_slug_filter_regex(), 200, xs)
}

/// Returns `true` if the input is already a valid slug for the given parameters.
fn is_valid_slug(s: &str, max_length: usize, allow_duplicate_dashes: bool) -> bool {
  if s.is_empty() || s.len() > max_length {
    return false;
  }
  if s.starts_with('-') || s.ends_with('-') {
    return false;
  }
  let mut prev_dash = false;
  for c in s.chars() {
    if c == '-' {
      if !allow_duplicate_dashes && prev_dash {
        return false;
      }
      prev_dash = true;
    } else {
      prev_dash = false;
      if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return false;
      }
    }
  }
  true
}

fn _to_slug(pattern: &Regex, max_length: usize, xs: Cow<'_, str>) -> Cow<'static, str> {
  // Fast path: if already a valid slug, return as-is
  if is_valid_slug(&xs, max_length, true) {
    return Cow::Owned(xs.into_owned());
  }

  let rslt = pattern
    .replace_all(xs.as_ref(), "-")
    .to_lowercase()
    .trim_matches('-')
    .to_string();

  if rslt.len() > max_length {
    Cow::Owned(rslt[..max_length + 1].to_string())
  } else {
    Cow::Owned(rslt)
  }
}

fn _to_pretty_slug(pattern: &Regex, max_length: usize, xs: Cow<'_, str>) -> Cow<'static, str> {
  if xs.is_empty() {
    return Cow::Owned(String::new());
  }

  // Fast path: if already a valid pretty slug, return as-is
  if is_valid_slug(&xs, max_length, false) {
    return Cow::Owned(xs.into_owned());
  }

  get_dash_filter_regex()
    .replace_all(&_to_slug(pattern, max_length, xs), "-")
    .to_string()
    .into()
}

/// Configurable version of `to_slug()` - allows for setting the max_length.
#[must_use]
#[derive(Clone, Debug, Default, Builder)]
pub struct SlugFilter {
  #[builder(setter(into), default = "200")]
  pub max_length: usize,

  #[builder(setter(into), default = "true")]
  pub allow_duplicate_dashes: bool,
}

impl SlugFilter {
  pub fn new(max_length: usize, allow_duplicate_dashes: bool) -> Self {
    SlugFilter {
      max_length,
      allow_duplicate_dashes,
    }
  }
}

impl Filter<Cow<'_, str>> for SlugFilter {
  type Output = Cow<'static, str>;

  fn filter(&self, xs: Cow<'_, str>) -> Self::Output {
    if self.allow_duplicate_dashes {
      _to_slug(get_slug_filter_regex(), self.max_length, xs)
    } else {
      _to_pretty_slug(get_slug_filter_regex(), self.max_length, xs)
    }
  }
}

#[cfg(feature = "fn_traits")]
impl FnOnce<(Cow<'_, str>,)> for SlugFilter {
  type Output = Cow<'static, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'_, str>,)) -> Self::Output {
    Filter::filter(&self, args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl Fn<(Cow<'_, str>,)> for SlugFilter {
  extern "rust-call" fn call(&self, args: (Cow<'_, str>,)) -> Self::Output {
    Filter::filter(self, args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl FnMut<(Cow<'_, str>,)> for SlugFilter {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'_, str>,)) -> Self::Output {
    Filter::filter(self, args.0)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::thread;

  #[test]
  fn test_to_slug_standalone_method() {
    for (cow_str, expected) in [
      (Cow::Borrowed("Hello World"), "hello-world"),
      (Cow::Borrowed("#$@#$Hello World$@#$"), "hello-world"),
      (Cow::Borrowed("$Hello'\"@$World$"), "hello----world"),
    ] {
      assert_eq!(to_slug(cow_str), expected);
    }
  }

  #[test]
  fn test_to_pretty_slug_standalone_method() {
    for (cow_str, expected) in [
      (Cow::Borrowed("Hello World"), "hello-world"),
      (Cow::Borrowed("$Hello World$"), "hello-world"),
      (Cow::Borrowed("$Hello'\"@$World$"), "hello-world"),
    ] {
      assert_eq!(to_pretty_slug(cow_str), expected);
    }
  }

  #[test]
  fn test_slug_filter_constructor() {
    for x in [0, 1, 2] {
      let instance = SlugFilter::new(x, false);
      assert_eq!(instance.max_length, x);
      assert!(!instance.allow_duplicate_dashes);
    }
  }

  #[test]
  fn test_slug_filter_builder() {
    let instance = SlugFilterBuilder::default().build().unwrap();
    assert_eq!(instance.max_length, 200);
    assert!(instance.allow_duplicate_dashes);
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_fn_trait_impls() {
    let slug_filter = SlugFilter {
      max_length: 200,
      allow_duplicate_dashes: true,
    };

    assert_eq!(slug_filter(Cow::Borrowed("Hello World")), "hello-world");
    assert_eq!(slug_filter(Cow::Borrowed("Hello   World")), "hello---world");
    assert_eq!(
      slug_filter(Cow::Borrowed("$@#$Hello   @World@#$@#$")),
      "hello----world"
    );
  }

  #[test]
  fn test_standalone_methods_in_threaded_contexts() {
    thread::scope(|scope| {
      scope.spawn(move || {
        assert_eq!(to_slug(Cow::Borrowed("Hello World")), "hello-world");
        assert_eq!(to_slug(Cow::Borrowed("Hello   World")), "hello---world");
        assert_eq!(
          to_pretty_slug(Cow::Borrowed("$@#$Hello@#$@#$World@#$@#$")),
          "hello-world"
        );
      });
    });
  }

  #[test]
  fn test_slug_noop_already_valid() {
    // These are already valid slugs — should be no-op
    for input in ["hello-world", "abc123", "test_slug", "a"] {
      let result = to_slug(Cow::Borrowed(input));
      assert_eq!(result, input);
    }
  }

  #[test]
  fn test_pretty_slug_noop_already_valid() {
    // These are already valid pretty slugs (no duplicate dashes)
    for input in ["hello-world", "abc123", "test_slug", "a"] {
      let result = to_pretty_slug(Cow::Borrowed(input));
      assert_eq!(result, input);
    }
  }

  #[test]
  fn test_slug_filter_noop() {
    let filter = SlugFilter::new(200, false);

    // Already a valid pretty slug
    let result = filter.filter(Cow::Borrowed("hello-world"));
    assert_eq!(result, "hello-world");
  }

  #[test]
  fn test_slug_filter_noop_reuses_owned_input() {
    let filter = SlugFilter::new(200, false);

    // When input is Cow::Owned and no-op, should reuse the owned String
    let input = "hello-world".to_string();
    let result = filter.filter(Cow::Owned(input));
    assert_eq!(result, "hello-world");
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_struct_in_threaded_contexts() {
    let slug_filter = SlugFilterBuilder::default()
      .allow_duplicate_dashes(false)
      .build()
      .unwrap();

    thread::scope(|scope| {
      scope.spawn(move || {
        assert_eq!(slug_filter(Cow::Borrowed("Hello World")), "hello-world");
        assert_eq!(slug_filter(Cow::Borrowed("Hello   World")), "hello-world");
      });
    });
  }
}

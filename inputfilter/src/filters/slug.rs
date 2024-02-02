use std::borrow::Cow;
use std::sync::OnceLock;
use regex::Regex;

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
/// use walrs_inputfilter::filters::slug::to_slug;
///
/// assert_eq!(to_slug(Cow::Borrowed("Hello World")), "hello-world");
/// ```
pub fn to_slug(xs: Cow<str>) -> Cow<str> {
  _to_slug(get_slug_filter_regex(), 200, xs)
}

/// Same as `to_slug` method but removes duplicate '-' symbols.
///
/// ```rust
/// use std::borrow::Cow;
/// use walrs_inputfilter::filters::slug::to_pretty_slug;
///
/// assert_eq!(to_pretty_slug(Cow::Borrowed("%$Hello@#$@#!(World$$")), "hello-world");
/// ```
pub fn to_pretty_slug(xs: Cow<str>) -> Cow<str> {
  _to_pretty_slug(get_slug_filter_regex(), 200, xs)
}

fn _to_slug<'a>(pattern: &Regex, max_length: usize, xs: Cow<'a, str>) -> Cow<'a, str> {
  let rslt = pattern.replace_all(xs.as_ref(), "-")
    .to_lowercase()
    .trim_matches('-')
    .to_string();

  if rslt.len() > max_length {
    Cow::Owned(rslt[..max_length + 1].to_string())
  } else {
    Cow::Owned(rslt)
  }
}

fn _to_pretty_slug<'a>(pattern: &Regex, max_length: usize, xs: Cow<'a, str>) -> Cow<'a, str> {
  if xs.is_empty() { return xs; }

  get_dash_filter_regex()
    .replace_all(&_to_slug(pattern, max_length, xs), "-")
    .to_string()
    .into()
}

/// Configurable version of `to_slug()` - allows for setting the max_length.
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

  pub fn filter<'a>(&self, xs: Cow<'a, str>) -> Cow<'a, str> {
    if self.allow_duplicate_dashes {
      _to_slug(get_slug_filter_regex(), self.max_length, xs)
    } else {
      _to_pretty_slug(get_slug_filter_regex(), self.max_length, xs)
    }
  }
}

impl<'a> FnOnce<(Cow<'a, str>, )> for SlugFilter {
  type Output = Cow<'a, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'a, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a> Fn<(Cow<'a, str>, )> for SlugFilter {
  extern "rust-call" fn call(&self, args: (Cow<'a, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a> FnMut<(Cow<'a, str>, )> for SlugFilter {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'a, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

#[cfg(test)]
mod test {
  use std::{borrow::Cow, thread};
  use super::*;

  #[test]
  fn test_to_slug_standalone_method() {
    for (cow_str, expected) in [(Cow::Borrowed("Hello World"), "hello-world"),
      (Cow::Borrowed("#$@#$Hello World$@#$"), "hello-world"),
      (Cow::Borrowed("$Hello'\"@$World$"), "hello----world")] {
      assert_eq!(to_slug(cow_str), expected);
    }
  }

  #[test]
  fn test_to_pretty_slug_standalone_method() {
    for (cow_str, expected) in [(Cow::Borrowed("Hello World"), "hello-world"),
      (Cow::Borrowed("$Hello World$"), "hello-world"),
      (Cow::Borrowed("$Hello'\"@$World$"), "hello-world")] {
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

  #[test]
  fn test_fn_trait_impls()  {
    let slug_filter = SlugFilter { max_length: 200, allow_duplicate_dashes: true };

    assert_eq!(slug_filter(Cow::Borrowed("Hello World")), "hello-world");
    assert_eq!(slug_filter(Cow::Borrowed("Hello   World")), "hello---world");
    assert_eq!(slug_filter(Cow::Borrowed("$@#$Hello   @World@#$@#$")), "hello----world");
  }

  #[test]
  fn test_standalone_methods_in_threaded_contexts() {
    thread::scope(|scope| {
      scope.spawn(move ||{
        assert_eq!(to_slug(Cow::Borrowed("Hello World")), "hello-world");
        assert_eq!(to_slug(Cow::Borrowed("Hello   World")), "hello---world");
        assert_eq!(to_pretty_slug(Cow::Borrowed("$@#$Hello@#$@#$World@#$@#$")), "hello-world");
      });
    });
  }

  #[test]
  fn test_struct_in_threaded_contexts() {
    let slug_filter = SlugFilterBuilder::default()
      .allow_duplicate_dashes(false)
      .build()
      .unwrap();

    thread::scope(|scope| {
      scope.spawn(move ||{
        assert_eq!(slug_filter(Cow::Borrowed("Hello World")), "hello-world");
        assert_eq!(slug_filter(Cow::Borrowed("Hello   World")), "hello-world");
      });
    });
  }
}

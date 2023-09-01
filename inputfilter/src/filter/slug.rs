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
/// use walrs_inputfilter::filter::slug::to_slug;
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
/// use walrs_inputfilter::filter::slug::to_pretty_slug;
///
/// assert_eq!(to_pretty_slug(Cow::Borrowed("%$Hello@#$@#!(World$$")), "hello-world");
/// ```
pub fn to_pretty_slug(xs: Cow<str>) -> Cow<str> {
  _to_pretty_slug(get_slug_filter_regex(), 200, xs)
}

pub fn _to_slug<'a, 'b>(pattern: &Regex, max_length: usize, xs: Cow<'a, str>) -> Cow<'b, str> {
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

pub fn _to_pretty_slug<'a>(pattern: &Regex, max_length: usize, xs: Cow<'a, str>) -> Cow<'a, str> {
  if xs.is_empty() { return xs.to_owned() }

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
}

impl SlugFilter {
  pub fn new(max_length: usize) -> Self {
    SlugFilter {
      max_length,
    }
  }

  pub fn filter<'a, 'b: 'a>(&self, xs: Cow<'a, str>) -> Cow<'b, str> {
    _to_slug(get_slug_filter_regex(), self.max_length, xs)
  }
}

impl<'a> FnOnce<(Cow<'a, str>, )> for SlugFilter {
  type Output = Cow<'a, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'a, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a, 'b> Fn<(Cow<'a, str>, )> for SlugFilter {
  extern "rust-call" fn call(&self, args: (Cow<'a, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a, 'b> FnMut<(Cow<'a, str>, )> for SlugFilter {
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
    for (cow_str, expected) in vec![
      (Cow::Borrowed("Hello World"), "hello-world"),
      (Cow::Borrowed("#$@#$Hello World$@#$"), "hello-world"),
      (Cow::Borrowed("$Hello'\"@$World$"), "hello----world"),
    ] {
      assert_eq!(to_slug(cow_str), expected);
    }
  }

  #[test]
  fn test_to_pretty_slug_standalone_method() {
    for (cow_str, expected) in vec![
      (Cow::Borrowed("Hello World"), "hello-world"),
      (Cow::Borrowed("$Hello World$"), "hello-world"),
      (Cow::Borrowed("$Hello'\"@$World$"), "hello-world"),
    ] {
      assert_eq!(to_pretty_slug(cow_str), expected);
    }
  }

  #[test]
  fn test_slug_filter_constructor() {
    for x in vec![0, 1, 2] {
      assert_eq!(SlugFilter::new(x).max_length, x);
    }
  }

  #[test]
  fn test_slug_filter_builder() {
    assert_eq!(SlugFilterBuilder::default().build().unwrap().max_length, 200);
  }

  #[test]
  fn test_fn_trait_impls()  {
    let slug_filter = SlugFilter { max_length: 200 };

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
    let slug_filter = SlugFilterBuilder::default().build().unwrap();

    thread::scope(|scope| {
      scope.spawn(move ||{
        assert_eq!(slug_filter(Cow::Borrowed("Hello World")), "hello-world");
        assert_eq!(slug_filter(Cow::Borrowed("Hello   World")), "hello---world");
      });
    });
  }
}

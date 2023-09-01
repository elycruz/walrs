use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::OnceLock;
use regex::Regex;

static SLUG_REGEX: OnceLock<Regex> = OnceLock::new();
static SLUG_FILTER_REGEX_STR: &str = r"(?i)[^\w\-]";

/// Returns the static regex used for filtering a string to slug.
pub fn get_slug_regex() -> &'static Regex {
  SLUG_REGEX.get_or_init(|| Regex::new(SLUG_FILTER_REGEX_STR).unwrap())
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
  _to_slug(get_slug_regex(), 200, xs)
}

pub fn _to_slug<'a, 'b>(pattern: &Regex, max_length: usize, xs: Cow<'a, str>) -> Cow<'b, str> {
  let rslt = pattern.replace_all(xs.as_ref(), "-")
    .to_lowercase()
    .trim_matches('-')
    .to_string();

  if rslt.len() > max_length {
    Cow::Owned(rslt[..201].to_string())
  } else {
    Cow::Owned(rslt)
  }
}

/// Configurable version of `to_slug()` - allows for setting the max_length.
#[derive(Clone, Debug, Default, Builder)]
pub struct SlugFilter<'a> {
  #[builder(setter(into), default = "200")]
  pub max_length: usize,

  _phantom_field: PhantomData<&'a str>
}

impl<'a> SlugFilter<'a> {
  pub fn new(max_length: usize) -> Self {
    SlugFilter {
      max_length,
      _phantom_field: Default::default(),
    }
  }

  pub fn filter<'b: 'a>(&self, xs: Cow<'a, str>) -> Cow<'b, str> {
    _to_slug(get_slug_regex(), self.max_length, xs)
  }
}

impl<'a> FnOnce<(Cow<'a, str>,)> for SlugFilter<'_> {
  type Output = Cow<'a, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'a, str>,)) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a, 'b> Fn<(Cow<'a, str>,)> for SlugFilter<'_> {
  extern "rust-call" fn call(&self, args: (Cow<'a, str>,)) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a, 'b> FnMut<(Cow<'a, str>,)> for SlugFilter<'_> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'a, str>,)) -> Self::Output {
    self.filter(args.0)
  }
}

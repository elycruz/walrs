use std::borrow::Cow;
use std::sync::OnceLock;
use regex::Regex;

static SLUG_REGEX: OnceLock<Regex> = OnceLock::new();

/// Normalizes given string into a slug - e.g., a string matching /^\w[\w\-]{0,198}\w?$/
///
/// ```rust
/// use std::borrow::Cow;
/// use walrs_inputfilter::filter::slug::to_slug;
///
/// assert_eq!(to_slug(Cow::Borrowed("Hello World")), "hello-world");
/// ```
pub fn to_slug(xs: Cow<str>) -> Cow<str> {
  let rx = SLUG_REGEX.get_or_init(|| Regex::new(r"(?i)[^\w\-]").unwrap());
  let rslt = rx.replace_all(xs.as_ref(), "-")
    .to_lowercase()
    .trim_matches('-')
    .to_string();

  if rslt.len() > 200 {
    Cow::Owned(rslt[..201].to_string())
  } else {
    Cow::Owned(rslt)
  }
}


use std::collections::{HashMap, HashSet};

pub struct StripTags {
  /// Tags to strip.
  tags: Option<Vec<String>>,

  /// Map of tags, and attributes to allow on them.
  tags_and_attribs: Option<HashMap<String, HashSet<String>>>,

  /// Attributes to strip.
  attribs: Option<Vec<String>>,

  /// Whether to strip comments or not.
  strip_comments: bool,
}

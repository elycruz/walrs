//! Example: Chaining filters
//!
//! This example demonstrates how to chain multiple filters together
//! to create a processing pipeline.
//!
//! Run with: `cargo run --example filter_chain`

use std::borrow::Cow;
use walrs_filter::{Filter, SlugFilter, StripTagsFilter, XmlEntitiesFilter};

/// A simple filter chain (example) that applies multiple filters in sequence.
struct FilterChain<'a> {
  strip_tags: StripTagsFilter<'a>,
  slug: SlugFilter,
}

impl<'a> FilterChain<'a> {
  fn new() -> Self {
    Self {
      strip_tags: StripTagsFilter::new(),
      slug: SlugFilter::new(100, false),
    }
  }

  /// Strip HTML tags and then convert to slug
  fn to_clean_slug(&self, input: &str) -> String {
    // First strip HTML tags
    let stripped = self.strip_tags.filter(Cow::Borrowed(input));
    // Then convert to slug
    self.slug.filter(stripped).into_owned()
  }
}

fn main() {
  println!("=== Filter Chain Example ===\n");

  let chain = FilterChain::new();

  let inputs = [
    "<h1>My Blog Post Title</h1>",
    "<p>Hello <b>World</b>!</p>",
    "<script>alert('xss')</script>Clean Title Here",
    "<span class=\"title\">Special & Characters</span>",
  ];

  println!("Converting HTML content to URL slugs:\n");

  for input in inputs {
    let result = chain.to_clean_slug(input);
    println!("Input:  \"{}\"", input);
    println!("Output: \"{}\"\n", result);
  }

  // Example: Preparing user input for safe display
  println!("--- Safe Display Pipeline ---\n");

  let xml_filter = XmlEntitiesFilter::new();
  let strip_filter = StripTagsFilter::new();

  let user_input = "<script>alert('hack')</script>Hello <b>World</b> & \"Friends\"";

  println!("Original user input:");
  println!("  {}\n", user_input);

  // Option 1: Strip all tags (for plain text output)
  let stripped = strip_filter.filter(Cow::Borrowed(user_input));
  println!("After StripTagsFilter (plain text):");
  println!("  {}\n", stripped);

  // Option 2: Encode as XML entities (for safe HTML display)
  let encoded = xml_filter.filter(Cow::Borrowed(user_input));
  println!("After XmlEntitiesFilter (safe HTML):");
  println!("  {}\n", encoded);

  println!("=== Example Complete ===");
}

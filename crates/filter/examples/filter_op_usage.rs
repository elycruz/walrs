//! Example: `FilterOp` enum — composable, serializable filter operations
//!
//! This example demonstrates how to use `FilterOp<T>` for config-driven
//! filter pipelines, including all string variants, numeric clamping,
//! composing operations with `Chain`, custom functions, and JSON
//! serialization/deserialization round-trips.
//!
//! Run with: `cargo run -p walrs_filter --example filter_op_usage`

use std::borrow::Cow;
use std::sync::Arc;
use walrs_filter::FilterOp;

fn main() {
  println!("=== FilterOp Usage Examples ===\n");

  // ---- Example 1: String filter variants ----
  println!("--- String filter variants ---");

  let filters: &[(&str, FilterOp<String>)] = &[
    ("Trim", FilterOp::Trim),
    ("Lowercase", FilterOp::Lowercase),
    ("Uppercase", FilterOp::Uppercase),
    ("StripTags", FilterOp::StripTags),
    ("HtmlEntities", FilterOp::HtmlEntities),
    ("Slug", FilterOp::Slug { max_length: None }),
    ("Truncate(10)", FilterOp::Truncate { max_length: 10 }),
    (
      "Replace(hello→hi)",
      FilterOp::Replace {
        from: "hello".to_string(),
        to: "hi".to_string(),
      },
    ),
  ];

  let inputs = [
    "  Hello World  ",
    "<b>Hello</b> & <i>World</i>",
    "Hello World! This is a long string.",
  ];

  for input in inputs {
    println!("\n  Input: {:?}", input);
    for (name, filter) in filters {
      let result = filter.apply_ref(input);
      println!("    {:20} -> {:?}", name, result.as_ref());
    }
  }

  println!();

  // ---- Example 2: Sanitize filter variants ----
  println!("--- Sanitize filters (Digits, Alnum, NormalizeWhitespace, UrlEncode, etc.) ---");

  let sanitizers: &[(&str, FilterOp<String>)] = &[
    ("Digits", FilterOp::Digits),
    (
      "Alnum(ws=false)",
      FilterOp::Alnum {
        allow_whitespace: false,
      },
    ),
    (
      "Alpha(ws=true)",
      FilterOp::Alpha {
        allow_whitespace: true,
      },
    ),
    ("StripNewlines", FilterOp::StripNewlines),
    ("NormalizeWhitespace", FilterOp::NormalizeWhitespace),
    (
      "AllowChars(a-z )",
      FilterOp::AllowChars {
        set: "abcdefghijklmnopqrstuvwxyz ".to_string(),
      },
    ),
    (
      "DenyChars(<>&)",
      FilterOp::DenyChars {
        set: "<>&".to_string(),
      },
    ),
    (
      "UrlEncode(rfc3986)",
      FilterOp::UrlEncode {
        encode_unreserved: false,
      },
    ),
  ];

  for input in [
    "  phone: (555) 123-4567  \n",
    "café-日本語 123!",
    "<script>alert('x')</script>",
  ] {
    println!("\n  Input: {:?}", input);
    for (name, filter) in sanitizers {
      let result = filter.apply_ref(input);
      println!("    {:20} -> {:?}", name, result.as_ref());
    }
  }

  println!();

  // ---- Example 3: apply_ref zero-copy optimization ----
  println!("--- apply_ref: Cow::Borrowed (no-op) vs Cow::Owned (mutated) ---");

  let trim = FilterOp::<String>::Trim;

  let already_trimmed = "already_trimmed";
  let result = trim.apply_ref(already_trimmed);
  match &result {
    Cow::Borrowed(_) => println!("  Trim {:?} -> Borrowed (zero-copy)", already_trimmed),
    Cow::Owned(s) => println!("  Trim {:?} -> Owned {:?}", already_trimmed, s),
  }

  let needs_trim = "  needs trimming  ";
  let result = trim.apply_ref(needs_trim);
  match &result {
    Cow::Borrowed(_) => println!("  Trim {:?} -> Borrowed", needs_trim),
    Cow::Owned(s) => println!("  Trim {:?} -> Owned {:?} (allocated)", needs_trim, s),
  }

  println!();

  // ---- Example 4: Chain multiple operations ----
  println!("--- Chain: composing multiple filters ---");

  let pipeline: FilterOp<String> = FilterOp::Chain(vec![
    FilterOp::Trim,
    FilterOp::Lowercase,
    FilterOp::Replace {
      from: " ".to_string(),
      to: "-".to_string(),
    },
    FilterOp::Truncate { max_length: 20 },
  ]);

  let inputs = ["  Hello World!  ", "  RUST PROGRAMMING IS GREAT  "];
  for input in inputs {
    let result = pipeline.apply_ref(input);
    println!("  {:30} -> {:?}", input, result.as_ref());
  }

  println!();

  // ---- Example 5: Numeric clamping ----
  println!("--- Clamp: numeric range clamping ---");

  let clamp_i32 = FilterOp::<i32>::Clamp { min: 0, max: 100 };
  for value in [-10_i32, 0, 50, 100, 150] {
    println!(
      "  Clamp<i32>(0..=100): {:5} -> {}",
      value,
      clamp_i32.apply(value)
    );
  }

  let clamp_f64 = FilterOp::<f64>::Clamp { min: 0.0, max: 1.0 };
  for value in [-0.5_f64, 0.0, 0.5, 1.0, 1.5] {
    println!(
      "  Clamp<f64>(0.0..=1.0): {:4} -> {}",
      value,
      clamp_f64.apply(value)
    );
  }

  println!();

  // ---- Example 6: Custom filter function ----
  println!("--- Custom: runtime filter function ---");

  let custom: FilterOp<String> = FilterOp::Custom(Arc::new(|s: String| {
    // Reverse the string
    s.chars().rev().collect()
  }));

  for input in ["hello", "world", "rust"] {
    let result = custom.apply(input.to_string());
    println!("  Reverse {:?} -> {:?}", input, result);
  }

  println!();

  // ---- Example 7: Serde JSON round-trip ----
  println!("--- Serde: JSON serialization/deserialization ---");

  let chain: FilterOp<String> = FilterOp::Chain(vec![
    FilterOp::Trim,
    FilterOp::Lowercase,
    FilterOp::Slug {
      max_length: Some(50),
    },
  ]);

  let json = serde_json::to_string_pretty(&chain).unwrap();
  println!("  Serialized:\n{}", json);

  let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
  let result = deserialized.apply("  Hello World!  ".to_string());
  println!("\n  Applied to \"  Hello World!  \" -> {:?}", result);

  println!();

  // Serialize a Truncate + Replace chain
  let text_pipeline: FilterOp<String> = FilterOp::Chain(vec![
    FilterOp::Trim,
    FilterOp::Replace {
      from: "foo".to_string(),
      to: "bar".to_string(),
    },
    FilterOp::Truncate { max_length: 30 },
  ]);
  let json = serde_json::to_string(&text_pipeline).unwrap();
  println!("  Truncate+Replace chain JSON: {}", json);

  let deserialized: FilterOp<String> = serde_json::from_str(&json).unwrap();
  println!(
    "  Applied: {:?}",
    deserialized.apply("  foo bar foo baz a very long sentence  ".to_string())
  );

  println!();
  println!("=== Examples Complete ===");
}

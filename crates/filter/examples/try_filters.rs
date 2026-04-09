//! Example: Fallible filter operations
//!
//! This example demonstrates how to use `TryFilterOp` for filters
//! that can legitimately fail, and how errors integrate with the
//! validation pipeline.
//!
//! Run with: `cargo run --example try_filters -p walrs_filter`

use std::sync::Arc;
use walrs_filter::{FilterError, FilterOp, TryFilterOp};

fn main() {
  println!("=== Fallible Filter Examples ===\n");

  // ---- Example 1: Lifting infallible filters ----
  println!("--- Infallible Filters in Fallible Pipeline ---");

  let trim: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
  let input = "  hello  ";
  let result = trim.try_apply(input.to_string()).unwrap();
  println!("  Trim \"{}\" -> \"{}\"", input, result);

  // Using From trait
  let lowercase: TryFilterOp<String> = FilterOp::Lowercase.into();
  let result = lowercase.try_apply("HELLO".to_string()).unwrap();
  println!("  Lowercase \"HELLO\" -> \"{}\"", result);

  println!();

  // ---- Example 2: Custom fallible filter ----
  println!("--- Custom Fallible Filter ---");

  let hex_normalize: TryFilterOp<String> = TryFilterOp::TryCustom(Arc::new(|s: String| {
    if s.chars().all(|c| c.is_ascii_hexdigit()) {
      Ok(s.to_uppercase())
    } else {
      Err(
        FilterError::new(format!("'{}' contains non-hex characters", s))
          .with_name("HexNormalize"),
      )
    }
  }));

  for input in ["abcdef", "ABCDEF", "123abc", "xyz123"] {
    let result = hex_normalize.try_apply(input.to_string());
    match &result {
      Ok(v) => println!("  HexNormalize \"{}\" -> Ok(\"{}\")", input, v),
      Err(e) => println!("  HexNormalize \"{}\" -> Err({})", input, e),
    }
  }

  println!();

  // ---- Example 3: Chaining fallible and infallible filters ----
  println!("--- Chained Fallible + Infallible Filters ---");

  let pipeline: TryFilterOp<String> = TryFilterOp::Chain(vec![
    TryFilterOp::Infallible(FilterOp::Trim),
    TryFilterOp::TryCustom(Arc::new(|s: String| {
      if s.is_empty() {
        Err(FilterError::new("value must not be empty after trimming"))
      } else {
        Ok(s)
      }
    })),
    TryFilterOp::Infallible(FilterOp::Lowercase),
  ]);

  for input in ["  HELLO  ", "  ", "  World  "] {
    let result = pipeline.try_apply(input.to_string());
    match &result {
      Ok(v) => println!("  Pipeline \"{}\" -> Ok(\"{}\")", input, v),
      Err(e) => println!("  Pipeline \"{}\" -> Err({})", input, e),
    }
  }

  println!();

  // ---- Example 4: Numeric fallible filter ----
  println!("--- Numeric Fallible Filter ---");

  let positive_only: TryFilterOp<i64> = TryFilterOp::TryCustom(Arc::new(|v: i64| {
    if v < 0 {
      Err(FilterError::new(format!("{} is negative", v)).with_name("PositiveOnly"))
    } else {
      Ok(v)
    }
  }));

  for input in [42i64, -5, 0, 100, -99] {
    let result = positive_only.try_apply(input);
    match &result {
      Ok(v) => println!("  PositiveOnly {} -> Ok({})", input, v),
      Err(e) => println!("  PositiveOnly {} -> Err({})", input, e),
    }
  }

  println!();
  println!("=== Examples Complete ===");
}

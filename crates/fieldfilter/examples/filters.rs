//! FilterOp enum examples.
//!
//! This example demonstrates the various filter operations available in the
//! `FilterOp<T>` enum for transforming values before validation.
//!
//! Run with: `cargo run --example filters`

#![allow(deprecated)]

use walrs_filter::FilterOp;

fn main() {
  println!("=== FilterOp<T> Examples ===\n");

  // Example 1: Trim filter
  println!("1. Trim filter:");
  let trim_filter = FilterOp::<String>::Trim;
  let input = "  hello world  ";
  let result = trim_filter.apply_ref(input);
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 2: Lowercase filter
  println!("\n2. Lowercase filter:");
  let lowercase_filter = FilterOp::<String>::Lowercase;
  let input = "HELLO World";
  let result = lowercase_filter.apply_ref(input);
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 3: Uppercase filter
  println!("\n3. Uppercase filter:");
  let uppercase_filter = FilterOp::<String>::Uppercase;
  let input = "hello world";
  let result = uppercase_filter.apply_ref(input);
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 4: StripTags filter
  println!("\n4. StripTags filter:");
  let strip_tags_filter = FilterOp::<String>::StripTags;
  let input = "<p>Hello <strong>World</strong>!</p>";
  let result = strip_tags_filter.apply_ref(input);
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 5: Slug filter
  println!("\n5. Slug filter:");
  let slug_filter = FilterOp::<String>::Slug { max_length: None };
  let input = "Hello World! This is a Test";
  let result = slug_filter.apply_ref(input);
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 6: Chain filter (multiple filters in sequence)
  println!("\n6. Chain filter (Trim -> Lowercase -> StripTags):");
  let chain_filter = FilterOp::<String>::Chain(vec![
    FilterOp::Trim,
    FilterOp::Lowercase,
    FilterOp::StripTags,
  ]);
  let input = "  <B>HELLO</B> World  ";
  let result = chain_filter.apply_ref(input);
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 7: Clamp filter for numeric values
  println!("\n7. Clamp filter for i32:");
  let clamp_filter = FilterOp::<i32>::Clamp { min: 0, max: 100 };
  let values = vec![-50, 0, 50, 100, 150];
  for value in values {
    let result = clamp_filter.apply(value);
    println!("   Input: {:4} -> Output: {:3}", value, result);
  }

  // Example 8: Processing email input
  println!("\n8. Practical example - Email normalization:");
  let email_normalizer = FilterOp::<String>::Chain(vec![FilterOp::Trim, FilterOp::Lowercase]);

  let emails = vec![
    "  USER@EXAMPLE.COM  ",
    "Admin@Company.ORG",
    "  support@WEBSITE.NET",
  ];

  for email in emails {
    let result = email_normalizer.apply_ref(email);
    println!("   '{}' -> '{}'", email, result);
  }

  // Example 9: Processing user input for URL slug
  println!("\n9. Practical example - Blog post slug:");
  let slug_processor = FilterOp::<String>::Chain(vec![
    FilterOp::Trim,
    FilterOp::StripTags,
    FilterOp::Slug { max_length: None },
  ]);

  let titles = vec![
    "  <h1>My First Blog Post!</h1>  ",
    "10 Tips for Better Coding",
    "What's New in Rust 2025?",
  ];

  for title in titles {
    let result = slug_processor.apply_ref(title);
    println!("   '{}'\n      -> '{}'", title, result);
  }

  println!("\n=== Examples Complete ===");
}

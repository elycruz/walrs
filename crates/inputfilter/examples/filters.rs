//! Filter enum examples.
//!
//! This example demonstrates the various filters available in the
//! `Filter<T>` enum for transforming values before validation.
//!
//! Run with: `cargo run --example filters`

use walrs_inputfilter::filter_enum::Filter;

fn main() {
  println!("=== Filter<T> Examples ===\n");

  // Example 1: Trim filter
  println!("1. Trim filter:");
  let trim_filter = Filter::<String>::Trim;
  let input = "  hello world  ".to_string();
  let result = trim_filter.apply(input.clone());
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 2: Lowercase filter
  println!("\n2. Lowercase filter:");
  let lowercase_filter = Filter::<String>::Lowercase;
  let input = "HELLO World".to_string();
  let result = lowercase_filter.apply(input.clone());
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 3: Uppercase filter
  println!("\n3. Uppercase filter:");
  let uppercase_filter = Filter::<String>::Uppercase;
  let input = "hello world".to_string();
  let result = uppercase_filter.apply(input.clone());
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 4: StripTags filter
  println!("\n4. StripTags filter:");
  let strip_tags_filter = Filter::<String>::StripTags;
  let input = "<p>Hello <strong>World</strong>!</p>".to_string();
  let result = strip_tags_filter.apply(input.clone());
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 5: Slug filter
  println!("\n5. Slug filter:");
  let slug_filter = Filter::<String>::Slug { max_length: None };
  let input = "Hello World! This is a Test".to_string();
  let result = slug_filter.apply(input.clone());
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 6: Chain filter (multiple filters in sequence)
  println!("\n6. Chain filter (Trim -> Lowercase -> StripTags):");
  let chain_filter =
    Filter::<String>::Chain(vec![Filter::Trim, Filter::Lowercase, Filter::StripTags]);
  let input = "  <B>HELLO</B> World  ".to_string();
  let result = chain_filter.apply(input.clone());
  println!("   Input:  '{}'", input);
  println!("   Output: '{}'", result);

  // Example 7: Clamp filter for numeric values
  println!("\n7. Clamp filter for i32:");
  let clamp_filter = Filter::<i32>::Clamp { min: 0, max: 100 };
  let values = vec![-50, 0, 50, 100, 150];
  for value in values {
    let result = clamp_filter.apply(value);
    println!("   Input: {:4} -> Output: {:3}", value, result);
  }

  // Example 8: Processing email input
  println!("\n8. Practical example - Email normalization:");
  let email_normalizer = Filter::<String>::Chain(vec![Filter::Trim, Filter::Lowercase]);

  let emails = vec![
    "  USER@EXAMPLE.COM  ",
    "Admin@Company.ORG",
    "  support@WEBSITE.NET",
  ];

  for email in emails {
    let result = email_normalizer.apply(email.to_string());
    println!("   '{}' -> '{}'", email, result);
  }

  // Example 9: Processing user input for URL slug
  println!("\n9. Practical example - Blog post slug:");
  let slug_processor = Filter::<String>::Chain(vec![
    Filter::Trim,
    Filter::StripTags,
    Filter::Slug { max_length: None },
  ]);

  let titles = vec![
    "  <h1>My First Blog Post!</h1>  ",
    "10 Tips for Better Coding",
    "What's New in Rust 2025?",
  ];

  for title in titles {
    let result = slug_processor.apply(title.to_string());
    println!("   '{}'\n      -> '{}'", title, result);
  }

  // Example 10: Applying filters to serde_json::Value
  println!("\n10. Filters with serde_json::Value:");
  use walrs_validation::Value;

  let value_trim = Filter::<Value>::Trim;
  let json_value = Value::String("  hello  ".to_string());
  let result = value_trim.apply(json_value.clone());
  println!("   Input:  {:?}", json_value);
  println!("   Output: {:?}", result);

  // Non-string values pass through unchanged
  let number_value = Value::Number(42.into());
  let result = value_trim.apply(number_value.clone());
  println!("\n   Input (number):  {:?}", number_value);
  println!("   Output (unchanged): {:?}", result);

  println!("\n=== Examples Complete ===");
}

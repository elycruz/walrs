//! Example: Using Attributes
//!
//! This example demonstrates the Attributes type for managing
//! HTML element attributes.
use walrs_form_core::Attributes;
fn main() {
  println!("Attributes Example");
  println!("==================\n");
  // Create from array
  let attrs = Attributes::from([
    ("class", "form-control"),
    ("id", "email-input"),
    ("placeholder", "Enter your email"),
  ]);
  println!("Created from array:");
  println!("  HTML: {}\n", attrs.to_html());
  // Create and build dynamically
  let mut attrs = Attributes::new();
  attrs.insert("type", "text");
  attrs.insert("name", "username");
  attrs.insert("required", "");
  attrs.insert("data-validate", "true");
  println!("Built dynamically:");
  println!("  HTML: {}\n", attrs.to_html());
  // Access individual attributes
  println!("Individual access:");
  println!("  name: {:?}", attrs.get("name"));
  println!("  type: {:?}", attrs.get("type"));
  println!("  nonexistent: {:?}", attrs.get("nonexistent"));
  println!();
  // Check for attribute presence
  println!("Contains 'required': {}", attrs.contains_key("required"));
  println!("Contains 'disabled': {}", attrs.contains_key("disabled"));
  println!();
  // Remove an attribute
  let removed = attrs.remove("data-validate");
  println!("Removed 'data-validate': {:?}", removed);
  println!("After removal: {}\n", attrs.to_html());
  // Merge attributes
  let mut base_attrs = Attributes::from([("class", "btn"), ("type", "button")]);
  let override_attrs = Attributes::from([("class", "btn btn-primary"), ("disabled", "")]);
  base_attrs.merge(override_attrs);
  println!("After merge:");
  println!("  HTML: {}\n", base_attrs.to_html());
  // Iterate over attributes
  println!("Iteration:");
  for (key, value) in attrs.iter() {
    println!("  {} = \"{}\"", key, value);
  }
  println!();
  // HTML escaping
  let mut escaped = Attributes::new();
  escaped.insert("data-json", r#"{"key": "value"}"#);
  escaped.insert("onclick", "alert('hello')");
  println!("With escaping:");
  println!("  HTML: {}", escaped.to_html());
}

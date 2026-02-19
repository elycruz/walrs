//! Example: Using ValueExt trait
//!
//! This example demonstrates the ValueExt extension trait for checking
//! if form values are "empty".
use serde_json::json;
use walrs_form_core::{Value, ValueExt};
fn main() {
  println!("ValueExt Example");
  println!("================\n");
  // Null is empty
  let null = Value::Null;
  println!("Value::Null is_empty_value: {}", null.is_empty_value());
  // Empty string is empty
  let empty_string = json!("");
  println!(
    "Empty string is_empty_value: {}",
    empty_string.is_empty_value()
  );
  // Non-empty string is not empty
  let string = json!("hello");
  println!("\"hello\" is_empty_value: {}", string.is_empty_value());
  // Numbers are never empty
  let zero = json!(0);
  let number = json!(42);
  println!("0 is_empty_value: {}", zero.is_empty_value());
  println!("42 is_empty_value: {}", number.is_empty_value());
  // Booleans are never empty
  let false_val = json!(false);
  let true_val = json!(true);
  println!("false is_empty_value: {}", false_val.is_empty_value());
  println!("true is_empty_value: {}", true_val.is_empty_value());
  // Empty array is empty
  let empty_arr: Value = json!([]);
  let arr = json!([1, 2, 3]);
  println!("[] is_empty_value: {}", empty_arr.is_empty_value());
  println!("[1,2,3] is_empty_value: {}", arr.is_empty_value());
  // Empty object is empty
  let empty_obj: Value = json!({});
  let obj = json!({"key": "value"});
  println!("{{}} is_empty_value: {}", empty_obj.is_empty_value());
  println!("{{key: value}} is_empty_value: {}", obj.is_empty_value());
  println!("\n--- Form Validation Use Case ---\n");
  // Practical use case: form validation
  let form_data = json!({
      "email": "user@example.com",
      "name": "",
      "age": null,
      "tags": []
  });
  let fields = ["email", "name", "age", "tags"];
  for field in fields {
    let value = &form_data[field];
    let status = if value.is_empty_value() {
      "EMPTY"
    } else {
      "has value"
    };
    println!("  {}: {} ({})", field, value, status);
  }
}

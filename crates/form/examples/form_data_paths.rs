//! Example: Using FormData with path-based access
//!
//! This example demonstrates how to use FormData's dot notation
//! and array indexing for nested data structures.
use walrs_form::FormData;
use walrs_validation::Value;

fn main() {
  println!("FormData Path-Based Access Example");
  println!("===================================\n");
  let mut data = FormData::new();
  // Simple values
  data.insert("email", Value::from("user@example.com"));
  data.insert("age", Value::from(25i32));
  println!("Simple access:");
  println!("  email: {:?}", data.get("email").unwrap().as_str());
  println!("  age: {:?}", data.get("age").unwrap().as_i64());
  println!();
  // Nested object - use serde_json bridge for complex literal construction
  let user_json = serde_json::json!({
      "profile": {
          "firstName": "John",
          "lastName": "Doe",
          "avatar": "avatar.png"
      },
      "settings": {
          "theme": "dark",
          "notifications": true
      }
  });
  data.insert("user", Value::from(user_json));
  println!("Dot notation access:");
  println!(
    "  user.profile.firstName: {:?}",
    data.get("user.profile.firstName").unwrap().as_str()
  );
  println!(
    "  user.settings.theme: {:?}",
    data.get("user.settings.theme").unwrap().as_str()
  );
  println!();
  // Array data - use serde_json bridge for complex literal
  let items_json = serde_json::json!([
      {"id": 1, "name": "Item 1", "price": 10.99},
      {"id": 2, "name": "Item 2", "price": 24.99},
      {"id": 3, "name": "Item 3", "price": 5.49}
  ]);
  data.insert("items", Value::from(items_json));
  println!("Array indexing:");
  println!(
    "  items[0].name: {:?}",
    data.get("items[0].name").unwrap().as_str()
  );
  println!(
    "  items[1].price: {:?}",
    data.get("items[1].price").unwrap().as_f64()
  );
  println!(
    "  items[2].id: {:?}",
    data.get("items[2].id").unwrap().as_i64()
  );
  println!();
  // Setting nested values
  println!("Setting nested values:");
  data.set("address.street", Value::from("123 Main St"));
  data.set("address.city", Value::from("Springfield"));
  data.set("address.state", Value::from("IL"));
  println!(
    "  address.street: {:?}",
    data.get("address.street").unwrap().as_str()
  );
  println!(
    "  address.city: {:?}",
    data.get("address.city").unwrap().as_str()
  );
  println!();
  // Setting array values
  println!("Setting array values:");
  data.set("tags[0]", Value::from("rust"));
  data.set("tags[1]", Value::from("web"));
  data.set("tags[2]", Value::from("forms"));
  println!("  tags[0]: {:?}", data.get("tags[0]").unwrap().as_str());
  println!("  tags[1]: {:?}", data.get("tags[1]").unwrap().as_str());
  println!("  tags[2]: {:?}", data.get("tags[2]").unwrap().as_str());
  println!();
  // Complex nested array
  data.set("orders[0].items[0].name", Value::from("Widget"));
  data.set("orders[0].items[0].qty", Value::from(5i32));
  data.set("orders[0].items[1].name", Value::from("Gadget"));
  data.set("orders[0].items[1].qty", Value::from(2i32));
  println!("Complex nested paths:");
  println!(
    "  orders[0].items[0].name: {:?}",
    data.get("orders[0].items[0].name").unwrap().as_str()
  );
  println!(
    "  orders[0].items[1].qty: {:?}",
    data.get("orders[0].items[1].qty").unwrap().as_i64()
  );
  println!();
  // Out of bounds returns None
  println!("Out of bounds access:");
  println!("  items[99]: {:?}", data.get("items[99]"));
  println!("  nonexistent.path: {:?}", data.get("nonexistent.path"));
}

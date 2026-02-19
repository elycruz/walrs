# walrs_form_core
Core types for the walrs form ecosystem.
## Overview
`walrs_form_core` provides shared foundation types used across all form-related crates in the walrs ecosystem. It re-exports `serde_json::Value` as the primary dynamic value type and provides additional helpers.
## Key Types
### Value
Re-export of `serde_json::Value` for dynamic form data:
```rust
use walrs_form_core::Value;
use serde_json::json;
let value: Value = json!({"name": "John", "age": 30});
// Built-in helpers
assert_eq!(value["name"].as_str(), Some("John"));
assert_eq!(value["age"].as_i64(), Some(30));
```
### ValueExt
Extension trait adding form-specific helpers:
```rust
use walrs_form_core::{Value, ValueExt};
use serde_json::json;
let null = Value::Null;
let empty_string = json!("");
let value = json!("hello");
assert!(null.is_empty_value());
assert!(empty_string.is_empty_value());
assert!(!value.is_empty_value());
```
The `is_empty_value()` method returns `true` for:
- `Value::Null`
- Empty strings
- Empty arrays
- Empty objects
### Attributes
HTML attributes storage with escaping:
```rust
use walrs_form_core::Attributes;
let mut attrs = Attributes::new();
attrs.insert("class", "form-control");
attrs.insert("id", "email-input");
attrs.insert("data-validate", "true");
// Render as HTML
assert_eq!(attrs.to_html(), r#"class="form-control" data-validate="true" id="email-input""#);
```
## Installation
Add to your `Cargo.toml`:
```toml
[dependencies]
walrs_form_core = { path = "../form_core" }
```
## Usage
```rust
use walrs_form_core::{Value, ValueExt, Attributes};
use serde_json::json;
// Dynamic values
let form_data: Value = json!({
    "email": "user@example.com",
    "remember": true,
    "tags": ["rust", "web"]
});
// Check if fields are empty
if !form_data["email"].is_empty_value() {
    println!("Email: {}", form_data["email"]);
}
// HTML attributes
let mut attrs = Attributes::new();
attrs.insert("placeholder", "Enter email");
attrs.insert("required", "");
```
## Why serde_json::Value?
We use `serde_json::Value` directly instead of a custom enum because:
1. **Transparent serialization**: Automatic JSON compatibility
2. **Rich API**: Methods like `as_str()`, `as_i64()`, `as_f64()`, `as_bool()`, `as_array()`, `as_object()`
3. **Conversions**: `From` implementations for all common types
4. **Ecosystem**: Compatible with all serde-based crates
5. **Zero overhead**: No wrapper or conversion needed
## Architecture
This crate serves as the foundation for:
- `walrs_inputfilter`: Field-level validation
- `walrs_form`: Form structure and elements
- `walrs_form_serde`: JSON/YAML loading
## License
MIT OR Apache-2.0

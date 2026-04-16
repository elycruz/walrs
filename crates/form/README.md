# walrs_form
Form elements and structure for the walrs form ecosystem.
## Overview
`walrs_form` provides data structures for representing HTML form elements in server-side environments. All structures are serializable and designed for use in both Rust web frameworks and JavaScript/TypeScript via WASM.
## Features
- **Form Elements**: `InputElement`, `SelectElement`, `TextareaElement`, `ButtonElement`
- **Type Enums**: `InputType`, `SelectType`, `ButtonType`
- **Containers**: `Form`, `FieldsetElement`
- **Data Handling**: `FormData` with path-based access for nested structures
- **Polymorphism**: `Element` enum for handling mixed element collections
- **Serialization**: Full serde support for JSON/YAML
## Installation
Add to your `Cargo.toml`:
```toml
[dependencies]
walrs_form = { path = "../form" }
```
## Quick Start
```rust
use walrs_form::{Form, FormMethod, InputElement, InputType, ButtonElement, ButtonType, FormData};
use serde_json::json;
// Create a form
let mut form = Form::new("login");
form.action = Some("/api/login".to_string());
form.method = Some(FormMethod::Post);
// Add elements
form.add_element(InputElement::new("username", InputType::Text).into());
form.add_element(InputElement::new("password", InputType::Password).into());
form.add_element(ButtonElement::with_label("Sign In", ButtonType::Submit).into());
// Bind data
let mut data = FormData::new();
data.insert("username", json!("john_doe"));
form.bind_data(data);
// Serialize to JSON
let json = serde_json::to_string_pretty(&form).unwrap();
```
## Element Types
### InputElement
```rust
use walrs_form::{InputElement, InputType};
let email = InputElement::new("email", InputType::Email);
let password = InputElement::new("password", InputType::Password);
let number = InputElement::new("age", InputType::Number);
```
Supported input types: `Text`, `Email`, `Password`, `Number`, `Checkbox`, `Radio`, `File`, `Date`, `DateTime`, `Month`, `Week`, `Time`, `Tel`, `Url`, `Color`, `Range`, `Search`, `Hidden`.
### SelectElement
```rust
use walrs_form::{SelectElement, SelectOption};
let mut select = SelectElement::new("country");
select.add_option(SelectOption::new("us", "United States"));
select.add_option(SelectOption::new("ca", "Canada"));
// Multi-select
let multi = SelectElement::multiple("tags");
```
### TextareaElement
```rust
use walrs_form::TextareaElement;
let textarea = TextareaElement::with_size("bio", 5, 40);
```
### FieldsetElement
```rust
use walrs_form::{FieldsetElement, InputElement, InputType};
let mut fieldset = FieldsetElement::with_legend("Address");
fieldset.add_element(InputElement::new("street", InputType::Text).into());
fieldset.add_element(InputElement::new("city", InputType::Text).into());
```
## FormData with Path Access
`FormData` supports dot notation and array indexing for nested data:
```rust
use walrs_form::FormData;
use serde_json::json;
let mut data = FormData::new();
data.insert("user", json!({"email": "test@example.com"}));
// Dot notation
assert_eq!(data.get("user.email").unwrap().as_str(), Some("test@example.com"));
// Array indexing
data.insert("items", json!([{"name": "Item 1"}, {"name": "Item 2"}]));
assert_eq!(data.get("items[0].name").unwrap().as_str(), Some("Item 1"));
// Set nested values
data.set("address.city", json!("New York"));
```
## Validation
Elements can have `Field<Value>` configurations for validation:
```rust
use walrs_form::{InputElement, InputType, Field, FieldBuilder};

let mut input = InputElement::new("email", InputType::Email);
input.field = Some(
    FieldBuilder::default()
        .required(true)
        .build()
        .unwrap()
);
```
## Async Validation
Enable the `async` feature for async validation support:
```toml
[dependencies]
walrs_form = { path = "../form", features = ["async"] }
```
This enables `validate_value_async` on `InputElement`, `SelectElement`, and `TextareaElement`, plus `Form::validate_async()` and `Form::process_async()`:
```rust
use walrs_form::{Form, InputElement, InputType, FormData, FieldBuilder};
use walrs_validation::{Rule, Value};

let mut form = Form::new("login");
let mut input = InputElement::new("email", InputType::Email);
input.field = Some(
    FieldBuilder::default()
        .rule(Rule::required())
        .build()
        .unwrap(),
);
form.add_element(input.into());

let mut data = FormData::new();
data.insert("email", Value::from("user@example.com"));

// Async validation
let result = form.validate_async(&data).await;
assert!(result.is_ok());

// Async process (validate + return data)
let processed = form.process_async(&data).await.unwrap();
```
The async feature is runtime-agnostic (`std::future::Future` only). Filtering remains synchronous; only validation becomes async, following the pattern from `walrs_fieldfilter`.
## Architecture
This crate is part of the walrs form ecosystem:
- `walrs_validation`: Shared types (`Value`, `Attributes`) and validation rules
- `walrs_fieldfilter`: Field-level validation (`Field<T>`, `FieldFilter`)
- `walrs_form`: Form structure and elements (this crate)
- `walrs_filter`: Value transformation filters
## License
Elastic-2.0

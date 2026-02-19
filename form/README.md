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
use walrs_inputfilter::filter_enum::Filter;
let mut input = InputElement::new("email", InputType::Email);
input.field = Some(
    FieldBuilder::default()
        .required(true)
        .filters(vec![Filter::Trim, Filter::Lowercase])
        .build()
        .unwrap()
);
```
## Architecture
This crate is part of the walrs form ecosystem:
- `walrs_form_core`: Shared types (`Value`, `Attributes`)
- `walrs_inputfilter`: Field-level validation (`Field<T>`, `FieldFilter`)
- `walrs_form`: Form structure and elements (this crate)
- `walrs_validator`: Validation rules
- `walrs_filter`: Value transformation filters
## License
MIT OR Apache-2.0

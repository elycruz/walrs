# walrs_inputfilter
Field-level validation and filtering for the walrs form ecosystem.
## Overview
`walrs_inputfilter` provides the core validation and filtering infrastructure for form processing. It includes the new unified `Field<T>` API that replaces the older `Input`/`RefInput` approach, plus multi-field validation with `FieldFilter`.
## Key Types
### Field<T>
Unified field configuration for validation and filtering:
```rust
use walrs_inputfilter::{Field, FieldBuilder};
use walrs_inputfilter::filter_enum::Filter;
use walrs_validator::Rule;
use walrs_form_core::Value;
use serde_json::json;
// Single rule
let field: Field<Value> = FieldBuilder::default()
    .rule(Rule::Required)
    .filters(vec![Filter::Trim, Filter::Lowercase])
    .build()
    .unwrap();
// Multiple rules using Rule::All (via .and() combinator)
let field: Field<Value> = FieldBuilder::default()
    .rule(Rule::Required.and(Rule::MinLength(3)))
    .build()
    .unwrap();
// Validate
let result = field.validate(&json!(""));
assert!(result.is_err()); // Required field is empty
```
### Filter<T> Enum
Serializable filters for value transformation:
```rust
use walrs_inputfilter::filter_enum::Filter;
let filters: Vec<Filter<String>> = vec![
    Filter::Trim,
    Filter::Lowercase,
    Filter::StripTags,
];
let mut value = "  <b>HELLO</b>  ".to_string();
for filter in &filters {
    value = filter.apply(value);
}
assert_eq!(value, "hello");
```
Available filters:
- `Trim` - Remove whitespace
- `Uppercase` / `Lowercase` - Case transformation
- `StripTags` - Remove HTML tags
- `Slug` - URL-safe slug generation
- `Clamp(min, max)` - Numeric clamping
- `Chain(filters)` - Sequential filter chain
- `Custom(fn)` - Custom filter function
### FieldFilter
Multi-field validation with cross-field rules:
```rust
use walrs_inputfilter::{FieldFilter, Field, FieldBuilder};
use walrs_inputfilter::field_filter::CrossFieldRule;
use walrs_validator::Rule;
use std::collections::HashMap;
use serde_json::json;
let filter = FieldFilter::new()
    .with_field("email", FieldBuilder::default().rule(Rule::Required).build().unwrap())
    .with_field("password", FieldBuilder::default().rule(Rule::Required).build().unwrap())
    .with_field("confirm_password", FieldBuilder::default().rule(Rule::Required).build().unwrap())
    .with_cross_field_rule(CrossFieldRule::FieldsEqual {
        fields: vec!["password".to_string(), "confirm_password".to_string()],
        message: "Passwords must match".to_string(),
    });
let mut data = HashMap::new();
data.insert("email".to_string(), json!("user@example.com"));
data.insert("password".to_string(), json!("secret123"));
data.insert("confirm_password".to_string(), json!("secret456"));
let result = filter.validate(&data);
assert!(result.is_err()); // Passwords don't match
```
### Cross-Field Rules
Built-in rules for multi-field validation:
- **FieldsEqual** - Fields must have equal values (password confirmation)
- **RequiredIf** - Field required if another field has specific value
- **RequiredUnless** - Field required unless another field has specific value
- **OneOfRequired** - At least one field must have a value
- **MutuallyExclusive** - Only one field can have a value
- **DependentRequired** - Field required when another field has any value
- **Custom** - Custom validation function
### FormViolations
Aggregate validation errors:
```rust
use walrs_inputfilter::FormViolations;
use walrs_validator::{Violations, Violation, ViolationType};
let mut violations = FormViolations::new();
// Add field-specific violations
violations.add_field_violation("email", 
    Violation::new(ViolationType::ValueMissing, "Email is required"));
// Add form-level violations
violations.add_form_violation(
    Violation::new(ViolationType::CustomError("auth".into()), "Invalid credentials"));
if !violations.is_empty() {
    for field_name in violations.field_names() {
        println!("Field '{}' has errors", field_name);
    }
}
```
## Legacy Types
The following types are still available but deprecated:
- `Input<T>` - Use `Field<T>` instead
- `RefInput<T>` - Use `Field<T>` instead
## Installation
```toml
[dependencies]
walrs_inputfilter = { path = "../inputfilter" }
```
## Architecture
This crate sits between `walrs_validator` and `walrs_form`:
```
walrs_validator    → walrs_inputfilter → walrs_form
(Rule<T> enum)       (Field<T>,          (Form,
                      FieldFilter,        Element)
                      Filter<T>)
```
## License
MIT OR Apache-2.0

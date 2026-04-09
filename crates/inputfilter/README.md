# walrs_inputfilter
Field-level validation and filtering for the walrs form ecosystem.
## Overview
`walrs_inputfilter` provides the core validation and filtering infrastructure for form processing. It includes the new unified `Field<T>` API that replaces the older `Input`/`RefInput` approach, plus multi-field validation with `FieldFilter`.
## Key Types
### Field<T>
Unified field configuration for validation and filtering:
```rust
use walrs_inputfilter::{Field, FieldBuilder};
use walrs_filter::FilterOp;
use walrs_validation::Rule;
use walrs_validation::Value;
use serde_json::json;
// Simple field with just a rule (filters are optional)
let field: Field<Value> = FieldBuilder::default()
    .rule(Rule::Required)
    .build()
    .unwrap();
// Field with rule and filter operations
let field: Field<Value> = FieldBuilder::default()
    .rule(Rule::Required.and(Rule::MinLength(3)))
    .filters(vec![FilterOp::Trim, FilterOp::Lowercase])
    .build()
    .unwrap();
// Validate
let result = field.validate(&json!(""));
assert!(result.is_err()); // Required field is empty
```
### FilterOp<T> Enum
Serializable filter operations for value transformation (defined in `walrs_filter`, re-exported here):
```rust
use walrs_filter::FilterOp;
let filters: Vec<FilterOp<String>> = vec![
    FilterOp::Trim,
    FilterOp::Lowercase,
    FilterOp::StripTags,
];
// apply_ref accepts &str — no allocation needed at the call site
let mut value = "  <b>HELLO</b>  ".to_string();
for filter in &filters {
    value = filter.apply_ref(&value);
}
assert_eq!(value, "hello");
```
Available filter operations:
- `Trim` - Remove whitespace
- `Uppercase` / `Lowercase` - Case transformation
- `StripTags` - Remove HTML tags
- `Slug` - URL-safe slug generation
- `Clamp(min, max)` - Numeric clamping
- `Chain(ops)` - Sequential filter chain
- `Custom(fn)` - Custom filter function
### FieldFilter
Multi-field validation with cross-field rules:
```rust
use walrs_inputfilter::FieldFilter;
use walrs_inputfilter::field::FieldBuilder;
use walrs_inputfilter::field_filter::{CrossFieldRule, CrossFieldRuleType};
use walrs_validation::Value;
use walrs_validation::Rule;
use std::collections::HashMap;
use serde_json::json;

let mut filter = FieldFilter::new();

// Fluent API - chain add_field and add_cross_field_rule calls
filter
    .add_field("email", FieldBuilder::<Value>::default().rule(Rule::Required).build().unwrap())
    .add_field("password", FieldBuilder::<Value>::default().rule(Rule::Required).build().unwrap())
    .add_field("confirm_password", FieldBuilder::<Value>::default().rule(Rule::Required).build().unwrap())
    .add_cross_field_rule(CrossFieldRule {
        name: Some("password_match".into()),
        fields: vec!["password".to_string(), "confirm_password".to_string()],
        rule: CrossFieldRuleType::FieldsEqual {
            field_a: "password".to_string(),
            field_b: "confirm_password".to_string(),
        },
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
use walrs_validation::{Violations, Violation, ViolationType};
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
This crate sits between `walrs_validation` and `walrs_form`:
```
walrs_validation    → walrs_filter      → walrs_inputfilter → walrs_form
(Rule<T> enum)       (Filter trait,       (Field<T>,          (Form,
                      FilterOp<T> enum)    FieldFilter)        Element)
```
## License
MIT OR Apache-2.0

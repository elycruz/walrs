# walrs_fieldfilter
Field-level validation and filtering for form processing.
## Overview
`walrs_fieldfilter` provides the core validation and filtering infrastructure for form processing, built around `Field<T>` for single-field configuration and `Fieldset` for typed multi-field structs.
## Key Types
### Field<T>
Unified field configuration for validation and filtering:
```rust
use walrs_fieldfilter::{Field, FieldBuilder};
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
    value = filter.apply_ref(&value).into_owned();
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
### Fieldset

Typed struct validation and filtering with compile-time guarantees — the recommended approach for multi-field forms. Use the `derive` feature to auto-generate implementations:

```rust
use walrs_fieldfilter::{DeriveFieldset, Fieldset};

#[derive(Debug, DeriveFieldset)]
struct ContactForm {
    #[validate(required, email)]
    #[filter(trim, lowercase)]
    email: String,

    #[validate(required, min_length = 2)]
    #[filter(trim)]
    name: String,
}

fn main() {
    let form = ContactForm {
        email: "  USER@EXAMPLE.COM  ".into(),
        name: "  Alice  ".into(),
    };
    
    match form.clean() {
        Ok(cleaned) => println!("Cleaned: {:?}", cleaned),
        Err(violations) => eprintln!("Errors: {}", violations),
    }
}
```

**Key features:**
- `clean()` — filter then validate (convenience method)
- `validate()` — validate without filtering
- `filter()` — filter without validation
- Nested struct support with `#[validate(nested)]` and `#[filter(nested)]`
- Cross-field validation with `#[cross_validate(...)]`
- `Option<T>` handling
- Custom validators and filters

See [`crates/fieldset_derive/README.md`](../fieldset_derive/README.md) for complete documentation.

## Installation

### Basic (without derive macro)
```toml
[dependencies]
walrs_fieldfilter = { path = "../fieldfilter" }
```

### With derive macro
```toml
[dependencies]
walrs_fieldfilter = { path = "../fieldfilter", features = ["derive"] }
```
## Architecture
This crate sits between `walrs_validation` and `walrs_fieldset_derive`:
```
walrs_validation    → walrs_filter      → walrs_fieldfilter ← walrs_fieldset_derive
(Rule<T> enum)       (Filter trait,       (Field<T>,           (#[derive(Fieldset)])
                      FilterOp<T> enum)    Fieldset)
```
## License
Elastic-2.0

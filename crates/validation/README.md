# walrs_validation

Composable validation rules for input validation.

This crate provides a serializable, composable validation rule system based on
the `Rule<T>` enum, along with core validation traits.

## Validation Rules

The `Rule` enum provides built-in validation for common constraints:

- `Rule::Required` - Value must not be empty
- `Rule::MinLength` / `Rule::MaxLength` - Length constraints
- `Rule::Min` / `Rule::Max` - Range constraints
- `Rule::Pattern` - Regex pattern matching
- `Rule::Email` - Email format validation
- `Rule::Step` - Step/multiple validation
- `Rule::Hostname` - Configurable hostname validation (DNS/IP/local/public IPv4)
- `Rule::Custom` - Custom closure-based validation

## Rule Composition

Rules can be composed using methods on `Rule`:

- `.and()` - Both rules must pass (AND logic, produces `Rule::All`)
- `.or()` - At least one rule must pass (OR logic, produces `Rule::Any`)
- `.not()` - Negates a rule (produces `Rule::Not`)
- `.when()` / `.when_else()` - Conditional validation (produces `Rule::When`)

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
walrs_validation = { path = "../validation" }
```

## Example

```rust
use walrs_validation::{Rule, Validate, ValidateRef};

fn main() {
    // Length validation
    let length_rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(20));

    assert!(length_rule.validate_ref("hello").is_ok());
    assert!(length_rule.validate_ref("hi").is_err());

    // Range validation
    let range_rule = Rule::<i32>::Min(0).and(Rule::Max(100));

    assert!(range_rule.validate(50).is_ok());
    assert!(range_rule.validate(-1).is_err());
}
```

## Validation Traits

The crate provides two main validation traits:

```rust
// For Copy types (numbers, etc.)
pub trait Validate<T> {
    fn validate(&self, value: T) -> ValidatorResult;
}

// For referenced types (str, slices, etc.)
pub trait ValidateRef<T: ?Sized> {
    fn validate_ref(&self, value: &T) -> ValidatorResult;
}
```

## Shared Types

This crate also provides shared foundation types used across form-related crates:

- **`Value`** - Native form value enum with distinct numeric variants (`I64`, `U64`, `F64`), `Str`, `Bool`, `Array`, `Object(HashMap<String, Value>)`, and `Null`
- **`ValueExt`** - Extension trait with form-specific helper methods (e.g., `is_empty_value()`)
- **`value!`** - Convenience macro for constructing `Value` literals
- **`Attributes`** - HTML attributes storage and rendering

### `serde_json` Bridge

The `serde_json_bridge` feature (enabled by default) provides `From<serde_json::Value> for Value` and vice-versa for interoperability.

### `indexmap` Support

The `indexmap` feature provides `From<IndexMap<String, V>> for Value` for constructing `Value::Object` from an `IndexMap`.

## License

MIT & Apache-2.0

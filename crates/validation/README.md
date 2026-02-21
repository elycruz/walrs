# walrs_validation

Validator structs for input validation.

This crate provides reusable validator implementations that can validate input values against various constraints. Validators are typically used in form processing pipelines to ensure user input meets requirements.

## Available Validators

- **`LengthValidator`** - Validates string/collection length constraints
- **`PatternValidator`** - Validates strings against regex patterns
- **`RangeValidator`** - Validates scalar values within a range (numbers, chars, etc.)
- **`StepValidator`** - Validates that numeric values are multiples of a step.
- **`EqualityValidator`** - Validates equality against a specified value

## Combinators

Validators can be combined using logical operations:

- **`ValidatorAnd`** - Both validators must pass (AND logic)
- **`ValidatorOr`** - At least one validator must pass (OR logic)
- **`ValidatorNot`** - Negates a validator
- **`ValidatorOptional`** - Skips validation for empty values
- **`ValidatorWhen`** - Conditional validation
- **`ValidatorAll`** - Collects all validation errors

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
walrs_validation = { path = "../validator" }  # or from crates.io when published
```

## Example

```rust
use walrs_validation::{
    LengthValidatorBuilder, RangeValidatorBuilder,
    Validate, ValidateRef, ValidateExt,
};

fn main() {
    // Length validation
    let length_validator = LengthValidatorBuilder::<str>::default()
        .min_length(3)
        .max_length(20)
        .build()
        .unwrap();
    
    assert!(length_validator.validate_ref("hello").is_ok());
    assert!(length_validator.validate_ref("hi").is_err());
    
    // Range validation with combinators
    let min_validator = RangeValidatorBuilder::<i32>::default()
        .min(0)
        .build()
        .unwrap();
    
    let max_validator = RangeValidatorBuilder::<i32>::default()
        .max(100)
        .build()
        .unwrap();
    
    let range_validator = min_validator.and(max_validator);
    assert!(range_validator.validate(50).is_ok());
    assert!(range_validator.validate(-1).is_err());
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

## Features

- **`fn_traits`** - Enables nightly `Fn` trait implementations so validators can be called as functions.
- **`debug_closure_helpers`** - Enables debug helpers for closures.
- **`nightly`** - Enables all nightly features.

## Running Examples

The crate includes several examples demonstrating validator usage:

```bash
# Basic validator usage (Length, Pattern, Range, Number, Equality)
cargo run -p walrs_validation --example basic_validators

# Using validator combinators (AND, OR, NOT, Optional, When)
cargo run -p walrs_validation --example validator_combinators

# Form validation example with multiple validators
cargo run -p walrs_validation --example form_validation
```

## Running Benchmarks

Benchmarks are available to measure validator performance:

```bash
# Run all benchmarks
cargo bench -p walrs_validation

# Run specific benchmark group
cargo bench -p walrs_validation -- LengthValidator
cargo bench -p walrs_validation -- RangeValidator
cargo bench -p walrs_validation -- PatternValidator
```

Benchmark groups include:
- **LengthValidator** - Tests length validation with various string sizes
- **RangeValidator** - Tests numeric range validation
- **StepValidator** - Tests step/multiple validation
- **PatternValidator** - Tests regex pattern matching
- **EqualityValidator** - Tests equality comparisons
- **CombinedValidators** - Tests combinator performance
- **ValidatorComparison** - Compares performance across validator types

## Shared Types

This crate also provides shared foundation types used across form-related crates:

- **`Value`** - Re-export of `serde_json::Value` for dynamic form data
- **`ValueExt`** - Extension trait with form-specific helper methods (e.g., `is_empty_value()`)
- **`Attributes`** - HTML attributes storage and rendering

## License

MIT & Apache-2.0

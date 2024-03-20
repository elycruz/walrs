# Validators

## Equality Validator

Equality validator - Holds a 'rhs_value' and validates incoming value against the stored value.

## Number Validator

Can validate (signed, and unsigned) integer, and float values:

- i8
- i16
- i32
- i64
- i128
- isize

- u8
- u16
- u32
- u64
- u128
- usize

- f32
- f64

## Pattern Validator

Validates a string (`&str`) against stored regular expression.

## Range Validator

Validates a scalar value against a `min`, and/or a `max`.

## LengthValidator

Validates an item's length.
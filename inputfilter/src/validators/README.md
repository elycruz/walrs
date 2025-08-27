# Validators

Structs that implement the `Fn*` traits which can be used to validate input values from any context.

## Equality Validator

Equality validator - Holds a `rhs_value` and validates incoming value against the stored value.

Currently only supports:

- `&str`
- Signed integers (i8, ..., isize)
- Unsigned integers (u8, ..., usize)
- Floating point numbers (f32, f64)

## Length Validator

Validates values that implement the `WithLength` trait, which is currently implemented for the following:

- `&str`
- `&[T]`
- `HashMap<K, V>`
- `HashSet<T>`
- `BTreeMap<K, V>`
- `BTreeSet<T>`
- `LinkedList<T>`
- `BinaryHeap<T>`
- `Vec<T>`
- `VecDeque<T>`

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

Validates an `&str` against stored regular expression.

## Range Validator

Validates a scalar value against a `min`, and/or a `max`, value.

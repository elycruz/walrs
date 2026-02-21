# Validators

Structs that implement the `Fn*` traits which can be used to validate input values from any context.

## Types of Validators

There are two types of validators in this crate:

1.  Ones that work with "Scalar" values (integers, floats, bools, chars, etc.).
2.  Ones that work with Unsized values (like `str`, slices, and collections).

When working with these make sure to pick the type that will work for your use case.

## Equality Validator

Works with scalar and/or unsized types.

Holds a `rhs_value` and validates incoming value against the stored value.

## Length Validator

Validates that given value's length is within specified range.

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

Validates number values against set range and/or step values.

Works with primitive number types:

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

Validates a string against a pattern.

## Range Validator

Validates given scalar value against specified range.

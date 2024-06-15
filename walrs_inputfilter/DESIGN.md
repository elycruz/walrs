# `inputfilter` package design

## Implementation Goals

Controls here should:

- not be stateful - In the sense of 'changing' state;  E.g., should not hold on to/mutate values.
- Should only work with primitive values;  E.g., scalars, array, vector, hash_map, etc., to limit implementation complexity (note we can support arbitrary structures (later) via derive macros, etc.).

## Inspiration

Original inspiration comes from:

- https://docs.laminas.dev/laminas-inputfilter/
- https://github.com/functional-jslib/fjl/tree/monorepo/packages/fjl-validator
- https://github.com/functional-jslib/fjl/tree/monorepo/packages/fjl-inputfilter

**Note:** In comparison to laminas-inputfilter we don't need to convert string values to numbers, etc., when using a web-framework like actix-web, as they automatically do this for us (see https://actix.rs/docs/extractors, for more).  

Due to the above, in this library, we'll require less Validator, and Filter, structs since type coercion is handled for us.

## Where and how would we use `*Input`/`*Constraint` controls

- In action handlers where we might need to instantiate a constraints object, or optionally, retrieve a globally instantiated/stored one.
- In a terminal application where we might want to reuse the same functionality stored (though in this instance rust's built-in facilities for working with command line flags might be more appropriate (possibly less memory overhead, et al.?)).

## Design:

### Current:

- `./constraints` - Structs with validation methods, validation properties, and filter methods used for validating/filtering given value(s).
- `./filters` - Structs that implement `Fn` traits that transform incoming values.
- [tentative] `./validators` - `Fn` structs that validate a given value against some inherent configuration.

### Multi trait implementations approach

One Input struct with multiple implementations of the `InputConstraints` trait.

The Input Constraints trait itself needs to accept the validator and filter types themselves.

## Questions

### General

- Do function references need to be wrapped in `Arc<...>` to be shared across threads safely?  No - If the owning struct is itself wrapped in `Arc<...>` then all members that can satisfy `Send + Sync` automatically become shareable (across threads) .

### `Cow<T>` vs `&T` vs `T` in `validate` method calls 

| Type     | PROs                           | CONs                                                            |
|----------|--------------------------------|-----------------------------------------------------------------|
| `Cow<T>` | Allows better type flexibility | Tedious to type                                                 |
| `&T`     | Simplifies APIs                | Can cause overhead when requiring `Copy` types.                 |
| `T`      | Simplifies APIs                | Offsets API complexity elsewhere but can cause lifetime errors. |

Here we're going with `&T` for simplicity's sake.

### Other

- Do scalars, and `str`, implement:
  - [x] Debug
  - [x] Display
  - [x] PartialOrd
  - [x] PartialEq

## TODOs

- [ ] Add constraint `T: Into<FT>`
- [x] ~~Change `validator*` methods to accept `&T`.~~ - No longer required as we're only supporting 'Copy', and/or Scalar, types.
- [ ] Question: Do we need `Debug`, and `Display`, traits on `InputConstraints` type?
- [ ] Consider making `ValueMissingCallback` types accept Constraints/Input type as first param. - In 'Input' struct we will support this.

## Visited Designs

### ViolationTuple

```rust
struct ViolationTuple (ViolationEnum, ViolationMessage);

impl ToString for ViolationTuple {
    fn to_string(&self) -> String {
        self.1.to_string()
    }
}

fn main () {
  let violation_tuples = vec![
      ViolationTuple(ViolationEnum::ValueMissing, "Value is missing".to_string()),
      ViolationTuple(ViolationEnum::ValueMissing, "Value is missing".to_string())
  ];
  
  let violation_strings: Vec<String> = violation_tuples.into();
}

```
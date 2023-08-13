# `inputfilter` package design

## Implementation Goals

Controls here should:

- not be stateful - In the sense of 'changing' state;  E.g., should not hold on/mutate values received for validation.
- Should only work with primitive values;  E.g., scalars, array, vector, hash_map, etc. (note we can support arbitrary structures (later) via derive macros).

## Inspiration

Original inspiration comes from:

- https://docs.laminas.dev/laminas-inputfilter/
- https://github.com/functional-jslib/fjl/tree/monorepo/packages/fjl-validator
- https://github.com/functional-jslib/fjl/tree/monorepo/packages/fjl-inputfilter

## Where and how would we use `Input` controls

- In action handlers where we might need to instantiate a validator, or optionally, retrieve a globally instantiated/stored one.
- In a terminal application where we might want to reuse the same functionality stored (though in this instance rust built-in facilities for working with command line flags might be more appropriate (possibly less memory over, et al.?)).

## Notes:

- So far the only (test) implementation that worked out is the one where we recieve `Cow<T>` in `validate` (method) calls - The more desirable type, though, here is `T`.

## Questions

### General

- Do function references need to be wrapped in `Arc<...>` to be shared across threads safely?  Yes.

### `Cow<T>` vs `&T` vs `T` in `validate` method calls 

| Type     | PROs                           | CONs                                                            |
|----------|--------------------------------|-----------------------------------------------------------------|
| `Cow<T>` | Allows better type flexibility | Tedious to type                                                 |
| `&T`     | Simplifies APIs                | Can cause overhead when requiring `Copy` types.                 |
| `T`      | Simplifies APIs                | Offsets API complexity elsewhere and can cause lifetime errors. |

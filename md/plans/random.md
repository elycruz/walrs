## Random Todos:

- Remove:
  - Field
  - (legacy) Input
  - (legacy) RefInput
  - FieldFilter
- Move Filter enum to filter crate
- Remove old validator structs (`Rule` will supersede.)
- Disambiguate `Filter` enum from `Filter` trait.
- Make `pub(crate)` updates (`pub(crate) fn validate_(str|len|etc)`).
- Address ` message: Message::Static(String::new()),` in `WithMessage` handling.
- Address "Value used after move" in navigation_benchmarks.rs.
- Consider adding a `builder(name)` method to `*Element` structures - enables builder pattern than can compile `Rule<T>` from field values to be populated for target struct.
- Expose all crates from root crate `lib` using their short names.
- Finalize lib licenses and reference them in all crates (READMEs, LICENSEs, Cargo.toml, etc.).

Remove:
- Field
- (legacy) Input
- (legacy) RefInput
- FieldFilter

- Change `walrs_form_core` to `walrs_validation_core`
- Move Filter enum to filter crate
- Rename `walrs_validation` to `walrs_validation`.
- Remove old validator structs (`Rule` will supersede.)
- change `with_message_provider` to `with_message`.
- Disambiguate `Filter` enum from `Filter` trait.
- Move all crates to 'crates/'.
- Make `pub(crate)` updates (`pub(crate) fn validate_(str|len|etc)`).
- Address ` message: Message::Static(String::new()),` in `WithMessage` handling.
- Address "Value used after move" in navigation_benchmarks.rs.
- Consider adding a `builder(name)` method to `*Element` structures - enables builder pattern than can compile `Rule<T>` from field values to be populated for target struct.
- Expose all crates from root crate `lib` using their short names.
- Finalize lib licenses and reference them in all crates (READMEs, LICENSEs, Cargo.toml, etc.).
- 
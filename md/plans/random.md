Remove:
- Field
- (legacy) Input
- (legacy) RefInput
- FieldFilter

- Change `walrs_form_core` to `walrs_validation_core`
- Move Filter enum to filter crate
- Move `Rule<T>` to it's own crate.
- Remove old validator structs (`Rule` will supersede.)
- change `with_message_provider` to `with_message`.
- Disambiguate `Filter` enum from `Filter` trait.
- Move all crates to 'crates/'.
- Make `pub(crate)` updates (`pub(crate) fn validate_(str|len|etc)`).
- Address ` message: Message::Static(String::new()),` in `WithMessage` handling.
- Address "Value used after move" in navigation_benchmarks.rs.
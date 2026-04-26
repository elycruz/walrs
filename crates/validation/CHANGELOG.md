# Changelog

All notable changes to `walrs_validation` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-04-26

Coordinated pre-1.0 bump removing the dynamic `Value` path. See
[`md/plans/2026-04-25-dynamic-path-removal.md`](../../md/plans/2026-04-25-dynamic-path-removal.md)
and [issue #267](https://github.com/elycruz/walrs/issues/267) for context.

### Removed (breaking)

- `Value` enum and all variants (`I64`, `U64`, `F64`, `Str`, `Bool`, `Array`,
  `Object`, `Null`).
- `ValueExt` trait and the `value!` macro.
- `Rule<Value>` and `Condition<Value>` dispatch (`crates/validation/src/rule_impls/value.rs`).
- `IsEmpty for Value` impl.
- The `value` feature flag.
- `value_validation` example.

### Changed (breaking)

- `serde_json_bridge` no longer implies the removed `value` feature. The
  bridge uses `serde_json::Value` directly and never required the now-removed
  `walrs_validation::Value`.
- `serde_json` is now an optional dependency, gated on `serde_json_bridge`.

### Migration

Replace `Field<Value>` / `Rule<Value>` / `FilterOp<Value>` with typed
struct definitions and `#[derive(Fieldset)]` from `walrs_fieldset_derive`.

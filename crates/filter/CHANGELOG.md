# Changelog

All notable changes to `walrs_filter` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The crate is still pre-publish; nothing has shipped to crates.io yet. The
notes below describe breaking changes that have landed on `main` since the
crate was created and will be folded into the eventual `0.1.0` release.

Removes the dynamic `Value` path. See
[`md/plans/2026-04-25-dynamic-path-removal.md`](../../md/plans/2026-04-25-dynamic-path-removal.md)
and [issue #267](https://github.com/elycruz/walrs/issues/267) for context.

### Removed (breaking)

- `FilterOp<Value>` and `TryFilterOp<Value>` impls, including the
  `apply_string_op_to_value` helper.
- `Filter<Value>` and `TryFilter<Value>` impl blocks for those types.
- The `value` feature flag.
- `bench_filter_op_value` benchmark group.

### Migration

Use `FilterOp<String>` (or numeric `FilterOp<T>`) on typed struct fields.
The recommended path is `#[derive(Fieldset)]` from `walrs_fieldset_derive`.

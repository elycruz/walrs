# Changelog

All notable changes to `walrs_fieldfilter` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The crate is still pre-publish; nothing has shipped to crates.io yet. The
notes below describe breaking changes that have landed on `main` since the
crate was created and will be folded into the eventual `0.1.0` release.

Removes the dynamic `FieldFilter` / `Value` path. See
[`md/plans/2026-04-25-dynamic-path-removal.md`](../../md/plans/2026-04-25-dynamic-path-removal.md)
and [issue #267](https://github.com/elycruz/walrs/issues/267) for context.

### Removed (breaking)

- `FieldFilter` type and the `field_filter` module.
- `CrossFieldRule` and `CrossFieldRuleType` types.
- `Field<Value>` impls (sync and async).
- Re-exports of `Value` and `ValueExt` from `walrs_validation`.
- `field_filter` example.
- `fuzz_fieldfilter_validate` fuzz target.

### Migration

Define a typed struct describing your fields and use `#[derive(Fieldset)]`
from `walrs_fieldset_derive`. Cross-field rules are expressed via the
`#[cross_validate(...)]` derive attribute.

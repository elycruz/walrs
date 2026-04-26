# Changelog

All notable changes to `walrs_fieldset_derive` are documented here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The crate is still pre-publish; nothing has shipped to crates.io yet. The
notes below describe breaking changes that have landed on `main` since the
crate was created and will be folded into the eventual `0.1.0` release.

Removes the dynamic `FormData` bridge. See
[`md/plans/2026-04-25-dynamic-path-removal.md`](../../md/plans/2026-04-25-dynamic-path-removal.md)
and [issue #267](https://github.com/elycruz/walrs/issues/267) for context.

### Removed (breaking)

- `#[fieldset(into_form_data)]` and `#[fieldset(try_from_form_data)]` struct
  attributes.
- `gen_form_data` codegen module (the `From<&T> for FormData` and
  `TryFrom<FormData> for T` impl emitters).

### Migration

The bridge attributes existed only to interoperate with the now-removed
`walrs_form::FormData`. Typed struct fields no longer need a dynamic
counterpart — derive `Fieldset` directly on the struct.

# Changelog

All notable changes to `walrs_acl` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The crate is still pre-publish; nothing has shipped to crates.io yet. The
notes below describe breaking changes that have landed on `main` since the
crate was created and will be folded into the eventual `0.1.0` release.

### Removed (breaking)

- `wasm` cargo feature.
- `cdylib` from `[lib] crate-type` (now defaults to `rlib` only).
- Optional WASM dependencies (`wasm-bindgen`, `serde-wasm-bindgen`, `js-sys`,
  `web-sys`). The WebAssembly bindings have moved to the sibling crate
  `walrs_acl_wasm`. See [issue #243](https://github.com/elycruz/walrs/issues/243).

### Changed (breaking)

- `Rule` no longer implements `Copy`. The new `Rule::AllowIf(AssertionKey)` and
  `Rule::DenyIf(AssertionKey)` variants carry a `String`, which is not `Copy`,
  so the enum as a whole cannot be `Copy` either. Downstream code that copied
  `Rule` values (e.g. `let r = *existing_rule;`) must now use `.clone()`.
  Introduced in PR #247 (closes #244).

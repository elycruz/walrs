# Changelog

All notable changes to `walrs_rbac` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The crate is still pre-publish; nothing has shipped to crates.io yet. The
notes below describe breaking changes that have landed on `main` since the
crate was created and will be folded into the eventual `0.1.0` release.

### Removed (breaking)

- `wasm` cargo feature.
- `cdylib` from `[lib] crate-type` (now defaults to `rlib` only).
- Optional WASM dependencies (`wasm-bindgen`, `serde-wasm-bindgen`, `js-sys`).
  The WebAssembly bindings have moved to the sibling crate `walrs_rbac_wasm`.
  See [issue #243](https://github.com/elycruz/walrs/issues/243).

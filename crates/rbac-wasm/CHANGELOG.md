# Changelog

All notable changes to `walrs_rbac_wasm` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-26

### Added

- Initial release. Extracted from `walrs_rbac` v0.2.0 — see
  [issue #243](https://github.com/elycruz/walrs/issues/243) and PR #282
  (which landed phase 2 of the extraction).
- `wasm-bindgen` bindings: `JsRbac`, `JsRbacBuilder`, plus convenience fns
  `createRbacFromJson` and `checkPermission`.
- JavaScript test suite in `tests-js/` (Node built-in test runner).

# Changelog

All notable changes to `walrs_acl_wasm` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-26

### Added

- Initial release. Extracted from `walrs_acl` — see
  [issue #243](https://github.com/elycruz/walrs/issues/243) and PR #283
  (which landed phase 1 of the extraction).
- `wasm-bindgen` bindings: `JsAcl`, `JsAclBuilder`, plus convenience fns
  `createAclFromJson` and `checkPermission`.
- JavaScript test suite in `tests-js/` (Node built-in test runner).

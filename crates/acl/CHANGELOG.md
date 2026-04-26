# Changelog

All notable changes to `walrs_acl` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-04-26

Coordinated pre-1.0 bump completing the WASM extraction cycle. See
[issue #243](https://github.com/elycruz/walrs/issues/243) for context. The
WebAssembly bindings have moved to a sibling crate; JavaScript consumers
should depend on `walrs_acl_wasm` instead.

### Removed (breaking)

- `wasm` cargo feature.
- `cdylib` from `[lib] crate-type` (now defaults to `rlib` only).
- Optional WASM dependencies (`wasm-bindgen`, `serde-wasm-bindgen`, `js-sys`,
  `web-sys`).

### Migration

```toml
# Before
walrs_acl = { version = "0.2", features = ["wasm"] }

# After (Rust consumers — no change needed beyond version bump)
walrs_acl = "0.3"

# After (WASM consumers — depend on the sibling crate)
walrs_acl_wasm = "0.1"
```

## [0.2.0] - 2026-04-26

Coordinated pre-1.0 bump alongside the rest of the workspace. No `walrs_acl`
API changes were driven by [issue #267](https://github.com/elycruz/walrs/issues/267);
the entry below predates this release and is included here for completeness.

### Changed (breaking)

- `Rule` no longer implements `Copy`. The new `Rule::AllowIf(AssertionKey)` and
  `Rule::DenyIf(AssertionKey)` variants carry a `String`, which is not `Copy`,
  so the enum as a whole cannot be `Copy` either. Downstream code that copied
  `Rule` values (e.g. `let r = *existing_rule;`) must now use `.clone()`.
  Introduced in PR #247 (closes #244).

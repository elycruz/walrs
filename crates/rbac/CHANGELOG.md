# Changelog

All notable changes to `walrs_rbac` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-04-26

Coordinated pre-1.0 bump completing the WASM extraction cycle. See
[issue #243](https://github.com/elycruz/walrs/issues/243) for context. The
WebAssembly bindings have moved to a sibling crate; JavaScript consumers
should depend on `walrs_rbac_wasm` instead.

### Removed (breaking)

- `wasm` cargo feature.
- `cdylib` from `[lib] crate-type` (now defaults to `rlib` only).
- Optional WASM dependencies (`wasm-bindgen`, `serde-wasm-bindgen`, `js-sys`).

### Migration

```toml
# Before
walrs_rbac = { version = "0.2", features = ["wasm"] }

# After (Rust consumers — no change needed beyond version bump)
walrs_rbac = "0.3"

# After (WASM consumers — depend on the sibling crate)
walrs_rbac_wasm = "0.1"
```

## [0.2.0] - 2026-04-26

Coordinated pre-1.0 bump alongside the rest of the workspace as part of
[issue #267](https://github.com/elycruz/walrs/issues/267) (Phase 4, b66ea99).
No `walrs_rbac` API changes — this entry is included for changelog completeness.

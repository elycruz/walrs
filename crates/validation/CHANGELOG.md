# Changelog

## Unreleased

### Added

- New `value` feature flag (default-on) gating the dynamic `Value` enum, its
  `Rule<Value>` / `Condition<Value>` dispatch, the `value!` macro, and the
  `IsEmpty` implementation for `Value`. Typed-only consumers can now opt out
  via `default-features = false` to avoid compiling the dynamic path.
- `serde_json_bridge` now implies `value` (the bridge converts `serde_json::Value`
  to/from `walrs_validation::Value`, so it cannot function without it).

### Changed

- Default-build behavior is unchanged: `default = ["value", "serde_json_bridge"]`
  still enables the dynamic path out of the box. Existing code that relies on
  default features continues to compile with no action required.

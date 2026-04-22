# Changelog

## Unreleased

### Added

- New `value` feature flag (default-on) gating `FilterOp<Value>` and
  `TryFilterOp<Value>`. The feature forwards to `walrs_validation/value`,
  so typed-only consumers can disable it via `default-features = false` to
  build with just `FilterOp<String>` and scalar numeric types.

### Changed

- Default-build behavior is unchanged: `default = ["validation", "value"]`
  still enables the dynamic path out of the box.

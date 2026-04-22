# Changelog

All notable changes to `walrs_acl` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed (breaking)

- `Rule` no longer implements `Copy`. The new `Rule::AllowIf(AssertionKey)` and
  `Rule::DenyIf(AssertionKey)` variants carry a `String`, which is not `Copy`,
  so the enum as a whole cannot be `Copy` either. Downstream code that copied
  `Rule` values (e.g. `let r = *existing_rule;`) must now use `.clone()`.
  Introduced in PR #247 (closes #244).

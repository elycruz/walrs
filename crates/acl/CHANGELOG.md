# Changelog

All notable changes to `walrs_acl` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this crate adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

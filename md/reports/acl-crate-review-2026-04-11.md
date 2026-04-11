# ACL Crate (`walrs_acl`) — Code Review

**Date:** 2026-04-11
**Scope:** `crates/acl/src/` — all source files listed in #163
**Focus:** Correctness, soundness, error handling, type safety, WASM, tests, docs

---

## Summary

| Severity | Count |
|---|---|
| 🔴 Critical | 0 |
| 🟠 High | 2 |
| 🟡 Medium | 5 |
| 🔵 Low | 5 |
| ✅ Clean | 2 files (`types.rs`, `rule.rs`) |

**No unsafe code.** No unintended privilege escalation paths found.
All 48 unit tests + 20 doc-tests pass. `walrs_digraph` is used correctly
for role/resource DAG inheritance and cycle detection.

---

## 🟠 High (2)

### 1. `wasm.rs` — Builder methods panic instead of returning `Result`

**Lines:** `wasm.rs:166-174`, `193-209`, `220-231`, `247-266`

`JsAclBuilder::add_role`, `add_roles`, `add_resource`, and `add_resources`
all call `unwrap_or_else(|e| panic!("{}", e))`. In a WASM context, panics
become `RuntimeError` which is **uncatchable** in JavaScript `try/catch`
(it kills the WASM instance). These should return `Result<Self, JsValue>`
to match the error-handling pattern already used by `allow()`, `deny()`,
and `build()` in the same file.

**Impact:** Any invalid input (e.g., adding a role with a non-existent parent)
crashes the WASM module irrecoverably.

**Fix:** Change the four methods to return `Result<Self, JsValue>` and
replace `unwrap_or_else(|e| panic!(...))` with `.map_err(|e| JsValue::from_str(&e))?`.

---

### 2. `acl_builder.rs:386-398` — Silent rule dropping for non-existent roles/resources

`_get_only_keys_in_graph` filters out roles/resources not present in the
graph, and `allow()`/`deny()` return `Ok(self)` even when **all** supplied
keys were filtered out (i.e., none existed). The rule is silently not applied.

**Reproduction:**

```rust
let acl = AclBuilder::new()
    .add_role("guest", None)?
    .add_resource("blog", None)?
    .allow(Some(&["typo_role"]), Some(&["blog"]), Some(&["read"]))? // silently ignored
    .build()?;

// Expect: error about "typo_role" not existing
// Actual: Ok, but rule was not applied
```

**Impact:** Typos in role/resource names go undetected. Users believe rules are
applied when they are not.

**Fix:** Return `Err(String)` (or at least warn) when all supplied keys are
filtered out, or validate keys upfront in `allow()`/`deny()`.

---

## 🟡 Medium (5)

### 3. `resource_role_rules.rs:40-47` — `get_or_create_role_privilege_rules_mut` never creates

The method name says "get or create" but its body is **identical** to
`get_role_privilege_rules_mut` (lines 31-38). For a non-existent resource,
it returns `&mut self.for_all_resources` instead of inserting a new entry.

**Impact:** Misleading API; callers expecting lazy insertion get the global
fallback instead.

**Fix:** Either implement actual insert-on-miss semantics, or remove the
method and use `get_role_privilege_rules_mut` directly.

---

### 4. `resource_role_rules.rs:56-66` — `set_role_privilege_rules` returns wrong scope for empty slice

When `resources = Some(&[])` (empty slice), the method falls into the
`Some(resource_ids)` match arm, correctly sets `self.for_all_resources`,
but returns `RuleContextScope::PerSymbol`. It should return
`RuleContextScope::ForAllSymbols` since it set the "for all" value.

```rust
// Line 66: always returns PerSymbol for Some(_), even when the slice is empty
RuleContextScope::PerSymbol
```

**Fix:** Move the `if !resource_ids.is_empty()` check before the return,
or return `ForAllSymbols` in the empty-slice branch.

---

### 5. `README.md:41` — Broken code examples

```rust
acl.is_allowed(admin("guest"), Some("public"), Some("read"))? // invalid syntax
```

Should be `acl.is_allowed(Some("guest"), ...)`. Also:

- Line 52: `use walrs_acl::{AclBuilder};` should be
  `use walrs_acl::simple::AclBuilder;` (the re-export path).
- Line 41: the trailing `?` on `is_allowed` is incorrect — it returns `bool`,
  not `Result`.

---

### 6. `acl_builder.rs:664` — `TryFrom<&AclBuilder> for AclData` is incomplete

The implementation is marked with:

```rust
// TODO finalize implementation (still in progress).
```

This is public API that may produce incomplete or incorrect `AclData`
(e.g., the `Deny`-as-default elision logic on line 722 could lose explicit
deny rules in round-trip scenarios).

**Fix:** Either complete and test the implementation, or mark it
`#[doc(hidden)]` / gate it behind a feature flag until finalized.

---

### 7. `lib.rs:2-3` — Crate-wide `#[allow(dead_code)]` / `#[allow(unused_variables)]`

```rust
#![allow(dead_code)]
#![allow(unused_variables)]
```

These suppress warnings across the entire crate, hiding potentially dead
or unused code. In a security-relevant crate (ACL), this is risky.

**Fix:** Remove the crate-level allows and address individual warnings,
or scope them to specific items during development.

---

## 🔵 Low (5)

### 8. `wasm.rs:49-55` — `to_json()` stub always returns `Err`

`JsAcl::to_json()` is a public method that unconditionally returns
`Err("Direct ACL serialization not yet implemented...")`. Shipping a
public API that always errors is confusing.

**Fix:** Implement it (leveraging `TryFrom<&AclBuilder> for AclData` once
finalized), or remove it from the public API.

---

### 9. `acl.rs:281` — Broken intra-doc link

```
/// should be used before using the [acl] structure
```

`cargo doc` warns: `no item named 'acl' in scope`. Use `[Acl]` or
`` [`Acl`] `` instead.

---

### 10. 17 clippy warnings

`cargo clippy -p walrs_acl -- -W clippy::all` reports 17 warnings:

- 5× collapsible `if` statements (`acl_builder.rs`)
- 5× "very complex type" (`acl_builder.rs`, `acl_data.rs`)
- 1× missing `Default` impl for `ResourceRoleRules`
- 1× unnecessary `unwrap` after `is_some` check (`role_privilege_rules.rs:76`)
- 3× `map_or` simplifications
- 1× elidable lifetime (`acl_data.rs:21`)
- 1× redundant deref (`acl_builder.rs:394`)

None are correctness bugs, but they indicate areas where the code could be
cleaner and more idiomatic.

---

### 11. `acl.rs:89` — Redundant `.as_ref()` on `&str`

```rust
pub fn has_role(&self, role: &str) -> bool {
    self._roles.has_vertex(role.as_ref()) // .as_ref() is a no-op here
}
```

`role` is already `&str`; `.as_ref()` is redundant.

---

### 12. `acl_builder.rs:277-286` — Undocumented global reset behavior

When all parameters to `_add_rule` are empty/None, the entire `_rules`
structure is replaced with a fresh `ResourceRoleRules::new()`:

```rust
if _is_empty(&roles) && _is_empty(&resources) && _is_empty(&privileges) {
    self._rules = ResourceRoleRules::new();
    // ...
}
```

This wipes all previously configured per-resource and per-role rules.
The `allow()`/`deny()` doc comments don't mention this reset behavior.

**Impact:** `builder.allow(None, None, None)?` silently destroys all
prior rules, which may surprise users.

---

## ✅ Clean Files

- **`types.rs`** — Simple type aliases, no issues.
- **`rule.rs`** — Clean enum definitions, no issues.

---

## Test Coverage Assessment

The crate has good test coverage:

- **Unit tests:** `privilege_rules` (4 tests), `role_privilege_rules` (4 tests),
  `resource_role_rules` (3 tests), `acl` (14 tests) — all in-module.
- **Integration tests:** `acl_test.rs` (4 tests), `acl_builder_test.rs` (extensive),
  `opposing_rules_test.rs` (5 tests).
- **Doc-tests:** 20 passing.

**Gaps:**
- No tests for non-existent role/resource being silently dropped (High #2).
- No tests for `_get_only_keys_in_graph` returning empty vec when all keys filtered.
- WASM module (`wasm.rs`) has no Rust-side tests (only JS tests in `tests-js/`).
- `TryFrom<&AclBuilder> for AclData` round-trip fidelity is not tested.

---

## Dependency Review: `walrs_digraph`

Used correctly for:
- `DisymGraph` (symbol graph) for role and resource DAG representation.
- `DirectedPathsDFS` for inheritance path queries.
- `DirectedCycle` for cycle detection during `build()`.

No misuse or incorrect API calls observed.

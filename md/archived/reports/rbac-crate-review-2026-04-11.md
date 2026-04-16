# RBAC Crate (`walrs_rbac`) — Code Review

**Date:** 2026-04-11
**Issue:** #168
**Scope:** `crates/rbac/src/` — all source files
**Focus:** Correctness, soundness, security, edge cases

---

## Summary

| Severity | Count |
|---|---|
| 🔴 Critical | 0 |
| 🟠 High | 1 |
| 🟡 Medium | 3 |
| 🔵 Low | 4 |
| ✅ Clean | 2 files |

---

## Coverage

| File | Line % | Function % |
|---|---|---|
| `error.rs` | 100.00% | 100.00% |
| `rbac.rs` | 100.00% | 100.00% |
| `rbac_builder.rs` | 98.20% | 97.83% |
| `rbac_data.rs` | 97.04% | 84.00% |
| `role.rs` | 100.00% | 100.00% |
| `wasm.rs` | 0.00% | 0.00% |

104 tests (72 unit + 32 doc-tests). All pass.

---

## 🟠 High (1)

### 1. `role.rs:135` / `rbac.rs:158` — Stack overflow on deeply nested `Role` trees via deserialization

`has_permission_recursive` is a recursive DFS over the `Role` tree (children are
`Vec<Role>`, embedded by value). There is no depth limit.

Since `Role` derives `Deserialize`, an attacker providing untrusted JSON/YAML
input can craft an arbitrarily deep nesting of roles, causing a **stack overflow**
(process abort, not catchable) when `is_granted` is called.

**Reproduction (conceptual):**

```rust
// Deserialize from malicious JSON with 100,000 levels of nesting
let json = r#"{"name":"r","permissions":[],"children":[{"name":"r","permissions":["secret"],"children":[...]}]}"#;
let role: Role = serde_json::from_str(&json).unwrap();
role.has_permission_recursive("secret"); // stack overflow
```

The same risk exists for manually constructed roles via the public `Role::add_child` API,
though this requires adversarial programmatic use.

**Note:** The `RbacBuilder` path is safe — it builds from a flat role map and `check_for_cycles`
prevents infinite recursion. The risk is limited to direct `Role` deserialization from untrusted
input.

**Suggested fix:** Add a depth limit to `has_permission_recursive`, or use an iterative
traversal:

```rust
pub fn has_permission_recursive(&self, permission: &str) -> bool {
    let mut stack = vec![self];
    while let Some(role) = stack.pop() {
        if role.permissions.contains(permission) {
            return true;
        }
        stack.extend(role.children.iter());
    }
    false
}
```

---

## 🟡 Medium (3)

### 2. `rbac_builder.rs:226–253` — Dead `_visit_stack` parameter in `resolve_role`

The `resolve_role` method accepts a `_visit_stack: &mut Vec<String>` parameter that is
**never read** (prefixed with `_` to suppress the warning). On line 247, each recursive
call passes `&mut Vec::new()` instead of forwarding the existing stack:

```rust
let child = self.resolve_role(child_name, resolved, &mut Vec::new())?;
```

This means `resolve_role` has **no cycle protection of its own**. It relies entirely on
`check_for_cycles()` being called first (which it is, in `build()`). If someone were to
call `resolve_role` directly (it's private, so this is mitigated), it would infinite-loop
on cycles.

The dead parameter is misleading — it suggests the method tracks visited nodes when it
does not. It should be removed for clarity.

**Suggested fix:** Remove the `_visit_stack` parameter entirely since `check_for_cycles()`
handles cycle detection:

```rust
fn resolve_role(&self, name: &str, resolved: &mut HashMap<String, Role>) -> Result<Role> {
    // ... same logic without _visit_stack
}
```

---

### 3. `rbac_builder.rs:75–88` — `add_role` silently overwrites existing roles

`add_role` uses `HashMap::insert`, which silently replaces any existing role with the
same name. This means a second `add_role("admin", ...)` call discards the first one
without warning.

While this is tested and documented in `rbac.rs` tests (`test_replace_existing_role`),
the **builder** does not offer any feedback that an overwrite occurred. In a configuration
loaded from a file, a duplicate role name (likely a typo) would silently drop permissions
from the first definition.

**Suggested fix:** Either return an error for duplicate role names, or log/warn. At
minimum, document this behavior on `RbacBuilder::add_role`:

```rust
/// Note: If a role with the same name already exists, it will be replaced.
```

---

### 4. `rbac_builder.rs:191–223` — `build()` creates O(n²) cloned `Role` objects for linear chains

The `build()` method resolves roles by cloning child subtrees into parent roles. In a
linear chain of depth D (e.g., `guest <- user <- editor <- admin`), each ancestor
embeds a full clone of all descendants:

- `admin` contains clones of `editor`, `user`, `guest`
- `editor` contains clones of `user`, `guest`
- `user` contains a clone of `guest`
- Total Role objects: D + (D-1) + ... + 1 = O(D²)

Additionally, all roles (including children) are stored in the top-level `HashMap`,
so every role exists both standalone AND embedded in each ancestor.

For typical RBAC hierarchies (< 20 roles), this is negligible. For very large
hierarchies, memory usage grows quadratically.

**Not a correctness bug.** Performance/memory concern only.

---

## 🔵 Low (4)

### 5. `Cargo.toml:21` — Redundant `serde_derive` dependency

The crate declares both `serde = { features = ["derive"] }` and `serde_derive = "1.0.228"`.
The `derive` feature on `serde` already re-exports `Serialize`/`Deserialize` derive macros.
The separate `serde_derive` dependency is redundant.

Source files import from `serde_derive` directly (e.g., `use serde_derive::{Deserialize,
Serialize}`) instead of `use serde::{Deserialize, Serialize}`.

**Suggested fix:** Remove `serde_derive` from `[dependencies]` and change imports to use
`serde` directly.

---

### 6. `lib.rs:2` — Global `#![allow(unused_imports)]` suppresses useful warnings

This blanket allow could mask real issues where imports are accidentally left behind
after refactoring. It was likely added to suppress warnings from the `prelude` module's
conditional imports.

**Suggested fix:** Use targeted `#[allow(unused_imports)]` only on the `prelude` module.

---

### 7. `lib.rs:7` — Unnecessary `extern crate core`

`extern crate core;` is unnecessary since Rust edition 2018. This crate uses edition
2024 where `core` is always in scope.

**Suggested fix:** Remove the line.

---

### 8. `wasm.rs` — 0% test coverage

The entire `wasm.rs` module (137 lines, 25 functions) has 0% test coverage. While WASM
bindings are difficult to test in a standard `cargo test` environment, the core logic
(JSON parsing → builder → Rbac → permission checks) could be tested by extracting shared
logic or adding `#[cfg(test)]` tests that exercise the non-WASM parts.

The `JsRbacBuilder::add_role` method (line 116) takes `self` by value (move semantics
for WASM chaining), which is correct for JS interop but prevents reuse in Rust.

---

## ✅ Clean Files

### `error.rs`

No issues found. Well-designed error enum with proper `Display` impl, `Clone`, `PartialEq`,
and conditional `std::error::Error` impl behind the `std` feature gate. All 8 tests pass
with 100% coverage.

### `role.rs`

No issues found (aside from the stack overflow concern in High #1, which is a usage
pattern issue rather than a bug in `Role` itself). Clean API design with `HashSet`-based
permissions (deduplication), chaining methods, and thorough tests. 100% coverage.

---

## Recommendations (Priority Order)

1. **Make `has_permission_recursive` iterative** (High) — eliminates stack overflow risk
   from deserialized or manually constructed deep hierarchies.
2. **Remove dead `_visit_stack` parameter** (Medium) — reduces code confusion and
   eliminates a misleading API surface.
3. **Document or warn on duplicate role names in builder** (Medium) — prevent silent
   permission loss from configuration typos.
4. **Remove redundant `serde_derive` dep** (Low) — cleaner dependency tree.
5. **Narrow `#![allow(unused_imports)]` scope** (Low) — better warning hygiene.
6. **Remove `extern crate core`** (Low) — dead code.
7. **Add WASM test coverage** (Low) — 0% is a gap, even if hard to test.

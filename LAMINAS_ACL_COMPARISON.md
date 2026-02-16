# Comparison: walrs ACL vs Laminas ACL

This document compares the `walrs_acl` crate (Rust) with the
[`laminas-permissions-acl`](https://github.com/laminas/laminas-permissions-acl) library (PHP).
Both provide Access Control List (ACL) functionality with support for roles, resources,
privileges, and hierarchical inheritance.

---

## 1. High-Level Overview

| Aspect | walrs ACL | Laminas ACL |
|---|---|---|
| **Language** | Rust | PHP |
| **Paradigm** | Builder → immutable ACL | Mutable object, modify at any time |
| **Core Model** | Roles, Resources, Privileges with Allow/Deny rules | Roles, Resources, Privileges with Allow/Deny rules |
| **Role Inheritance** | DAG (directed acyclic graph) via `DisymGraph` | DAG via `Role\Registry` |
| **Resource Inheritance** | DAG via `DisymGraph` | Tree (single parent per resource) |
| **Cycle Detection** | Explicit check at build time | Prevented by single-parent constraint |
| **Default Policy** | Deny-by-default | Deny-by-default |
| **Assertions / Conditional Rules** | Not supported | Supported via `AssertionInterface` |
| **Serialization** | JSON (serde) with optional YAML | PHP serialization; no built-in JSON/YAML |
| **WASM Support** | Yes (optional feature) | No |
| **no_std Support** | Yes (alloc-based) | N/A |

---

## 2. Architecture & Design

### walrs ACL

The walrs ACL uses a **two-phase design**:

1. **Build phase** — `AclBuilder` accumulates roles, resources, and rules.
   All mutations happen here. Cycle detection runs at `build()` time.
2. **Query phase** — `Acl` is an immutable struct. It can only be queried
   (`is_allowed`, `has_role`, `inherits_role`, etc.). No further modifications
   are possible after construction.

Key types:
- `Acl` — immutable query engine
- `AclBuilder` — fluent builder with `Result<&mut Self, String>` chaining
- `AclData` — serializable representation for JSON I/O
- `Rule` enum — `Allow` or `Deny`
- `ResourceRoleRules` → `RolePrivilegeRules` → `PrivilegeRules` — three-level
  nested rule storage

### Laminas ACL

Laminas uses a **single mutable object**:

The `Acl` class handles role registration, resource registration, rule
management, and permission queries all in one place. The ACL can be modified
at any point during its lifetime.

Key types:
- `Acl` — main class (mutable, handles everything)
- `AclInterface` — contract with `hasResource()` and `isAllowed()`
- `RoleInterface` / `GenericRole` — role abstraction
- `ResourceInterface` / `GenericResource` — resource abstraction
- `AssertionInterface` — runtime conditional rules
- `Role\Registry` — role hierarchy management

---

## 3. Roles

| Feature | walrs ACL | Laminas ACL |
|---|---|---|
| **Type** | `String` alias | `RoleInterface` (or string identifier) |
| **Multiple Parents** | Yes (DAG) | Yes (DAG) |
| **Add** | `builder.add_role("name", parents)` | `$acl->addRole($role, $parents)` |
| **Remove** | Not supported (immutable after build) | `$acl->removeRole($role)` |
| **Query** | `acl.has_role("name")` | `$acl->hasRole($role)` |
| **Inheritance Check** | `acl.inherits_role("child", "parent")` | `$acl->inheritsRole($child, $parent)` |
| **List Parents** | Not directly exposed | `$acl->getRole($role)->getParents()` |
| **Batch Add** | `builder.add_roles(&[...])` | Not built-in (loop required) |

**Notable differences:**
- Laminas roles are objects implementing `RoleInterface`, allowing custom role
  classes that carry additional data. walrs uses plain `String` identifiers.
- Laminas supports removing roles at runtime; walrs does not (the ACL is
  immutable once built).

---

## 4. Resources

| Feature | walrs ACL | Laminas ACL |
|---|---|---|
| **Type** | `String` alias | `ResourceInterface` (or string identifier) |
| **Multiple Parents** | Yes (DAG) | No (single parent — tree structure) |
| **Add** | `builder.add_resource("name", parents)` | `$acl->addResource($resource, $parent)` |
| **Remove** | Not supported | `$acl->removeResource($resource)` |
| **Remove All** | Not supported | `$acl->removeResourceAll()` |
| **Query** | `acl.has_resource("name")` | `$acl->hasResource($resource)` |
| **Inheritance Check** | `acl.inherits_resource("child", "parent")` | `$acl->inheritsResource($child, $parent)` |

**Notable differences:**
- walrs supports multiple parent resources (DAG), whereas Laminas restricts
  resources to a single parent (tree hierarchy). This gives walrs more
  flexibility for modeling complex resource relationships.
- Laminas allows runtime removal of individual resources or all resources;
  walrs does not.

---

## 5. Rules (Allow / Deny)

| Feature | walrs ACL | Laminas ACL |
|---|---|---|
| **Allow** | `builder.allow(roles, resources, privileges)` | `$acl->allow($roles, $resources, $privileges)` |
| **Deny** | `builder.deny(roles, resources, privileges)` | `$acl->deny($roles, $resources, $privileges)` |
| **Remove Allow** | Not supported | `$acl->removeAllow(...)` |
| **Remove Deny** | Not supported | `$acl->removeDeny(...)` |
| **Wildcard (all)** | `None` parameter | `null` parameter |
| **Assertions** | Not supported | 4th parameter: `AssertionInterface` |
| **Rule Override** | Setting a new rule overwrites the old one | Setting a new rule overwrites the old one |

**API comparison:**

```rust
// walrs ACL
builder.allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?;
builder.deny(Some(&["guest"]), Some(&["admin_panel"]), None)?;
```

```php
// Laminas ACL
$acl->allow('user', 'blog', ['read', 'write']);
$acl->deny('guest', 'admin_panel');
```

---

## 6. Permission Queries

| Feature | walrs ACL | Laminas ACL |
|---|---|---|
| **Basic Check** | `acl.is_allowed(role, resource, privilege)` | `$acl->isAllowed($role, $resource, $privilege)` |
| **Multi-check** | `acl.is_allowed_any(roles, resources, privileges)` | Not built-in (loop required) |
| **Null/None = all** | Yes | Yes |
| **Return Type** | `bool` | `bool` |
| **Traversal** | DFS through role & resource inheritance | DFS through role inheritance |

**Notable differences:**
- walrs provides `is_allowed_any()` which checks multiple role/resource/privilege
  combinations in one call. Laminas has no equivalent.
- Both use depth-first search for inheritance traversal.

---

## 7. Assertions (Conditional Rules)

This is the most significant feature gap between the two implementations.

**Laminas** supports assertions — runtime-evaluated conditions attached to rules:

```php
$acl->allow('user', 'document', 'edit', new OwnershipAssertion());
```

Built-in assertion types:
- `AssertionInterface` — base interface with `assert()` method
- `CallbackAssertion` — wraps a closure
- `ExpressionAssertion` — expression-based conditions
- `OwnershipAssertion` — checks resource ownership
- `AssertionAggregate` — combines multiple assertions (AND/OR)
- `AssertionManager` — service-locator for assertions

**walrs** does not have an assertion mechanism. All rules are static
(determined at build time). Dynamic/conditional access control must be
implemented in application code outside the ACL.

---

## 8. Mutability & Lifecycle

| Aspect | walrs ACL | Laminas ACL |
|---|---|---|
| **Construction** | `AclBuilder` → `Acl` (two-phase) | `new Acl()` then mutate (single object) |
| **Post-construction mutation** | Not supported (immutable) | Fully supported |
| **Thread safety** | Safe for concurrent reads (immutable) | Not thread-safe (mutable shared state) |
| **Rebuild from existing** | `AclBuilder::try_from(&acl)` | Mutate in place |
| **Cycle safety** | Validated at build time | Prevented by structural constraints |

The walrs approach trades runtime flexibility for thread safety and
predictability. Once built, an `Acl` can be shared across threads without
synchronization. The Laminas approach is more flexible but requires care in
concurrent environments.

---

## 9. Serialization & Data Loading

| Aspect | walrs ACL | Laminas ACL |
|---|---|---|
| **JSON** | Built-in via `serde_json` and `AclData` | Not built-in |
| **YAML** | Not in ACL crate (available in RBAC sibling) | Not built-in |
| **From File** | `AclBuilder::try_from(&mut File)` | Manual (developer responsibility) |
| **PHP Serialize** | N/A | Supported |
| **Custom Format** | Via `AclData` struct + serde | Via PHP serialization |

walrs provides a structured JSON format for defining ACLs declaratively:

```json
{
  "roles": [["guest", null], ["user", ["guest"]]],
  "resources": [["blog", null]],
  "allow": [["blog", [["user", ["read", "write"]]]]],
  "deny": [["admin_panel", [["guest", null]]]]
}
```

Laminas has no equivalent built-in serialization; persistence is left to the
developer.

---

## 10. WASM & Platform Support

| Aspect | walrs ACL | Laminas ACL |
|---|---|---|
| **WASM** | Yes (via `wasm-bindgen`) | No |
| **no_std** | Yes (alloc-based) | N/A |
| **Browser** | Via WASM bindings | Via PHP server only |
| **Embedded** | Possible (no_std) | No |

walrs provides `JsAcl` and `JsAclBuilder` wrappers for JavaScript
interoperability through WebAssembly.

---

## 11. Error Handling

| Aspect | walrs ACL | Laminas ACL |
|---|---|---|
| **Type** | `Result<T, String>` | PHP Exceptions |
| **Cycle errors** | At build time | At add time (structural prevention) |
| **Missing role/resource** | Error on `allow()`/`deny()` | Exception on `isAllowed()` |
| **Safe variants** | `inherits_role_safe()` returns `Result` | Exceptions only |

---

## 12. Summary of Feature Gaps

### Features in Laminas not present in walrs

| Feature | Impact | Notes |
|---|---|---|
| **Assertions / Conditional Rules** | High | Most significant gap; enables runtime conditions |
| **Runtime Mutation** | Medium | `removeRole()`, `removeResource()`, `removeAllow()`, `removeDeny()` |
| **Role/Resource Interfaces** | Low | Allows custom role/resource objects with metadata |
| **Role listing / enumeration** | Low | `getRoles()`, `getResources()` |
| **Ownership model** | Low | `ProprietaryInterface` for resource ownership |

### Features in walrs not present in Laminas

| Feature | Impact | Notes |
|---|---|---|
| **Multi-parent resources** | Medium | DAG vs tree for resources |
| **`is_allowed_any()` batch check** | Medium | Check multiple combinations at once |
| **WASM / JavaScript bindings** | Medium | Browser & Node.js usage |
| **Built-in JSON serialization** | Medium | Declarative ACL definition from files |
| **Immutable query object** | Medium | Thread-safe by design |
| **no_std support** | Low | Embedded / constrained environments |
| **Batch role/resource addition** | Low | `add_roles()`, `add_resources()` |
| **Explicit cycle detection** | Low | Multi-parent DAG requires this |

---

## 13. Recommendations

1. **Assertions**: Consider adding an assertion/predicate mechanism to walrs
   ACL. This could be implemented as an optional closure or trait object passed
   to `is_allowed()`, rather than attaching to rules at build time, to preserve
   the immutable design.

2. **Role/Resource enumeration**: Adding `roles()` and `resources()` iterators
   to the `Acl` struct would improve introspection capabilities.

3. **Rule removal in builder**: The `AclBuilder` could support `remove_allow()`
   and `remove_deny()` methods for use during the build phase, giving more
   flexibility when composing ACLs from multiple sources.

4. **Keep the immutable design**: The two-phase builder/query pattern is a
   strength that enables thread safety and predictable behavior. This is a
   deliberate advantage over Laminas's mutable approach.

5. **Keep multi-parent resources**: The DAG-based resource hierarchy is more
   expressive than Laminas's single-parent tree and should be preserved.

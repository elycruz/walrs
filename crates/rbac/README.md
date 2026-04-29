# walrs_rbac

Role-Based Access Control (RBAC) permissions management.

Provides a lightweight implementation of [Role-Based Access Control](https://en.wikipedia.org/wiki/Role-based_access_control) (RBAC), where permissions are attached to roles and roles can inherit permissions from child roles.

## Usage

Instantiate your `Rbac` struct — add `Role`s with permissions and child relationships — then query it from a middleware/application context.

**Inline declaration:**

```rust
use walrs_rbac::RbacBuilder;

fn main() -> Result<(), walrs_rbac::RbacError> {
    let rbac = RbacBuilder::new()
        // Add roles with permissions and inheritance
        .add_role("guest", &["read.public"], None)?
        .add_role("user", &["write.post", "comment.post"], Some(&["guest"]))?
        .add_role("editor", &["edit.post", "publish.post"], Some(&["user"]))?
        .add_role("admin", &["admin.panel", "manage.users"], Some(&["editor"]))?
        .build()?;

    // In an application context...
    assert!(rbac.is_granted("admin", "read.public"));   // inherited via editor <- user <- guest
    assert!(rbac.is_granted("admin", "admin.panel"));    // direct permission
    assert!(!rbac.is_granted("guest", "admin.panel"));   // guest has no admin access

    Ok(())
}
```

**From *.json representation:**

```rust
use std::convert::TryFrom;
use std::fs::File;
use walrs_rbac::RbacBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "./test-fixtures/example-rbac.json";
    let mut f = File::open(&file_path)?;
    let rbac = RbacBuilder::try_from(&mut f)?.build()?;
    // ...
    Ok(())
}
```

### Construction

The RBAC can be constructed:

- using the `RbacBuilder` structure (fluent interface).
- from a \*.json representation, using `RbacBuilder::try_from(&mut File)?.build()`.
- from a deserialized `RbacData`, using `RbacBuilder::try_from(&RbacData)?.build()` (`RbacData` can be parsed from JSON via `RbacData::from_json` or, with the `yaml` feature, from YAML via `RbacData::from_yaml`).

### JSON Representation

This represents an [`RbacData`](src/rbac_data.rs) struct.

```json5
{
  "roles": [
    // Each role is [name, [permissions], children_or_null]
    ["guest", ["read.public"], null],
    ["user", ["write.post", "comment.post"], ["guest"]],     // user inherits from guest
    ["editor", ["edit.post", "publish.post"], ["user"]],     // editor inherits from user
    ["admin", ["admin.panel", "manage.users"], ["editor"]]   // admin inherits from editor
  ]
}
```

### YAML Representation

Requires the `yaml` feature flag.

```yaml
roles:
  - - guest
    - - read.public
    - null
  - - user
    - - write.post
      - comment.post
    - - guest
  - - admin
    - - admin.panel
    - - user
```

## How it works

RBAC uses a role hierarchy where permissions are attached to roles and inherited through parent-child relationships:

- **Roles** have a name, a set of permissions, and optional child roles.
- **Permissions** are arbitrary strings (e.g., `"edit.article"`, `"admin.panel"`).
- **Inheritance**: A parent role inherits all permissions from its children (and their children, recursively).

The `is_granted(role, permission)` method checks if a role has a permission either directly or through inheritance.

### Key API

| Method | Description |
|--------|-------------|
| `RbacBuilder::new()` | Create a new builder |
| `.add_role(name, permissions, children)` | Add a role |
| `.add_roles(&[...])` | Add multiple roles |
| `.add_permission(role, permission)` | Add a permission to a role |
| `.add_child(parent, child)` | Add a child relationship |
| `.build()` | Build the final `Rbac` (validates for cycles) |
| `rbac.is_granted(role, permission)` | Check if a permission is granted |
| `rbac.is_granted_safe(role, permission)` | Same, but returns `Result` if role doesn't exist |
| `rbac.has_role(role)` | Check if a role exists |
| `rbac.get_role(role)` | Get a reference to a `Role` |
| `rbac.role_count()` | Get the number of roles |
| `rbac.role_names()` | Iterator over role names |

See tests, [benchmarks](benchmarks), and/or [examples](examples) for more details.

## Public API surface

Top-level re-exports from `walrs_rbac` (see `src/lib.rs`):

- **Core**: `Rbac`, `RbacBuilder`, `Role`
- **Data**: `RbacData` (serde-serializable representation; supports JSON and, with `yaml`, YAML)
- **Errors**: `RbacError`, `Result` (alias for `core::result::Result<T, RbacError>`)

`RbacError` variants: `RoleNotFound`, `CycleDetected`, `InvalidConfiguration`, `DeserializationError`, `SerializationError`.

## Features

| Feature | Default | Enables |
|---|---|---|
| `std` | yes | Standard-library support: file I/O (`TryFrom<&mut File>` for `RbacData`/`RbacBuilder`), `std::error::Error` impl on `RbacError`, `HashMap` storage. |
| `yaml` | no | YAML serialization/deserialization on `RbacData` (`from_yaml` / `to_yaml`) via `serde_yaml`. |

Disabling default features (`default-features = false`) drops `std` and switches the crate to `no_std + alloc` mode (uses `BTreeMap` instead of `HashMap`); this is the configuration used by `walrs_rbac_wasm`.

### Installation

```toml
# Default (JSON support, std)
[dependencies]
walrs_rbac = "0.1.0"

# With YAML support
[dependencies]
walrs_rbac = { version = "0.1.0", features = ["yaml"] }

# no_std / WASM-compatible (alloc only)
[dependencies]
walrs_rbac = { version = "0.1.0", default-features = false }
```

## Examples

Runnable examples live in [`examples/`](./examples/). Run any of them with `cargo run -p walrs_rbac --example <name>`.

| Example | Demonstrates | Run command |
|---|---|---|
| `rbac_builder_example` | Fluent `RbacBuilder` usage and `is_granted` checks | `cargo run -p walrs_rbac --example rbac_builder_example` |
| `rbac_try_from_json` | Loading an RBAC from a JSON file via `TryFrom<&mut File>` | `cargo run -p walrs_rbac --example rbac_try_from_json` (run from `crates/rbac/` so the fixture path resolves) |

## WASM Support

`walrs_rbac` is `no_std`-compatible (with `alloc`) when built with `default-features = false`. JavaScript/TypeScript bindings live in the companion crate [`walrs_rbac_wasm`](../rbac-wasm/README.md), which exposes `JsRbac`, `JsRbacBuilder`, and convenience helpers via `wasm-bindgen`.

To build the WASM bindings, work in `crates/rbac-wasm/` (see its README).

## Prior Art

This crate is inspired by:

- **[laminas-permissions-rbac](https://github.com/laminas/laminas-permissions-rbac)** (PHP) - The primary reference implementation. Created and maintained by the [Laminas Project](https://getlaminas.org/) (formerly Zend Framework). Licensed under BSD-3-Clause.
- **[Role-Based Access Control](https://en.wikipedia.org/wiki/Role-based_access_control)** - The RBAC security model.

## License

Elastic-2.0

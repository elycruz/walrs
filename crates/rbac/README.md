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
- from a \*.json representation, using `RbacBuilder::try_from(RbacData)?.build()` (`RbacData` can also be constructed from a \*.json file).
- from a \*.yaml representation (behind the `yaml` feature flag), using `RbacData::from_yaml()`.

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

See tests, [benchmarks](benchmarks), and/or [examples](examples) for more details.

## Features

- **`std`** (default): Full standard library support with file I/O and JSON
- **`yaml`**: YAML serialization/deserialization support
- **`wasm`**: WASM-compatible mode with `no_std` + `alloc`

### Usage

```toml
# Default (JSON support)
[dependencies]
walrs_rbac = "0.1.0"

# With YAML support
[dependencies]
walrs_rbac = { version = "0.1.0", features = ["yaml"] }

# For WASM targets
[dependencies]
walrs_rbac = { version = "0.1.0", default-features = false, features = ["wasm"] }
```

## WASM Support

The crate supports WebAssembly (see [WASM README](WASM_README.md) for details).

### Build

```bash
$ sh ./ci-cd-wasm.sh
```

## Prior Art

This crate is inspired by:

- **[laminas-permissions-rbac](https://github.com/laminas/laminas-permissions-rbac)** (PHP) - The primary reference implementation. Created and maintained by the [Laminas Project](https://getlaminas.org/) (formerly Zend Framework). Licensed under BSD-3-Clause.
- **[Role-Based Access Control](https://en.wikipedia.org/wiki/Role-based_access_control)** - The RBAC security model.

## License

Apache-2.0 AND GPL-3.0-only

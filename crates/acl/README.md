# walrs_acl 

Access Control List (ACL) structure for granting privileges on resources, by roles, or for all [roles or resources] in an application context.

## Usage

Instantiate your [`Acl`](examples) struct - add `Role`s, `Resource`s, and allow/deny rules - then query it from a middleware/application context.

**Inline declaration:**

```rust
fn main() -> Result<(), String> {
    // Build ACL
    let acl:Acl = AclBuilder::default()
        // Add roles with inheritance
        .add_roles(&[
            ("guest", None),
            ("user", Some(&["guest"])), // 'user' inherits rules from 'guest'
            ("editor", Some(&["user"])),  // ...
            ("admin", Some(&["editor"])), // ...
        ])?
        // Add resources
        .add_resources(&[
            ("public", None), // 'public' inherits from None 
            ("blog", None), // ...
            ("admin_panel", None), // ...
        ])?
        // Set allow rules
        .allow(Some(&["guest"]), Some(&["public"]), Some(&["read"]))?
        .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "comment"]))?
        .allow(Some(&["editor"]), Some(&["blog"]), Some(&["write", "edit"]))?
        .allow(Some(&["admin"]), None, None)? // has all privileges on all resources

        // Set deny rules
        .deny(Some(&["editor"]), Some(&["admin_panel"]), None)?

        // Build the final ACL (checks for directed cycles and outputs final `Acl` structure)
        .build()?;
    
    // In some application context...
    acl.is_allowed(admin("guest"), Some("public"), Some("read"))? // true
    // etc.
}
```

Note: if 'directed cycles' are detected the build step will result in `Err(String)`.

**From *.json representation:**

```rust
use std::fs::File;
use walrs_acl::{AclBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "./test-fixtures/example-acl-allow-and-deny-rules.json";
    let mut f = File::open(&file_path)?;
    let acl = AclBuilder::try_from(&mut f)?.build()?;
    // ...
}
```

### Construction

[The Acl] can be constructed: 

- using the `AclBuilder` structure.
- from a *.json representation, using `AclBuilder::try_from(&mut File)?.build()` (see docs for different `try_from` impls.).
- from a *.json representation using `AclBuilder::try_from(AclData)?.build()` (`AclData` can also be constructed from a *.json representation using `AclData::try_from(&mut File)`) (see docs for different `try_from` impls.).

### JSON Representation

This representation represents an [`AclData`](src/simple/acl_data.rs) struct.

```json5
{
  "roles": [             // Represents "role" symbol graph
    ["guest", null],     // `null` signals "no inheritance" 
                         // in "roles", and "resources" symbol graphs
    ["user", ["guest"]], // 'user' inherits [rules] from 'guest'
    ["special", null],
    ["admin", ["user", "special"]]
  ],
  "resources": [         // Represents "resource" symbol graph
    ["index", null],
    ["blog", ["index"]],
    ["account", null],
    ["users", null]
  ],
  "allow": [             // "allow" rules:
                         // overrides symbol graphs (roles, resources)
                         // inheritance.
    ["index", [["guest", null]]], // `null` in 'rules' structure signals "all privileges"
    ["account", [["user", ["index", "update", "read"]]]],
    ["users", [["admin", null]]]
  ],
  "deny": null           // "deny" rules (`null` at the top level fields means just null)
}
```

## Assertions

In addition to plain `Allow` / `Deny`, rules can be made conditional — fired only when a caller-supplied predicate resolves to `true`. Conditional rules are keyed by an `AssertionKey` (a plain string); the registry mapping keys to predicates lives in the caller, not in the crate. This keeps the ACL fully serializable and WASM-friendly — only the keys are persisted, never closures.

The Rust API exposes `AllowIf` / `DenyIf` via `AclBuilder::allow_if` / `deny_if`, and `Acl::is_allowed_with` / `is_allowed_any_with` which accept an `AssertionResolver`. Any `Fn(&str) -> bool` is a valid resolver via a blanket impl.

```rust
use walrs_acl::simple::AclBuilder;

let acl = AclBuilder::new()
    .add_role("editor", None)?
    .add_resource("post", None)?
    .allow_if(Some(&["editor"]), Some(&["post"]), Some(&["edit"]), "is_owner")?
    .build()?;

let is_owner = true;
let resolver = |key: &str| key == "is_owner" && is_owner;

assert!(acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver));
# Ok::<(), String>(())
```

The same rules can be expressed in JSON via the `allow_if` / `deny_if` top-level fields. The inner shape mirrors `allow` / `deny` but swaps the bare privilege name for a `[privilege, assertion_key]` pair:

```json5
{
  "roles": [["user", null], ["editor", ["user"]]],
  "resources": [["post", null], ["admin_panel", null]],
  "allow": [["post", [["editor", ["read"]]]]],
  "allow_if": [
    ["post", [["editor", [["edit", "is_owner"], ["publish", "is_owner"]]]]]
  ],
  "deny_if": [
    ["admin_panel", [["user", [["access", "outside_business_hours"]]]]]
  ]
}
```

**Conservative defaults.** Plain `is_allowed` (no resolver) treats `AllowIf` as **not-allow** and `DenyIf` as **not-deny**: without a resolver we can't evaluate the predicate, so we stay on the safe side for allows and don't synthesise a deny we aren't sure about. An explicit `Deny` still overrides a conditional `AllowIf` even when the resolver says `true`.

## How it works?

The ACL structure is made up of a `roles`, and a `resources`, symbol graph, and a "nested" `rules` structure [which is used to define, and query-for, "allow" and "deny" rules].

See tests, [benchmarks](benchmarks), and/or [examples](examples) for more details.

## WASM Support

The crate also supports WASM (WebAssembly) (see [WASM README](WASM_README.md) for more details).

### Build

```bash
$ sh ./ci-cd-wasm.sh
```

### Features

- **`std`** (default): Full standard library support with file I/O
- **`wasm`**: WASM-compatible mode with `no_std` + `alloc`

### Usage

```toml
# For WASM targets
[dependencies]
walrs_acl = { version = "0.1.0", default-features = false, features = ["wasm"] }
```

## Prior Art:

- MS Windows Registry: https://docs.microsoft.com/en-us/windows/win32/sysinfo/structure-of-the-registry#:~:text=The%20registry%20is%20a%20hierarchical,tree%20is%20called%20a%20key.&text=Value%20names%20and%20data%20can%20include%20the%20backslash%20character.
- Laminas (previously Zend Framework) Permissions/Acl: https://github.com/laminas/laminas-permissions-acl
- Registry module (Haskell): https://hackage.haskell.org/package/registry

## License

Elastic-2.0

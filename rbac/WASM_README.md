# walrs_rbac (js)

WebAssembly version of [walrs_rbac](../README.md).

## Prerequisites

Any environment that supports WebAssembly:

- Node.js v18+
- Deno v1.20+
- Chrome/Edge 57+
- Firefox 52+
- Safari 11+

## JavaScript API

### JsRbacBuilder

The builder for constructing RBACs:

```javascript
import init, { JsRbacBuilder } from './pkg/walrs_rbac.js';

await init();

let rbac;

try {
  rbac = new JsRbacBuilder()
    .addRole("guest", ["read.public"], null)
    .addRole("user", ["write.post"], ["guest"])
    .addRole("editor", ["edit.post"], ["user"])
    .addRole("admin", ["admin.panel"], ["editor"])
    .build();
} catch (error) {
  console.error("Failed to build RBAC:", error);
}
```

#### Methods

- **`new JsRbacBuilder()`** - Create a new builder
- **`fromJson(json: string): JsRbacBuilder`** - Load from JSON string
- **`addRole(name: string, permissions: string[], children?: string[]): JsRbacBuilder`** - Add a role
- **`addPermission(roleName: string, permission: string): JsRbacBuilder`** - Add a permission
- **`addChild(parentName: string, childName: string): JsRbacBuilder`** - Add a child relationship
- **`build(): JsRbac`** - Build the final RBAC

### JsRbac

The RBAC instance returned by the builder:

```javascript
// Check permissions
console.log(rbac.isGranted("admin", "read.public"));    // true (inherited)
console.log(rbac.isGranted("admin", "admin.panel"));     // true (direct)
console.log(rbac.isGranted("guest", "admin.panel"));     // false

// Check role existence
console.log(rbac.hasRole("admin"));  // true

// Get role count
console.log(rbac.roleCount());  // 4
```

#### Methods

- **`new JsRbac()`** - Create empty RBAC
- **`fromJson(json: string): JsRbac`** - Load from JSON configuration
- **`isGranted(role: string, permission: string): boolean`** - Check permission
- **`hasRole(role: string): boolean`** - Check if role exists
- **`roleCount(): number`** - Get number of roles

### Convenience Functions

```javascript
import { createRbacFromJson, checkPermission } from './pkg/walrs_rbac.js';

// Quick RBAC creation
const rbac = createRbacFromJson(jsonString);

// One-off permission check
const allowed = checkPermission(jsonString, "admin", "edit.article");
```

## JSON Configuration Format

```json
{
  "roles": [
    ["guest", ["read.public"], null],
    ["user", ["write.post", "comment.post"], ["guest"]],
    ["editor", ["edit.post", "publish.post"], ["user"]],
    ["admin", ["admin.panel", "manage.users"], ["editor"]]
  ]
}
```

**Format explanation:**
- `roles`: Array of `[name, permissions, children]` tuples
- Each role has a name, an array of permission strings, and optional child role names
- `null` in children means "no children"
- Children are role names whose permissions this role inherits

## Error Handling

Methods throw JavaScript errors on invalid configurations:

```javascript
try {
    const rbac = new JsRbacBuilder()
        .addRole("admin", ["manage"], ["nonexistent"])
        .build();
} catch (error) {
    console.error("Failed to build RBAC:", error);
}
```

## Development

### Building the WASM Module

**Prerequisites:**
```bash
cargo install wasm-pack
```

**For Node.js:**
```bash
wasm-pack build --target nodejs --no-default-features --features wasm
```

**For web (browser ESM):**
```bash
wasm-pack build --target web --no-default-features --features wasm
```

**For bundlers (webpack, rollup, etc.):**
```bash
wasm-pack build --target bundler --no-default-features --features wasm
```

### Publishing to NPM

```bash
cd pkg
npm publish
```

## Prior Art

This crate is inspired by the [laminas-permissions-rbac](https://github.com/laminas/laminas-permissions-rbac) PHP library, created by the [Laminas Project](https://getlaminas.org/) team.

## License

Same as the main crate: Apache-2.0 AND GPL-3.0-only.

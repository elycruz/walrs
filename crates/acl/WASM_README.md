# walrs_acl (js)

Webassembly version of [walrs_acl](../../README.md).

## Prerequisites

Any environment that supports WebAssembly:

- Node.js v18+
- Deno v1.20+
- Chrome/Edge 57+
- Firefox 52+
- Safari 11+

## JavaScript API

### JsAclBuilder

The builder for constructing ACLs:

```javascript
import init, {JsAclBuilder} from './walrs_acl.js';

await init();

let acl;

try {
    acl = new JsAclBuilder()
        .addRole("guest", null)
        .addRole("user", ["guest"])  // inherits from guest
        .addRole("admin", ["user"])   // inherits from user
        .addResource("blog", null)
        .addResource("admin_panel", null)
        .allow(["guest"], ["blog"], ["read"])
        .allow(["user"], ["blog"], ["read", "write"])
        .allow(["admin"], null, null)  // admin can do anything
        .deny(["user"], ["admin_panel"], null)  // user cannot access admin_panel
        .build(); // Can throw if there are 'directed cycles' in the ACL definition.
} catch (error) {
    console.error("Failed to build ACL:", error);
}
```

#### Methods

- **`new JsAclBuilder()`** - Create a new builder
- **`fromJson(json: string): JsAclBuilder`** - Load from JSON string
- **`addRole(role: string, parents?: string[]): JsAclBuilder`** - Add a role
- **`addResource(resource: string, parents?: string[]): JsAclBuilder`** - Add a resource
- **`allow(roles?: string[], resources?: string[], privileges?: string[]): JsAclBuilder`** - Add allow rule
- **`deny(roles?: string[], resources?: string[], privileges?: string[]): JsAclBuilder`** - Add deny rule
- **`build(): JsAcl`** - Build the final ACL

### JsAcl

The ACL instance returned by the `JsAclBuilder`:

```javascript
// Check permissions
console.log(acl.isAllowed("user", "blog", "write"));  // true
console.log(acl.isAllowed("user", "admin_panel", "read"));  // false
console.log(acl.isAllowed("admin", "anything", "delete"));  // true

// Check if any of the privileges are allowed
console.log(acl.isAllowedAny("user", "blog", ["write", "delete", "publish"]));  // true

// Check role/resource existence
console.log(acl.hasRole("admin"));  // true
console.log(acl.hasResource("blog"));  // true

// Check inheritance
console.log(acl.inheritsRole("admin", "user"));  // true
console.log(acl.inheritsResource("blog", "index"));  // depends on structure
```

#### Methods

- **`new JsAcl()`** - Create empty ACL (not usually needed)
- **`fromJson(json: string): JsAcl`** - Load from JSON configuration
- **`isAllowed(role?: string, resource?: string, privilege?: string): boolean`** - Check permission
- **`isAllowedAny(role?: string, resource?: string, privileges: string[]): boolean`** - Check if any privilege is allowed
- **`hasRole(role: string): boolean`** - Check if role exists
- **`hasResource(resource: string): boolean`** - Check if resource exists
- **`inheritsRole(role: string, inherits: string): boolean`** - Check role inheritance - Throws if any of the parameters don't exist in the ACL - Enforces strongly typed/architected code.
- **`inheritsResource(resource: string, inherits: string): boolean`** - Check resource inheritance - "".
- **`toJson(): string`** - Serialize to JSON (not yet implemented)

### Convenience Functions

```javascript
import { createAclFromJson, checkPermission } from './pkg/walrs_acl.js';

// Quick ACL creation
const acl = createAclFromJson(jsonString); // used `JsAclBuilder` behind the scenes

// One-off permission check
const allowed = checkPermission(jsonString, "user", "blog", "read"); // instantiates ACL in the backend
```

## JSON Configuration Format

```json
{
  "roles": [
    ["guest", null],
    ["user", ["guest"]],
    ["admin", ["user"]]
  ],
  "resources": [
    ["index", null],
    ["blog", ["index"]],
    ["admin_panel", null]
  ],
  "allow": [
    ["index", [["guest", ["read"]]]],
    ["blog", [
      ["user", ["read", "write"]],
      ["admin", null]
    ]]
  ],
  "deny": [
    ["admin_panel", [["user", null]]]
  ]
}
```

**Format explanation:**
- `roles`: Array of `[roleName, parentRoles]` tuples
- `resources`: Array of `[resourceName, parentResources]` tuples
- `allow`: Array of `[resource, [[role, privileges]]]` nested structures
- `deny`: Array of `[resource, [[role, privileges]]]` nested structures
- `null` in privileges means "all privileges"
- `null` in parents means "no parents"

## Additional Usage Examples

### Example 1: Blog Permission System

```javascript
import init, { JsAclBuilder } from './pkg/walrs_acl.js';

await init();

const blogAcl = new JsAclBuilder()
    // Add roles with inheritance using bulk method
    .addRoles([
        ["visitor", null],
        ["author", ["visitor"]],
        ["editor", ["author"]]
    ])
    // Add resources using bulk method
    .addResources([
        ["posts", null],
        ["comments", null]
    ])
    // Set permissions
    .allow(["visitor"], ["posts"], ["read"])
    .allow(["visitor"], ["comments"], ["read"])
    .allow(["author"], ["posts"], ["create", "read"])
    .allow(["author"], ["comments"], ["create", "read", "update"])
    .allow(["editor"], null, null)  // editor has all permissions
    .build();

// Check permissions
blogAcl.isAllowed("visitor", "posts", "read");     // true
blogAcl.isAllowed("visitor", "posts", "create");   // false
blogAcl.isAllowed("author", "posts", "create");    // true
blogAcl.isAllowed("editor", "posts", "delete");    // true

// Check if any visitor/author can do anything on posts
blogAcl.isAllowedAny(["visitor", "author"], ["posts"], ["create"]);  // true (author can create)
```

### Example 2: Loading from JSON (Backend Integration)

```javascript
import init, { JsAcl } from './pkg/walrs_acl.js';

await init();

// Fetch ACL config from backend
const response = await fetch('/api/acl-config');
const aclConfig = await response.json();

// Create ACL
const acl = JsAcl.fromJson(JSON.stringify(aclConfig));

// Use in application
function checkAccess(user, resource, action) {
    return acl.isAllowed(user.role, resource, action);
}

if (checkAccess(currentUser, "admin_panel", "access")) {
    // Show admin panel
}
```

### Example 3: React Integration (Frontend)

**Note:** It is not recommended to use ACLs on the frontend - unless you have a unique requirement for (the idea of an ACL is that it can only be accessed by application level code (not web console, etc.) so take care of using on the frontend).

```jsx
import { useEffect, useState } from 'react';
import init, { JsAcl } from './pkg/walrs_acl';

export function useAcl(config) {
    const [acl, setAcl] = useState(null);

    useEffect(() => {
        init().then(() => {
            const aclInstance = JsAcl.fromJson(JSON.stringify(config));
            setAcl(aclInstance);
        });
    }, [config]);

    return acl;
}

// Usage in component
function AdminPanel() {
    const acl = useAcl(aclConfig);
    const user = useCurrentUser();

    if (!acl) return <div>Loading...</div>;

    if (!acl.isAllowed(user.role, "admin_panel", "access")) {
        return <div>Access Denied</div>;
    }

    return <div>Admin Panel Content</div>;
}
```

### Error Handling

Some methods throw javascript errors in unique cases:

- `inherits_resource`, `inherits_role` throw when `inherit` param value doesn't exist in the ACL (ensures strongly architected code).
- `build` method throws when there is a "directed cycle" in the ACL:
  - When a role or resource is allowed to access itself or the first and last role/resource are the same - This is prevented by the library because directed cycles are not allowed in directed-graphs (which are used internally by the library).

```javascript
try {
    const acl = new JsAclBuilder()
        .addRole("admin", ["nonexistent"])  // This will throw
        .build();
} catch (error) {
    console.error("Failed to build ACL:", error);
}
```

## TypeScript Support

The package includes full TypeScript definitions:

```typescript
import init, { JsAclBuilder, JsAcl } from './pkg/walrs_acl';

await init();

const acl: JsAcl = new JsAclBuilder()
    .addRole("user", null)
    .addResource("api", null)
    .allow(["user"], ["api"], ["read"])
    .build();

const canRead: boolean = acl.isAllowed("user", "api", "read");
```

## Performance related

### Module Size
- **Gzipped**: **73KB** (74,571 bytes) - actual production size over HTTP (hence why it should only be used in server environment (node, deno, etc.)).
- **Uncompressed**: 165KB (168,540 bytes) after `wasm-opt -Oz` + `wasm-strip` optimization
- **Memory overhead**: ~50KB baseline RAM usage (for common ACLs (acls containing rules, roles and resources, in the low 100s) 
- **runtime overhead** Non-existent - Constructed Acls' internal data structures do not grow and support fast lookups (permission checks are benchmarked at 1.3M+ per millisecond!!) making any runtime overhead negligible (in rust tests performance overhead was benchmarked at 0.001ms per check (!!!) (see [benchmarks readme](./benchmarks/README.md) for more)).

### Performance Metrics
- **Initialization**: ~10-15ms (one-time cost, includes WASM compilation)
- **ACL Creation**: ~1ms for typical configuration (10-20 roles, 10-20 resources)
- **Single permission check** (`isAllowed`): ~0.01ms (10-50 microseconds)
- **Bulk permission check** (`isAllowedAny`): ~0.05-0.2ms (depends on array sizes)
- **JSON parsing** (`fromJson`): ~2-5ms for typical ACL configuration

### Optimization Tips
1. **Initialize once**: Call `init()` once when your application starts
2. **Cache ACL instances**: Building an ACL is fast, but caching is faster if config doesn't change
3. **Use bulk methods**: When adding multiple roles/resources, use `addRoles`/`addResources`
4. **Lazy loading**: Load ACL configuration on-demand if not immediately needed

#### Testing HTML example

Run the example HTML file:

```bash
# Start a local server (required for ES modules)
python3 -m http.server 8000
# or
npx serve .

# Open http://localhost:8000/example.html
```

Note: Use caution when using setting up an ACL instance on the frontend - ACLs 
should not be accessible from the browsers terminal, or public facing code -
as this violates the main idea of an ACL - "access control".

### Development

#### Building the WASM Module

##### Prerequisites

Install `wasm-pack`:
```bash
cargo install wasm-pack
```

##### Build Commands

**For web (browser ESM):**
```bash
wasm-pack build --target web --no-default-features --features wasm
```

**For Node.js:**
```bash
wasm-pack build --target nodejs --no-default-features --features wasm
```

**For bundlers (webpack, rollup, etc.):**
```bash
wasm-pack build --target bundler --no-default-features --features wasm
```

This generates a `pkg/` directory with:
- `walrs_acl.js` - JavaScript glue code
- `walrs_acl_bg.wasm` - Compiled WASM binary
- `walrs_acl.d.ts` - TypeScript definitions
- `package.json` - NPM package manifest

##### Build Optimization

Prerequisite: `wasm-opt`

**Default Build (Optimized):**
The default `wasm-pack build` command uses release mode with optimizations.

**Maximum Optimization (Recommended for Production):**
```bash
# 1. Build with wasm-pack (release mode is default)
wasm-pack build --target web --no-default-features --features wasm

# 2. Apply aggressive size optimization with wasm-opt
wasm-opt -Oz pkg/walrs_acl_bg.wasm -o pkg/walrs_acl_bg.wasm

# 3. (Optional) Strip custom sections with wasm-strip
# Saves an additional 244 bytes (213 bytes gzipped)
# Removes metadata like "producers" and "target_features" sections
wasm-strip pkg/walrs_acl_bg.wasm
```

**Size Results:**
- After `wasm-pack build`: ~165KB uncompressed
- After `wasm-opt -Oz`: **165KB uncompressed** (minimal additional reduction)
- After `wasm-strip`: **165KB uncompressed** (244 bytes saved - 0.14% reduction)
- **Gzipped (over HTTP): 73KB** ← This is what users download (74,571 bytes exact)

**Note on wasm-strip**: Removes non-functional metadata sections. The savings are small for release builds (~200-300 bytes) but it's still recommended as a best practice. Debug builds can save 5-20KB with wasm-strip.

**Development Build (with debug symbols):**
```bash
wasm-pack build --target web --no-default-features --features wasm --dev
```
- Faster compilation
- Larger binary (~200KB)
- Includes debug symbols

**wasm-opt Optimization Levels:**
- `-O`: Basic optimizations
- `-O2`: More optimizations  
- `-O3`: Even more optimizations
- `-Os`: Optimize for size
- `-Oz`: Aggressively optimize for size (recommended for production)

**Verify Build:**
```bash
# Check uncompressed size
ls -lh pkg/walrs_acl_bg.wasm

# Check gzipped size (what users actually download)
gzip -c pkg/walrs_acl_bg.wasm | wc -c

# Verify it's a valid WASM file
file pkg/walrs_acl_bg.wasm
```

#### Publishing to NPM

The `pkg/` directory is ready to publish:

```bash
cd pkg
npm publish
```

Then install in your project:

```bash
npm install walrs_acl
```

#### Troubleshooting

##### Common Issues

###### 1. "Cannot find module './pkg/walrs_acl.js'"
**Solution**: Make sure you've built the WASM module first:
```bash
wasm-pack build --target web --no-default-features --features wasm
```

###### 2. "MIME type mismatch" or "Failed to fetch WASM"
**Problem**: Browsers require a web server to load ES modules and WASM files.

**Solution**: Use a local development server:
```bash
# Python
python3 -m http.server 8000

# Node.js
npx serve .

# Or use your framework's dev server (Vite, webpack, etc.)
```

###### 3. "Memory access out of bounds" errors
**Cause**: Usually indicates a bug or incompatible WASM version.

**Solution**: 
- Rebuild the WASM module with the latest wasm-pack
- Clear browser cache
- Ensure you're using compatible versions of dependencies

###### 4. Large bundle size
**Problem**: WASM binary is too large for your use case.

**Solution**: Use aggressive optimization:
```bash
wasm-pack build --target web --no-default-features --features wasm --release
wasm-opt -Oz pkg/walrs_acl_bg.wasm -o pkg/walrs_acl_bg.wasm
```

###### 5. TypeScript errors with generated types
**Problem**: TypeScript complains about the generated `.d.ts` file.

**Solution**: Regenerate the types with the latest wasm-pack:
```bash
cargo install wasm-pack --force
wasm-pack build --target web --no-default-features --features wasm
```

##### Debug Setup

**Enable debug logging:**
```bash
# Build with debug symbols
wasm-pack build --target web --no-default-features --features wasm --dev
```

**Check WASM binary info:**
```bash
# Install wasm-objdump (from WABT tools)
wasm-objdump -h pkg/walrs_acl_bg.wasm

# Check exported functions
wasm-objdump -x pkg/walrs_acl_bg.wasm | grep export
```

**Verify integrity:**
```bash
# Check file size
ls -lh pkg/walrs_acl_bg.wasm

# Verify it's a valid WASM file
file pkg/walrs_acl_bg.wasm
# Should output: "WebAssembly (wasm) binary module"
```

## Getting Help

- Check the examples directory for working code.
- See `acl/pkg/example.html` for API usage examples.
- Open an issue on GitHub if you encounter bugs.

## License

Same as the main crate: Apache + GPL v3.


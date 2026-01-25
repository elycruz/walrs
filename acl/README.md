# walrs_acl 

Basic Access Control List (ACL) structure for adding role/permissions based access control to applications.

## How does it work?

The ACL control is meant to be used as a fact forest:  Each entity in the domain ([role, resource, privilege]) can be represented by a tree which can be queried upon:  E.g., 

- role tree/directional-graph.
- resource "".
- privilege "". Optional.

Essentially the component enables the possibility for resource, role, privilege and rule, relationships to be managed and queried all from one place.

## Runtime model

1.  Load the ACL tree from external source (text file, json, DB, etc.) into memory.
2.  Convert the loaded tree into fact forest structure (`*Acl` structure)j.
3.  Access the structure to check user permissions, from app middleware.

@todo example.

## Domain Model

*Definitions:*

- {entity} - One of role, resource, and/or privilege
- `type Symbol = str;` - Referential type from `*Acl` struct(ure).

*{entity} Structure:*

- `{entity}_(slug|alias): &Symbol` - Primary key, Not null.
- `{entity}_name: String` - Human Readable Name. Not null.
- `{entity}_description: Option<String>`.

## Usage

@todo

Instantiate your `Acl` struct - add `Role`s, `Resource`s, and allow/deny rules to it
Next tie the interface into your application's frontend/action controller/dispatcher, check permissions and rules and allow/deny access to resources as needed.

```rust
// @todo
```

## Todos

- [ ] `Simple` Implementation (symbol/string based implementation).
  - [x] `PrivilegeRules` - Provides access to related hash map/tree.
    - [x] `new()`
    - [x] `get_rule()`
    - [x] `set_rule()`
  - [x] `RolePrivilegeRules` - Provides access to related hash map/tree.
    - [x] `new()`
    - [x] `get_privilege_rules()`
    - [x] `set_privilege_rules()`
  - [x] `ResourceRoleRules` - ""
    - [x] `new()`
    - [x] `get_role_privilege_rules()`
    - [x] `set_role_privilege_rules()`
  - [ ] `Acl` - Forest like structure.
    - [x] `add_resource()`
    - [x] `add_role()`
    - [x] `allow()`
    - [x] `deny()`
    - [x] `has_resource()`
    - [x] `has_role()`
    - [x] `inherits_resource()`
    - [x] `inherits_resource_safe()`
    - [x] `inherits_role()`
    - [x] `inherits_role_safe()`
    - [x] `is_allowed()`
    - [x] `is_allowed_any()`
    - [x] `new()`
    - [x] `resource_count()`
    - [x] `role_count()`
    - [ ] Ensure all public API methods, with a complexity higher than 1-2, have doc tests. (`is_allowed`, `is_allowed_any` etc.).
    - [ ] Internal symbol (role, resource, or privilege) storage should be on the heap - We don't know how long passed in symbols will live so need to own the ones that are added/tracked.
    - [ ] Control should be thread safe - Add related tests.
  - [x] `AclData` - Provides a data parseable struct, when loading data from files, for `Acl` struct.
  - [x] `impl From<... File> for AclData`
    - [ ] Write companion tests against json representation, of an Acl.
  - [x] `impl From<... AclData> for Acl`
    - [ ] Write companion tests against json representation, of an Acl.

- [ ] Usage examples
  - [ ] "From DB" example (use sqlite).
  - [ ] "From JSON" example.
  - [ ] "From Text file" example.
- [ ] Documentation and Doc tests.
- [ ] Tests.
- [ ] e2e Tests, where applicable.

### Other
- [ ] Language in acl, and API, should be changed to match 'privilege' instead of 'access to privilege' since, we're effectively saying the same thing (lol).
- [ ] Cleanup API - Remove un-required methods etc., in rule structs, etc..


### About Access Control Lists (ACLs)

ACL's can be defined either in a domain model, or as one, or more, text files.

ACLs are made up of the following entities:

- Resources - Named internet resources (internal, or otherwise).
- Roles - Roles a user may take in an application.
- Group - Group of Roles.
- Role and Groups - List of groups of roles.
- Privileges - A subset, action, and/or a subset entrypoint to a resource.
- Access Control Lists (ACLs).

Additionally, ACLs can be made up of the following data structures:

1.  A roles/role-groups graph - Allows nodes to inherit from each other.
2.  A resources graph - "".
3.  An ACL.

#### Storage Mechanisms

ACLs can be stored in any storage format that is accessible by a target application:

- Relational DB.
- Text files (*.txt, *.yaml, etc.).
- etc.

##### Text Representation

Common text based formats, that can easily be used to create an ACL representation, include (but are not limited to):

- *.txt
- *.json
- *.yaml
- etc.

###### Plain Text Example:

Here we'll demonstrate storing role, resource, and role-group, graphs alongside the ACL structure, in a plain text file.

**General Structure:**

```text
# Role Graph

{role} {[..., {role}]} 

# Role Group Graph

{role-group} {[..., {role-group}]} 

# Resource Graph

{resource} {[..., {resource}]}

# ACL

{resource}
  {[deny, allow]}
    {privilege}
      {user-group}
```

**Example:**

```text
# Roles
# ----
guest
user guest
admin user

# Resources
# ----
index
blog index
products blog

# Groups
# ----
app-guest guest
app-user app-guest
app-admin app-user
cms-guest app-guest
cms-user cms-guest
cms-admin cms-user

# ACL
# ----
all
  deny
    all
index all
  allow
    index
      guest
blog index
  allow
    create
      cms-user
    read 
      guest
    update
      cms-user
    delete
      cms-admin
    disable
      cms-user
products blog
```

###### JSON Example

**example-acl.json** (WIP)

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
    ["account", null],
    ["users", null]
  ],
  "rules": {
    "allow": [
      ["index", [["guest", null]]],
      ["account", [["user", ["index", "update", "read"]]]],
      ["users", [["admin", null]]]
    ],
    "deny": null
  }
}
```

Here roles inherit from other roles, and resources, from other resources.

Where ever you see `null` those we represent as `Option<...>`, in data struct.

For `rules.allow` resources allow access to roles on privileges, if `null` means all privileges ('read', 'update', etc.).

## Brainstorm

Distill rules structure:

```rust

enum RuleType {
  Allow = 0,
  Deny = 1,
}

type PrivilegeRule = RuleType;

struct PrivilegeRules<'a> {
  fpr_all_privileges: PrivilegeRule,
  by_privilege_id: Option<HashMap<&'a str, PrivilegeRule>>,
}

struct RoleRules<'a> {
  for_all_roles: PrivilegeRules<'a>,
  by_role_id: Option<HashMap<&'a str, PrivilegeRules<'a>>>,
}

struct ResourceRules<'a> {
  for_all_resources: RoleRules<'a>,
  by_resource_id: HashMap<&'a str, RoleRules<'a>>,
}
```

## Prior Art:
- MS Windows Registry: https://docs.microsoft.com/en-us/windows/win32/sysinfo/structure-of-the-registry#:~:text=The%20registry%20is%20a%20hierarchical,tree%20is%20called%20a%20key.&text=Value%20names%20and%20data%20can%20include%20the%20backslash%20character.
- Laminas (previously Zend Framework) Permissions/Acl: https://github.com/laminas/laminas-permissions-acl
- Registry module (Haskell): https://hackage.haskell.org/package/registry

## License
Apache + GPL v3 Clause

# walrs_acl 

Access Control List (ACL) structure for granting privileges on resources, by roles or for all respectively.

## How does it work?

The ACL control is meant to be used as a fact forest:  Each entity in the domain ([role, resource, privilege]) can be represented by a tree which can be queried upon:

- role tree/directional-graph.
- resource "".
- privilege "". Optional.

Visual representation of structure: 

```rust
// ResourceRoleRules {
//     for_all_resources: RolePrivilegeRules {
//       for_all_roles: PrivilegeRules {
//         for_all_privileges: Rule
//         by_privilege_id: Option<HashMap<Privilege}, Rule>>
//       }
//       by_role_id: Option<HashMap<Role, PrivilegeRules>>
//     }
//     by_resource_id: HashMap<Resource, RolePrivilegeRules>
// }
```

Essentially, the component enables the possibility for resource, role, privilege and rule, relationships to be managed and queried all from one place.

## Runtime model

1.  Load the ACL tree from external source (text file, json, DB, etc.) into memory.
2.  Convert the loaded tree into fact forest structure (`*Acl` structure).
3.  Access the structure to check user permissions, from app middleware.

@todo example.

## Domain Model

*Definitions:*

- {entity} - One of `role`, `resource`, and/or `privilege`
- `type Symbol = str;` - Referential type used in `*Acl` structure.

*{entity} Structure:*

- `{entity}_(slug|alias): &Symbol` - Primary key, Not null.
- `{entity}_name: String` - Human Readable Name. Not null.
- `{entity}_description: Option<String>` - Nullable description.

## Usage

@todo

Instantiate your `Acl` struct - add `Role`s, `Resource`s, and allow/deny rules to it
Next tie the interface into your application's frontend/action controller/dispatcher, check permissions and rules and allow/deny access to resources as needed.

```rust
// @todo
```

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
  "allow": [
    ["index", [["guest", null]]],
    ["account", [["user", ["index", "update", "read"]]]],
    ["users", [["admin", null]]]
  ],
  "deny": null
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

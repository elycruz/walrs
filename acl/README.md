# walrs_acl 

Access Control List (ACL) structure for granting privileges on resources, by roles, or for all (roles or resources) in an application context.

## Usage

Instantiate your `Acl` struct - add `Role`s, `Resource`s, and allow/deny rules as required and then query it from a middleware/application context.

```rust
// @todo
```

## How does it work?

The ACL structure is made up of a `roles`, and a `resources`, symbol graph, and a "nested" `rules` structure [used to define the "allow" and "deny" rules on given resources, roles, and privileges, see below for more].

Internal `rules` structure: 

```rust
//  {
//     for_all_resources: RolePrivilegeRules {
//       for_all_roles: PrivilegeRules {
//         for_all_privileges: Rule
//         by_privilege_id: Option<HashMap<Privilege, Rule>>
//       }
//       by_role_id: Option<HashMap<Role, PrivilegeRules>>
//     }
//     by_resource_id: HashMap<Resource, RolePrivilegeRules>
// }
```

## Runtime model

1.  Load the ACL tree from external source (text file, json, DB, etc.) into memory.
2.  Convert the loaded tree into an acl structure.
3.  Access the structure from app middle to check user privileges.

## Domain Models

*Definitions:*

- {entity} - One of `role`, `resource`, and/or `privilege`
- `type Symbol = str;` - Referential type used in `*Acl` structure.

Example of what this [domain] model would like in a database:

*{entity} Structure:*

- `{entity}_(slug|alias): &Symbol` - Primary key, Not null.
- `{entity}_name: String` - Human Readable Name. Not null.
- `{entity}_description: Option<String>` - Nullable description.

#### Storage Mechanisms

ACLs can be stored in any storage format that is accessible by a target application:

- Relational DB.
- Text files (*.txt, *.yaml, etc.).
- etc.

##### Text Representation

Common text based formats, that can easily be used to create an ACL representation, include (but are not limited to):

- *.json
- *.yaml
- etc.

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

## RDBMS Example Relations

*Definitions:*

- {entity} - One of `role`, `resource`, and/or `privilege`
- `type Symbol = str;` - Referential type used in `*Acl` structure.

Example of what this [domain] model would like in a database:

*{entity} Structure:*

- `{entity}_(slug|alias): &Symbol` - Primary key, Not null.
- `{entity}_name: String` - Human Readable Name. Not null.
- `{entity}_description: Option<String>` - Nullable description.


## Prior Art:

- MS Windows Registry: https://docs.microsoft.com/en-us/windows/win32/sysinfo/structure-of-the-registry#:~:text=The%20registry%20is%20a%20hierarchical,tree%20is%20called%20a%20key.&text=Value%20names%20and%20data%20can%20include%20the%20backslash%20character.
- Laminas (previously Zend Framework) Permissions/Acl: https://github.com/laminas/laminas-permissions-acl
- Registry module (Haskell): https://hackage.haskell.org/package/registry

## License
Apache + GPL v3 Clause

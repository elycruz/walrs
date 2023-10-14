# walrs_acl 

Basic Access Control List (ACL) structure for adding role/permissions based access control to applications.

## How does it work?

The ACL control is designed to manage relationships between resources, roles, privileges and rules and their interrelationships to each of their set (roles inheriting from other roles etc.).

@todo

## Usage

@todo

Instantiate your `Acl` struct - add `Role`s, `Resource`s, and allow/deny rules to it
Next tie the interface into your application's frontend/action controller/dispatcher, check permissions and rules and allow/deny access to resources as needed.

```rust
// @todo
```

## Todos

- [ ] `simple`
  - [x] `PrivilegeRules`
    - [x] `new()`
    - [x] `get_rule()`
    - [x] `set_rule()`
  - [x] `RolePrivilegeRules`
    - [x] `new()`
    - [x] `get_privilege_rules()`
    - [x] `set_privilege_rules()`
  - [x] `ResourceRoleRules`
    - [x] `new()`
    - [x] `get_role_privilege_rules()`
    - [x] `set_role_privilege_rules()`
  - [ ] `Acl`
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
  - [ ] Acl related:
    - [ ] Ensure all public API methods, with a complexity higher than 1-2, have doc tests. (`is_allowed`, `is_allowed_any` etc.).   
    - [ ] Internal symbol (role, resource, or privilege) storage should be on the heap - We don't know how long passed in symbols will live so need to own the ones that are added/tracked.
  - [x] `AclData` - Provides a data parseable struct, when loading data from files, for `Acl` struct. 
  - [x] `impl From<... File> for AclData`
    - [ ] Write companion tests against json representation, of an Acl. 
  - [x] `impl From<... AclData> for Acl`
    - [ ] Write companion tests against json representation, of an Acl. 
  - [ ] Control should function be thread safe - Add test showing this case.
  - [ ] Language in acl, and API, should be changed to match 'privilege' instead of 'access to privilege' since, we're effectively saying the same thing (lol).
  - [ ] Cleanup API - Remove un-required methods etc., in rule structs, etc..

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

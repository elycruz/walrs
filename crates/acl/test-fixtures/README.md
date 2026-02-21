## Test Fixtures

### example-acl.json

Data struct:

```rust
struct AclData<'a> {
    pub roles: Vec<(&'a str, Option<&[&'a str]>)>,
    pub resources: Vec<(&'a str, Option<&[&'a str]>)>,
    pub allow: Vec<(&'a str, Option<&[&'a str]>)>,
    pub deny: Vec<(&'a str, Option<Vec<(&'a str, Option<Vec<&'a str>>)>>)>,
}
```

Example *.json:

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

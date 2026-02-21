## graph package (work-in-progress)

### Reference Implementations:

- Algorithms 4th Ed. - Chapter on Graphs: https://algs4.cs.princeton.edu/40graphs/
- https://github.com/TheAlgorithms/Rust/tree/master

### Todos:

- [x] `Graph`
  - [x] `new()`
  - [x] `vert_count()`
  - [x] `edge_count()`
  - [x] `adj()`
  - [x] `degree()`
  - [x] `add_vertex()`
  - [x] `has_vertex()`
  - [x] `remove_vertex()`
  - [x] `add_edge()`
  - [x] `has_edge()`
  - [x] `remove_edge()`
  - [x] `validate_vertex()`
  - [x] `digest_lines()` - Should just be `try_from` for `BufReader<R>`, etc.

- [ ] `DepthFirstSearch`
  - [ ] @todo
  
- [ ] `SymbolGraph` (consider making this generic).
  - [ ] `new()`
  - [ ] `vert_count()`
  - [ ] `edge_count()`
  - [ ] `adj()`
  - [ ] `graph()`
  - [ ] `degree()`
  - [ ] `contains()` - Same as `has_vertex()`.
  - [ ] `index()`
  - [ ] `indices()`
  - [ ] `name()`
  - [ ] `names()`
  - [ ] `add_vertex()`
  - [ ] `has_vertex()`
  - [ ] `add_edge()`
  - [ ] `validate_vertex()` - Symgraph here should probably contain a BTree
  here, for enabling fast lookups
  - [ ] `remove_vertex()`
  - [ ] `impl<R: std::io::Read> From<&mut BufReader<R>>`
  - [ ] @todo 

### Example "Symbol" Graph

Example graph data for graph that uses strings as it's vertices:

#### Plain text example

Format: `symbol [,symbol]` - First `symbol` inherits from adjacent symbols (Backward Directed Graph representation) etc.

`roles.txt`

```text
guest
user guest
tester user
developer tester
editor developer
publisher editor
cms-guest guest
cms-tester tester cms-guest
cms-developer cms-tester
cms-editor editor cms-developer
cms-publisher cms-editor 
cms-super-admin cms-publisher
```

#### *.json examples

`roles.json`

```json
[ ["guest"],
  ["user", ["guest"]],
  ["tester", ["user"]],
  ["developer", ["tester"]],
  ["editor", ["developer"]],
  ["publisher", ["editor"]],
  ["cms-guest", ["guest"]],
  ["cms-tester", ["tester", "cms-guest"]]
]
```

`acl.json`

Here Role, Resource, and rule, relationships are defined in a single *.json file.

```json
{
  "roles": {
    "guest": null,
    "user": ["guest"]
  },
  "resources": {
    "index": null,
    "blog": ["index"],
    "account": null,
    "users": null
  },
  "rules": {
    "allow": [
      ["index", ["guest"]],
      ["account", ["user"]],
      ["users", ["admin"]]
    ],
    "deny": null
  }
}
```

Alternatively:

```json
{
  "roles": [
    ["guest"],
    ["user", ["guest"]]
  ]
}
```

And for better "cost-effective" "associative lists" approach (use flat lists):

```json
{
  "role": [
    ["guest"],
    ["user", "guest"]
  ]
}
```

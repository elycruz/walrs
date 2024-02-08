# Design

## Node based approach

General approach: 

```pseudo
- nav-item
  - items
    - nav-item
      - items
    - nav-item
    - nav-item
```

Questions:

- How do we traverse the tree in this approach? Using DFS.

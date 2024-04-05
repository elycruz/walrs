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

Component should:
- not allow cycles (e.g., be acyclic).

Questions:

- How do we traverse the tree in this approach? Using DFS.

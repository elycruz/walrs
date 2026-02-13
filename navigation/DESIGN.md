# Design

## Node-based Approach

The navigation component uses a tree-based structure where each node represents a navigation item (page).

```
- NavItem (root)
  - items: Vec<NavItem>
    - NavItem (child 1)
      - items: Vec<NavItem>
        - NavItem (grandchild 1)
        - NavItem (grandchild 2)
    - NavItem (child 2)
    - NavItem (child 3)
```

## Key Architectural Decisions

### Acyclic Tree Structure
- The component enforces an acyclic tree structure (no cycles allowed)
- Parent-child relationships are maintained through the `items` vector
- Each NavItem can have multiple children but no back-references to parents (to avoid cycles and reference issues)

### Traversal Strategy
- Uses Depth-First Search (DFS) for tree traversal
- Implemented in methods like `find`, `find_all`, and `remove`
- Recursive internal helper methods maintain proper lifetime management

### Core Components

#### NavItem
- Represents individual navigation pages/items
- Contains properties for web rendering (URI, label, fragment, CSS class, etc.)
- Supports ACL integration (resource, privilege properties)
- Includes ordering support for sorting siblings
- Builder pattern support via derive_builder

#### Container
- Root-level manager for navigation trees
- Provides operations on multiple top-level navigation items
- Implements search, sort, and manipulation methods

### Internal State Management
- Uses internal flags for lazy evaluation:
  - `_reevaluate_size`: Marks when size needs recalculation
  - `_reevaluate_order`: Marks when ordering needs to be applied
  - `_stored_size`: Caches calculated size
- This optimization avoids recalculating tree size on every access

### Lifetime Management
- Internal helper methods (`find_internal`, `find_all_internal`, `remove_internal`) handle borrowing correctly
- Explicit lifetime annotations used where needed to maintain proper reference lifetimes during recursive operations

## Implementation Details

### Search Operations
- `find`: Returns first match using provided predicate
- `find_all`: Returns all matches in tree
- `find_by_id`: Convenience method for ID-based lookup

### Modification Operations
- `add`: Adds child to navigation item
- `remove`: Recursively searches and removes matching item
- `sort`: Sorts items by order property

### Helper Methods
- `has_children`: Checks if item has any children
- `is_active`/`set_active`: Manages active state
- `get_href`: Builds complete URI with fragment
- `sort_children`: Recursively sorts child items

## Future Considerations

- Iterator implementation for tree traversal
- Serialization/deserialization support
- View helpers for rendering (menus, breadcrumbs, sitemaps)
- Tighter ACL integration with walrs_acl module
- Performance optimizations for large navigation trees

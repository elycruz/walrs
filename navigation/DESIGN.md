# Design

## Node based approach

General approach: 

```pseudo
- Container
  - Page (nav-item)
    - pages
      - Page (nav-item)
        - pages
      - Page (nav-item)
      - Page (nav-item)
```

Component should:
- not allow cycles (e.g., be acyclic) - enforced via owned `Vec<Page>` children.
- support depth-first traversal (both standard and depth-aware).
- support breadcrumb trail generation by finding the path to the active page.

## Architecture

- **Container**: Root collection of pages. Provides search, traversal, breadcrumbs, and rendering.
- **Page**: A node in the tree. Has properties (label, uri, title, etc.) and optional children.
- **PageBuilder**: Fluent builder for constructing `Page` instances.
- **NavigationError**: Error enum with `Result` type alias. No panics.
- **view**: HTML rendering helpers (menus, breadcrumbs, sitemaps) with XSS protection.

## Feature Flags

- `json` (default): JSON serialization via `serde_json`.
- `yaml`: YAML serialization via `serde_yaml`.

## Traversal

Tree traversal uses recursive DFS:
- `traverse()`: visits every page.
- `traverse_with_depth()`: visits every page with depth information.
- `breadcrumbs()`: finds the path from root to the active page.

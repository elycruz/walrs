# Navigation

A Rust implementation of a navigation component inspired by the Laminas Navigation component.

## Overview

`walrs_navigation` is a library for managing trees of pointers to web pages. It can be used for creating:
- Menus
- Breadcrumbs
- Links
- Sitemaps
- Or serve as a model for other navigation-related purposes

## Features

- **Tree Structure**: Hierarchical navigation with parent-child relationships
- **Search & Filter**: Find navigation items by ID, label, or custom predicates
- **Ordering**: Sort navigation items by order property
- **ACL Integration**: Built-in support for access control (resource/privilege)
- **Flexible Properties**: Support for URIs, fragments, labels, CSS classes, HTML attributes
- **Builder Pattern**: Easy construction of navigation items using derive_builder

## Usage

```rust
use walrs_navigation::navigation::{NavItem, Container, NavigationItem};

// Create a navigation container
let mut nav = Container::new();

// Add root-level items
nav.add(NavItem {
    id: Some("home".to_string()),
    label: Some("Home".to_string()),
    uri: Some("/".to_string()),
    order: Some(1),
    ..Default::default()
});

// Create nested navigation
let mut about = NavItem {
    id: Some("about".to_string()),
    label: Some("About".to_string()),
    uri: Some("/about".to_string()),
    order: Some(2),
    ..Default::default()
};

about.add(NavItem {
    id: Some("team".to_string()),
    label: Some("Our Team".to_string()),
    uri: Some("/about/team".to_string()),
    ..Default::default()
});

nav.add(about);

// Find items
let home = nav.find_by_id("home");
let all_items = nav.find_all(|item| item.visible);

// Sort navigation
nav.sort();
```

## License

MIT 3.0 + Apache 2.0

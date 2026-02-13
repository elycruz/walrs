# walrs_navigation

A Rust implementation of a navigation component for managing trees of pointers to web pages. This component can be used for creating menus, breadcrumbs, links, and sitemaps, or serve as a model for other navigation related purposes.

This crate is inspired by and follows the design principles of the [Laminas Navigation](https://github.com/laminas/laminas-navigation) component from the Laminas PHP framework.

## Features

- **Hierarchical Navigation Trees**: Create and manage trees of navigation pages with unlimited nesting
- **Flexible Page Properties**: Support for URIs, labels, titles, fragments, routes, ACL settings, and custom attributes
- **JSON/YAML Support**: Deserialize navigation structures from JSON or YAML files
- **Builder Pattern**: Fluent API for constructing navigation structures
- **Type Safety**: Full type safety with `Result`-based error handling (no panics)
- **Iterator Support**: Standard Rust iterator patterns for traversal
- **Comprehensive Testing**: Thoroughly tested with unit tests and doc tests
- **Performance**: Benchmarked and optimized for production use
- **Actix Web Integration**: Example showing integration with Actix Web framework

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
walrs_navigation = { path = "../navigation" }
```

## Quick Start

### Basic Usage

```rust
use walrs_navigation::{Container, Page};

fn main() {
    // Create a navigation container
    let mut nav = Container::new();

    // Add pages using the builder pattern
    nav.add_page(
        Page::builder()
            .label("Home")
            .uri("/")
            .build()
    );

    // Create nested navigation
    let mut products = Page::builder()
        .label("Products")
        .uri("/products")
        .build();

    products.add_page(
        Page::builder()
            .label("Books")
            .uri("/products/books")
            .build()
    );

    nav.add_page(products);

    // Find pages
    if let Some(page) = nav.find_by_uri("/products/books") {
        println!("Found: {}", page.label().unwrap_or(""));
    }

    // Traverse all pages
    nav.traverse(&mut |page| {
        println!("{}", page.label().unwrap_or("(no label)"));
    });
}
```

### Loading from JSON

```rust
use walrs_navigation::Container;

let json = r#"[
    {
        "label": "Home",
        "uri": "/"
    },
    {
        "label": "About",
        "uri": "/about",
        "pages": [
            {
                "label": "Team",
                "uri": "/about/team"
            }
        ]
    }
]"#;

let nav = Container::from_json(json)?;
println!("Loaded {} pages", nav.count());
```

### Loading from YAML

```rust
use walrs_navigation::Container;

let yaml = r#"
- label: Home
  uri: /
  order: 1
- label: About
  uri: /about
  order: 2
  pages:
    - label: Team
      uri: /about/team
"#;

let nav = Container::from_yaml(yaml)?;
```

## Page Properties

Each `Page` supports the following properties:

- **label**: Display text for the page
- **uri**: URI/URL for the page
- **title**: Page title (for HTML `<title>` tags, etc.)
- **fragment**: Fragment identifier (e.g., "#section")
- **route**: Route name for routing systems
- **resource**: ACL resource identifier
- **privilege**: ACL privilege identifier
- **active**: Whether the page is currently active
- **visible**: Whether the page should be displayed
- **class**: CSS class name
- **id**: HTML ID attribute
- **target**: Link target (e.g., "_blank")
- **attributes**: Custom key-value attributes
- **pages**: Child pages (for hierarchical navigation)
- **order**: Display order (lower values appear first)

## API Reference

### Container

The `Container` type manages a collection of root-level pages.

```rust
let mut nav = Container::new();

// Add pages
nav.add_page(page);

// Remove pages
let page = nav.remove_page(0)?;

// Find pages
let page = nav.find_by_uri("/about");
let page = nav.find_by_label("Home");
let page = nav.find_by_id("main-nav");

// Get page count
let count = nav.count();

// Check if empty
if nav.is_empty() {
    println!("No pages");
}

// Clear all pages
nav.clear();

// Traverse all pages
nav.traverse(&mut |page| {
    // Do something with each page
});

// Iterate over root pages
for page in nav.iter() {
    println!("{}", page.label().unwrap_or(""));
}

// Set active page
nav.set_active_by_uri("/current-page");

// Serialize to JSON/YAML
let json = nav.to_json()?;
let yaml = nav.to_yaml()?;
```

### Page

The `Page` type represents a single navigation item.

```rust
// Create with builder
let page = Page::builder()
    .label("Products")
    .uri("/products")
    .title("Our Products")
    .order(2)
    .visible(true)
    .active(false)
    .class("nav-item")
    .attribute("data-section", "main")
    .build();

// Add child pages
let mut parent = Page::builder().label("Products").build();
parent.add_page(
    Page::builder()
        .label("Books")
        .build()
);

// Remove child pages
let removed = parent.remove_page(0)?;

// Find child pages
let child = parent.find_page(|p| p.label() == Some("Books"));

// Get page count (including descendants)
let count = parent.count();

// Check if page has children
if parent.has_pages() {
    println!("Has child pages");
}

// Traverse page tree
parent.traverse(&mut |page| {
    // Visit each page in the tree
});
```

## Examples

The crate includes two comprehensive examples:

### Basic Navigation Example

Run the basic example to see core functionality:

```bash
cargo run --example basic_navigation
```

This example demonstrates:
- Creating navigation structures
- Adding and organizing pages
- Finding pages by URI and label
- Traversing the navigation tree
- JSON and YAML serialization/deserialization
- Setting active pages

### Actix Web Integration Example

Run the web server example:

```bash
cargo run --example actix_web_navigation
```

Then visit http://127.0.0.1:8080 in your browser.

This example demonstrates:
- Integration with Actix Web framework
- Rendering HTML navigation menus
- Active page highlighting
- Dropdown sub-menus
- JSON API endpoint for navigation data

## Performance

The navigation component is designed for performance:

- **O(1)** page addition and removal
- **O(n)** search operations (where n is the number of pages)
- **O(n)** traversal operations
- Automatic sorting by order value
- Minimal memory overhead

Run the benchmarks:

```bash
cargo bench --package walrs_navigation
```

## Testing

The crate includes comprehensive test coverage:

```bash
# Run unit tests
cargo test --package walrs_navigation

# Run with output
cargo test --package walrs_navigation -- --nocapture

# Run doc tests
cargo test --package walrs_navigation --doc
```

## Design Principles

This crate follows idiomatic Rust best practices:

- **No panics**: All operations return `Result` types for proper error handling
- **Type safety**: Strong typing prevents common errors at compile time
- **Zero-cost abstractions**: No runtime overhead for safety features
- **Composable**: Easy to integrate with web frameworks and other libraries
- **Well documented**: Comprehensive documentation with examples
- **Tested**: Extensive unit tests and integration tests

## Credits

This crate is inspired by the [Laminas Navigation](https://github.com/laminas/laminas-navigation) component, created by the Laminas Project and contributors. We are grateful for their excellent work in designing a flexible and powerful navigation abstraction.

The original Laminas Navigation component:
- GitHub: https://github.com/laminas/laminas-navigation
- Documentation: https://docs.laminas.dev/laminas-navigation/
- License: BSD-3-Clause

## License

This project is licensed under the same license as the walrs project. See the LICENSE file in the repository root for details.

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test --package walrs_navigation`
2. Code is formatted: `cargo fmt --package walrs_navigation`
3. Clippy is happy: `cargo clippy --package walrs_navigation`
4. Documentation is updated
5. New features include tests and examples

## Version History

### 0.1.0 (Initial Release)

- Core navigation types (Page, Container)
- Builder pattern for page creation
- JSON and YAML serialization/deserialization
- Search and traversal operations
- Comprehensive test coverage
- Performance benchmarks
- Example integrations (basic and Actix Web)

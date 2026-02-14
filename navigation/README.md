# walrs_navigation

A Rust implementation of a navigation component for managing trees of pointers to web pages. This component can be used for creating menus, breadcrumbs, links, and sitemaps, or serve as a model for other navigation related purposes.

This crate is inspired by and follows the design principles of the [Laminas Navigation](https://github.com/laminas/laminas-navigation) component from the Laminas PHP framework.

## Features

- **Hierarchical Navigation Trees**: Create and manage trees of navigation pages with unlimited nesting
- **Flexible Page Properties**: Support for URIs, labels, titles, fragments, routes, ACL settings, and custom attributes
- **JSON/YAML Support**: Deserialize navigation structures from JSON or YAML (feature-gated)
- **Builder Pattern**: Fluent API for constructing navigation structures
- **Public Fields**: Direct access to page properties for reading and writing
- **Type Safety**: Full type safety with `Result`-based error handling (no panics)
- **Iterator Support**: Standard Rust iterator patterns for traversal
- **View Helpers**: Built-in rendering for menus, breadcrumbs, and sitemaps with XSS protection
- **Breadcrumb Trails**: Automatic breadcrumb generation from the active page path
- **Comprehensive Testing**: Thoroughly tested with unit tests and doc tests
- **Performance**: Benchmarked and optimized for production use
- **Actix Web Integration**: Example showing integration with Actix Web framework

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
walrs_navigation = "0.1.0"
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `json`  | ✅ Yes  | JSON serialization/deserialization support |
| `yaml`  | ❌ No   | YAML serialization/deserialization support |

To enable YAML support:

```toml
[dependencies]
walrs_navigation = { version = "0.1.0", features = ["yaml"] }
```

To enable both JSON and YAML:

```toml
[dependencies]
walrs_navigation = { version = "0.1.0", features = ["json", "yaml"] }
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
        println!("Found: {}", page.label.as_deref().unwrap_or(""));
    }

    // Traverse all pages with depth
    nav.traverse_with_depth(&mut |page, depth| {
        let indent = "  ".repeat(depth);
        println!("{}{}", indent, page.label.as_deref().unwrap_or("(no label)"));
    });
}
```

### Loading from JSON

```rust
use walrs_navigation::Container;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
```

### Loading from YAML

Requires the `yaml` feature flag.

```rust
use walrs_navigation::Container;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
```

## Page Properties

Each `Page` supports the following properties:

| Property | Type | Description |
|----------|------|-------------|
| `label` | `Option<String>` | Display text for the page |
| `uri` | `Option<String>` | URI/URL for the page |
| `title` | `Option<String>` | Page title (for HTML `<title>` tags) |
| `fragment` | `Option<String>` | Fragment identifier (e.g., "section") |
| `route` | `Option<String>` | Route name for routing systems |
| `resource` | `Option<String>` | ACL resource identifier |
| `privilege` | `Option<String>` | ACL privilege identifier |
| `active` | `bool` | Whether the page is currently active |
| `visible` | `bool` | Whether the page should be displayed (default: `true`) |
| `class` | `Option<String>` | CSS class name |
| `id` | `Option<String>` | HTML ID attribute |
| `target` | `Option<String>` | Link target (e.g., "_blank") |
| `attributes` | `HashMap<String, String>` | Custom key-value attributes |
| `pages` | `Vec<Page>` | Child pages (for hierarchical navigation) |
| `order` | `i32` | Display order (lower values appear first) |

All properties are public fields and can be accessed directly. The builder pattern provides a convenient way to construct pages.

## API Reference

### Container

The `Container` type manages a collection of root-level pages.

```rust
use walrs_navigation::{Container, Page};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut nav = Container::new();

    // Add pages using fluent interface
    nav.add_page(Page::builder().label("Home").uri("/").build())
       .add_page(Page::builder().label("About").uri("/about").build());

    // Add multiple pages at once
    nav.add_pages(vec![
        Page::builder().label("Products").build(),
        Page::builder().label("Contact").build(),
    ]);

    // Replace all pages
    nav.set_pages(vec![Page::builder().label("Home").build()]);

    // Remove a page by index
    let page = nav.remove_page(0)?;

    // Find pages by various criteria
    let page = nav.find_by_uri("/about");
    let page = nav.find_by_label("Home");
    let page = nav.find_by_id("main-nav");
    let page = nav.find_by_route("home");

    // Find pages with custom predicates
    let page = nav.find_page(|p| p.class.as_deref() == Some("nav-primary"));

    // Check if a page exists
    let exists = nav.has_page(|p| p.uri.as_deref() == Some("/about"), true);

    // Get page count
    let count = nav.count();

    // Check if empty
    if nav.is_empty() {
        println!("No pages");
    }

    // Only visible pages
    let visible = nav.visible_pages();

    // Clear all pages
    nav.clear();

    // Traverse all pages
    nav.traverse(&mut |page| {
        // Do something with each page
    });

    // Traverse with depth information
    nav.traverse_with_depth(&mut |page, depth| {
        let indent = "  ".repeat(depth);
        println!("{}{}", indent, page.label.as_deref().unwrap_or(""));
    });

    // Iterate over root pages
    for page in nav.iter() {
        println!("{}", page.label.as_deref().unwrap_or(""));
    }

    // Set active page
    nav.set_active_by_uri("/current-page");

    // Get breadcrumb trail
    let crumbs = nav.breadcrumbs();
    for crumb in &crumbs {
        println!("{}", crumb.label.as_deref().unwrap_or(""));
    }

    // Serialize to JSON/YAML
    let json = nav.to_json()?;
    let yaml = nav.to_yaml()?;

    Ok(())
}
```

### Page

The `Page` type represents a single navigation item.

```rust
use walrs_navigation::Page;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create with builder
    let page = Page::builder()
        .label("Products")
        .uri("/products")
        .title("Our Products")
        .fragment("featured")
        .route("products")
        .resource("mvc:products")
        .privilege("view")
        .order(2)
        .visible(true)
        .active(false)
        .class("nav-item")
        .id("products-link")
        .target("_self")
        .attribute("data-section", "main")
        .build();

    // Get the full href (URI + fragment)
    assert_eq!(page.href(), Some("/products#featured".to_string()));

    // Dynamic property access
    let mut page = Page::new();
    page.label = Some("Dynamic".to_string());
    page.uri = Some("/dynamic".to_string());
    assert_eq!(page.label.as_deref(), Some("Dynamic"));

    // Direct field access for all properties
    let mut page = Page::new();
    page.label = Some("Home".to_string());
    page.uri = Some("/".to_string());
    page.order = 1;

    // Custom attributes with fluent interface
    page.set_attribute("data-id", "home")
        .set_attribute("data-section", "main");
    
    // Get attributes
    let id = page.get_attribute("data-id");
    let attrs = page.get_attributes(&["data-id", "data-section"]);
    
    // Remove attributes
    page.remove_attribute("data-id");
    page.remove_attributes(&["data-section"]);
    page.clear_attributes();

    // Add child pages using fluent interface
    let mut parent = Page::builder().label("Products").build();
    parent.add_page(Page::builder().label("Books").build())
          .add_page(Page::builder().label("Electronics").build())
          .add_page(Page::builder().label("Clothing").build());

    // Remove a child page by index
    let removed = parent.remove_page(0)?;
    
    // Clear all children
    parent.clear_pages();
    
    // Replace all children
    parent.set_pages(vec![
        Page::builder().label("New Child").build(),
    ]);

    // Find child pages
    let child = parent.find_page(|p| p.label.as_deref() == Some("Books"));
    let all_visible = parent.find_all_pages(|p| p.visible);

    // Check for child pages
    let has_books = parent.has_page(|p| p.label.as_deref() == Some("Books"), true);
    let is_branch_active = parent.is_active_branch();
    let visible_children = parent.visible_pages();

    // Get page count (including descendants)
    let count = parent.count();

    // Traverse page tree with depth
    parent.traverse_with_depth(0, &mut |page, depth| {
        let indent = "  ".repeat(depth);
        println!("{}{}", indent, page.label.as_deref().unwrap_or(""));
    });

    Ok(())
}
```

### View Helpers

The `view` module provides functions for rendering navigation as HTML.

```rust
use walrs_navigation::{Container, Page, view};

fn main() {
    let mut nav = Container::new();
    nav.add_page(Page::builder().label("Home").uri("/").active(true).build());
    let mut products = Page::builder().label("Products").uri("/products").build();
    products.add_page(Page::builder().label("Books").uri("/products/books").build());
    nav.add_page(products);

    // Render as HTML menu
    let menu = view::render_menu(&nav);

    // Render menu with custom CSS classes
    let menu = view::render_menu_with_class(&nav, "navbar-nav", "current");

    // Render breadcrumbs
    let crumbs = view::render_breadcrumbs(&nav, " &gt; ");

    // Render flat sitemap
    let sitemap = view::render_sitemap(&nav);

    // Render hierarchical sitemap
    let sitemap = view::render_sitemap_hierarchical(&nav);

    // HTML escape utility
    let safe = view::html_escape("<script>alert('xss')</script>");
}
```

All view helper functions automatically:
- Escape HTML special characters to prevent XSS attacks
- Skip invisible pages
- Mark active pages with appropriate CSS classes
- Support nested/hierarchical navigation structures

## Examples

The crate includes two comprehensive examples:

### Basic Navigation Example

Run the basic example to see core functionality:

```bash
cargo run --example basic_navigation --features json,yaml
```

This example demonstrates:
- Creating navigation structures
- Adding and organizing pages
- Finding pages by URI, label, route, and custom predicates
- Depth-aware traversal
- Breadcrumb generation
- View helper rendering (menus, breadcrumbs, sitemaps)
- JSON and YAML serialization/deserialization
- Direct field access
- Fragment-based hrefs

### Actix Web Integration Example

Run the web server example:

```bash
cargo run --example actix_web_navigation --features json,yaml
```

Then visit http://127.0.0.1:8080 in your browser.

This example demonstrates:
- Integration with Actix Web framework
- Rendering HTML navigation menus using view helpers
- Active page highlighting
- Breadcrumb trail rendering
- Dropdown sub-menus
- JSON API endpoint for navigation data

## Performance

The navigation component is designed for performance:

- **O(1)** page addition (amortized)
- **O(n)** search operations (where n is the number of pages)
- **O(n)** traversal and rendering operations
- Automatic sorting by order value
- Minimal memory overhead

Run the benchmarks:

```bash
cargo bench --package walrs_navigation --features json,yaml
```

Benchmarks cover:
- Page creation (builder and direct)
- Container operations (add, bulk add)
- Find operations (by URI, label, predicate)
- Nested operations at various depths
- Traversal (standard and depth-aware)
- Breadcrumb generation
- Serialization (JSON/YAML round-trips)
- View helper rendering (menu, breadcrumbs, sitemap)

## Testing

The crate includes comprehensive test coverage:

```bash
# Run all tests
cargo test --package walrs_navigation --features json,yaml

# Run with output
cargo test --package walrs_navigation --features json,yaml -- --nocapture

# Run doc tests only
cargo test --package walrs_navigation --features json,yaml --doc

# Run without YAML feature (JSON only)
cargo test --package walrs_navigation
```

## Design Principles

This crate follows idiomatic Rust best practices:

- **No panics**: All operations return `Result` types for proper error handling
- **Type safety**: Strong typing prevents common errors at compile time
- **Zero-cost abstractions**: No runtime overhead for safety features
- **Composable**: Easy to integrate with web frameworks and other libraries
- **XSS protection**: All HTML rendering escapes special characters
- **Well documented**: Comprehensive documentation with examples
- **Tested**: Extensive unit tests, doc tests, and benchmarks
- **Feature-gated**: JSON and YAML support behind feature flags

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

1. All tests pass: `cargo test --package walrs_navigation --features json,yaml`
2. Code is formatted: `cargo fmt --package walrs_navigation`
3. Clippy is happy: `cargo clippy --package walrs_navigation --features json,yaml`
4. Documentation is updated
5. New features include tests and examples

## Version History

### 0.1.0 (Initial Release)

- Core navigation types (Page, Container)
- Builder pattern for page creation
- JSON and YAML serialization/deserialization (feature-gated)
- Search and traversal operations
- Breadcrumb trail generation
- View helpers (menu, breadcrumb, sitemap rendering)
- XSS-safe HTML rendering
- Comprehensive test coverage
- Performance benchmarks
- Example integrations (basic and Actix Web)

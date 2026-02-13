//! # walrs_navigation
//!
//! A Rust implementation of a navigation component for managing trees of pointers to web pages.
//! This component can be used for creating menus, breadcrumbs, links, and sitemaps, or serve as
//! a model for other navigation related purposes.
//!
//! This crate is inspired by the [Laminas Navigation](https://github.com/laminas/laminas-navigation)
//! component from the Laminas PHP framework.
//!
//! ## Features
//!
//! - **Hierarchical Navigation Trees**: Create and manage trees of navigation pages
//! - **Flexible Page Properties**: Support for URIs, labels, titles, ACL settings, and more
//! - **JSON/YAML Support**: Deserialize navigation structures from JSON or YAML
//! - **Builder Pattern**: Fluent API for constructing navigation structures
//! - **Type Safety**: Full type safety with Result-based error handling
//! - **Iterator Support**: Standard Rust iterator patterns for traversal
//!
//! ## Quick Start
//!
//! ```
//! use walrs_navigation::{Container, Page};
//!
//! // Create a navigation container
//! let mut nav = Container::new();
//!
//! // Add pages using the builder pattern
//! nav.add_page(
//!     Page::builder()
//!         .label("Home")
//!         .uri("/")
//!         .build()
//! );
//!
//! // Create nested navigation
//! let mut products = Page::builder()
//!     .label("Products")
//!     .uri("/products")
//!     .build();
//!
//! products.add_page(
//!     Page::builder()
//!         .label("Books")
//!         .uri("/products/books")
//!         .build()
//! );
//!
//! nav.add_page(products);
//!
//! // Find pages
//! if let Some(page) = nav.find_by_uri("/products/books") {
//!     println!("Found: {}", page.label().unwrap_or(""));
//! }
//! ```
//!
//! ## JSON/YAML Support
//!
//! ```
//! use walrs_navigation::Container;
//!
//! let json = r#"[
//!     {
//!         "label": "Home",
//!         "uri": "/"
//!     },
//!     {
//!         "label": "About",
//!         "uri": "/about",
//!         "pages": [
//!             {
//!                 "label": "Team",
//!                 "uri": "/about/team"
//!             }
//!         ]
//!     }
//! ]"#;
//!
//! let nav = Container::from_json(json).unwrap();
//! assert_eq!(nav.count(), 3);
//! ```
//!
//! ## Credits
//!
//! This crate is inspired by and follows the design principles of the
//! [Laminas Navigation](https://github.com/laminas/laminas-navigation) component,
//! created by the Laminas Project and contributors. We are grateful for their excellent
//! work in designing a flexible and powerful navigation abstraction.

pub mod container;
pub mod error;
pub mod page;
mod serde_impls;
pub mod view;

// Re-export main types
pub use container::Container;
pub use error::{NavigationError, Result};
pub use page::{Page, PageBuilder};

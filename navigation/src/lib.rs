#[macro_use]
extern crate derive_builder;

pub mod navigation;

// Re-export commonly used types
pub use navigation::{Container, NavItem, NavItemBuilder, NavigationItem};

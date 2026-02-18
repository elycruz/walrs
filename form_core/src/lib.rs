//! # walrs_form_core
//!
//! Core types for the walrs form ecosystem.
//!
//! This crate provides shared foundation types used across all form-related crates:
//!
//! - [`Value`] - Re-export of `serde_json::Value` for dynamic form data
//! - [`ValueExt`] - Extension trait with form-specific helper methods
//! - [`Attributes`] - HTML attributes storage and rendering
//!
//! ## Example
//!
//! ```rust
//! use walrs_form_core::{Value, ValueExt, Attributes};
//! use serde_json::json;
//!
//! // Check if a value is empty
//! let value = json!(null);
//! assert!(value.is_empty_value());
//!
//! // Build HTML attributes
//! let mut attrs = Attributes::new();
//! attrs.insert("class", "form-control");
//! attrs.insert("id", "email");
//! println!("{}", attrs.to_html());
//! ```

pub mod attributes;
pub mod value;

pub use attributes::Attributes;
pub use value::{Value, ValueExt};


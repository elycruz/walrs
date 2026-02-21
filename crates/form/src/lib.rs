//! # walrs_form
//!
//! Form elements and structure for the walrs form ecosystem.
//!
//! This crate provides data structures for representing HTML form elements
//! in server-side environments. All structures are serializable and designed
//! for use in both Rust web frameworks and JavaScript/TypeScript via WASM.
//!
//! ## Features
//!
//! - **Form Elements**: [`InputElement`], [`SelectElement`], [`TextareaElement`], [`ButtonElement`]
//! - **Type Enums**: [`InputType`], [`SelectType`], [`ButtonType`]
//! - **Containers**: [`Form`], [`FieldsetElement`]
//! - **Data Handling**: [`FormData`] with path-based access for nested structures
//! - **Polymorphism**: [`Element`] enum for handling mixed element collections
//!
//! ## Example
//!
//! ```rust
//! use walrs_form::{Form, FormMethod, InputElement, InputType, ButtonElement, ButtonType, FormData};
//! use serde_json::json;
//!
//! // Create a login form
//! let mut form = Form::new("login");
//! form.action = Some("/api/login".to_string());
//! form.method = Some(FormMethod::Post);
//!
//! // Add form elements
//! form.add_element(InputElement::new("username", InputType::Text).into());
//! form.add_element(InputElement::new("password", InputType::Password).into());
//! form.add_element(ButtonElement::with_label("Sign In", ButtonType::Submit).into());
//!
//! // Bind data to the form
//! let mut data = FormData::new();
//! data.insert("username", json!("john_doe"));
//! form.bind_data(data);
//!
//! // Serialize to JSON
//! let json = serde_json::to_string_pretty(&form).unwrap();
//! println!("{}", json);
//! ```
//!
//! ## Architecture
//!
//! This crate is part of the walrs form ecosystem:
//!
//! - `walrs_validation`: Shared types (`Value`, `Attributes`) and validation rules
//! - `walrs_inputfilter`: Field-level validation (`Field<T>`, `FieldFilter`)
//! - `walrs_form`: Form structure and elements (this crate)
//! - `walrs_validation`: Validation rules
//! - `walrs_filter`: Value transformation filters
// Type enums
pub mod button_type;
pub mod input_type;
pub mod select_type;
// Element structs
pub mod button_element;
pub mod fieldset_element;
pub mod input_element;
pub mod select_element;
pub mod select_option;
pub mod textarea_element;
// Core types
pub mod element;
pub mod form;
pub mod form_data;
pub mod path;
// Re-exports for convenience
pub use button_element::{ButtonElement, ButtonElementBuilder};
pub use button_type::ButtonType;
pub use element::Element;
pub use fieldset_element::{FieldsetElement, FieldsetElementBuilder};
pub use form::{Form, FormBuilder, FormEnctype, FormMethod};
pub use form_data::FormData;
pub use input_element::{InputElement, InputElementBuilder};
pub use input_type::InputType;
pub use path::{PathError, PathSegment, parse_path};
pub use select_element::{SelectElement, SelectElementBuilder};
pub use select_option::{SelectOption, SelectOptionBuilder};
pub use select_type::SelectType;
pub use textarea_element::{TextareaElement, TextareaElementBuilder};
// Re-export core types
pub use walrs_validation::{Attributes, Value, ValueExt};
pub use walrs_inputfilter::{Field, FieldBuilder, FieldFilter, FormViolations};

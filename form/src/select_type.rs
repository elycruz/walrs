//! Select element types.
//!
//! This module provides the [`SelectType`] enum for distinguishing between
//! single and multiple selection modes.
//!
//! # Example
//!
//! ```rust
//! use walrs_form::SelectType;
//!
//! let single = SelectType::Single;
//! let multi = SelectType::Multiple;
//!
//! assert_eq!(single, SelectType::default());
//! ```
use serde::{Deserialize, Serialize};
/// Select element types.
///
/// Represents whether a `<select>` element allows single or multiple selection.
///
/// # Example
///
/// ```rust
/// use walrs_form::SelectType;
///
/// let select_type = SelectType::Multiple;
/// let json = serde_json::to_string(&select_type).unwrap();
/// assert_eq!(json, r#""select-multiple""#);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectType {
    /// Single selection (default).
    #[default]
    #[serde(rename = "select")]
    Single,
    /// Multiple selection allowed.
    #[serde(rename = "select-multiple")]
    Multiple,
}
impl std::fmt::Display for SelectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SelectType::Single => "select",
            SelectType::Multiple => "select-multiple",
        };
        write!(f, "{}", s)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default() {
        assert_eq!(SelectType::default(), SelectType::Single);
    }
    #[test]
    fn test_display() {
        assert_eq!(SelectType::Single.to_string(), "select");
        assert_eq!(SelectType::Multiple.to_string(), "select-multiple");
    }
    #[test]
    fn test_serialization() {
        let json = serde_json::to_string(&SelectType::Single).unwrap();
        assert_eq!(json, r#""select""#);
        let json = serde_json::to_string(&SelectType::Multiple).unwrap();
        assert_eq!(json, r#""select-multiple""#);
    }
    #[test]
    fn test_deserialization() {
        let st: SelectType = serde_json::from_str(r#""select-multiple""#).unwrap();
        assert_eq!(st, SelectType::Multiple);
    }
}

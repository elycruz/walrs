//! Button type enum.
//!
//! This module provides the [`ButtonType`] enum representing HTML button types.
//!
//! # Example
//!
//! ```rust
//! use walrs_form::ButtonType;
//!
//! let btn_type = ButtonType::Submit;
//! assert_eq!(btn_type, ButtonType::Submit);
//! ```
use serde::{Deserialize, Serialize};
/// HTML button types.
///
/// Represents the `type` attribute of an HTML `<button>` element.
///
/// # Example
///
/// ```rust
/// use walrs_form::ButtonType;
///
/// let submit = ButtonType::Submit;
/// let reset = ButtonType::Reset;
/// let button = ButtonType::Button;
///
/// // Serialization
/// let json = serde_json::to_string(&submit).unwrap();
/// assert_eq!(json, r#""submit""#);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ButtonType {
    /// Submit the form.
    #[default]
    Submit,
    /// Reset the form to default values.
    Reset,
    /// Generic button with no default behavior.
    Button,
}
impl std::fmt::Display for ButtonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ButtonType::Submit => "submit",
            ButtonType::Reset => "reset",
            ButtonType::Button => "button",
        };
        write!(f, "{}", s)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default() {
        assert_eq!(ButtonType::default(), ButtonType::Submit);
    }
    #[test]
    fn test_display() {
        assert_eq!(ButtonType::Submit.to_string(), "submit");
        assert_eq!(ButtonType::Reset.to_string(), "reset");
        assert_eq!(ButtonType::Button.to_string(), "button");
    }
    #[test]
    fn test_serialization() {
        let json = serde_json::to_string(&ButtonType::Submit).unwrap();
        assert_eq!(json, r#""submit""#);
    }
    #[test]
    fn test_deserialization() {
        let btn: ButtonType = serde_json::from_str(r#""reset""#).unwrap();
        assert_eq!(btn, ButtonType::Reset);
    }
}

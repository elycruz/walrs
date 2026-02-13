use std::fmt;

/// Error types for the navigation component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationError {
    /// Invalid page index
    InvalidIndex(usize),
    /// Page not found
    PageNotFound,
    /// Cycle detected in navigation tree
    CycleDetected,
    /// Invalid page configuration
    InvalidConfiguration(String),
    /// Deserialization error
    DeserializationError(String),
    /// Serialization error
    SerializationError(String),
}

impl fmt::Display for NavigationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NavigationError::InvalidIndex(idx) => {
                write!(f, "Invalid page index: {}", idx)
            }
            NavigationError::PageNotFound => {
                write!(f, "Page not found")
            }
            NavigationError::CycleDetected => {
                write!(f, "Cycle detected in navigation tree")
            }
            NavigationError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            NavigationError::DeserializationError(msg) => {
                write!(f, "Deserialization error: {}", msg)
            }
            NavigationError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
        }
    }
}

impl std::error::Error for NavigationError {}

pub type Result<T> = std::result::Result<T, NavigationError>;

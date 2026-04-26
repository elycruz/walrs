//! # walrs
//!
//! Façade crate that re-exports all `walrs_*` sub-crates as named modules.
//!
//! Each sub-crate is behind a feature flag (all enabled by default).
//! Disable features you don't need to reduce compile times:
//!
//! ```toml
//! [dependencies]
//! walrs = { version = "0.1", default-features = false, features = ["fieldfilter", "validation"] }
//! ```

#[cfg(feature = "acl")]
pub use walrs_acl as acl;

#[cfg(feature = "digraph")]
pub use walrs_digraph as digraph;

#[cfg(feature = "filter")]
pub use walrs_filter as filter;

#[cfg(feature = "graph")]
pub use walrs_graph as graph;

#[cfg(feature = "fieldfilter")]
pub use walrs_fieldfilter as fieldfilter;

#[cfg(feature = "navigation")]
pub use walrs_navigation as navigation;

#[cfg(feature = "rbac")]
pub use walrs_rbac as rbac;

#[cfg(feature = "validation")]
pub use walrs_validation as validation;

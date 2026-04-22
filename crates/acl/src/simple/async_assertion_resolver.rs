//! Async resolver trait for conditional assertions.
//!
//! [`AsyncAssertionResolver`] is the async counterpart to
//! [`AssertionResolver`](crate::simple::AssertionResolver): it maps an
//! [`AssertionKey`](crate::simple::AssertionKey) to a boolean, but awaits the
//! result. Use it when evaluating an assertion requires I/O (database lookups,
//! policy services, remote feature flags, etc.).
//!
//! This module is only compiled with the `async` feature.
//!
//! # Example
//!
//! ```rust,ignore
//! use walrs_acl::simple::{AclBuilder, AsyncAssertionResolver};
//! use async_trait::async_trait;
//!
//! struct Resolver;
//!
//! #[async_trait]
//! impl AsyncAssertionResolver for Resolver {
//!   async fn evaluate(&self, key: &str) -> bool {
//!     key == "is_owner"
//!   }
//! }
//! ```

use async_trait::async_trait;

/// Resolves an assertion key to `true` / `false` at check time, asynchronously.
///
/// Unknown keys should return `false` (conservative default), mirroring the
/// sync [`AssertionResolver`](crate::simple::AssertionResolver) contract.
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[async_trait]
pub trait AsyncAssertionResolver: Send + Sync {
  /// Return `true` if the assertion identified by `key` holds in the current
  /// context; `false` otherwise (including for unknown keys).
  async fn evaluate(&self, key: &str) -> bool;
}

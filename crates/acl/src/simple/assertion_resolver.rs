//! Resolver trait for conditional assertions.
//!
//! An [`AssertionResolver`] maps an [`AssertionKey`](crate::simple::AssertionKey)
//! to a boolean at check time. Callers implement this trait (or pass a closure)
//! to resolve [`Rule::AllowIf`](crate::simple::Rule::AllowIf) /
//! [`Rule::DenyIf`](crate::simple::Rule::DenyIf) variants.
//!
//! # Why the registry lives outside the crate
//!
//! Assertions often close over runtime state (current user, request, time of
//! day, feature flags, etc.) that can't be serialized. Keeping the registry in
//! the caller lets the [`Acl`](crate::simple::Acl) stay fully serializable and
//! WASM-friendly; only the keys are stored in rules.
//!
//! # Example
//!
//! ```rust
//! use walrs_acl::simple::{AclBuilder, AssertionResolver};
//!
//! let acl = AclBuilder::new()
//!   .add_role("editor", None)?
//!   .add_resource("post", None)?
//!   .allow_if(Some(&["editor"]), Some(&["post"]), Some(&["edit"]), "is_owner")?
//!   .build()?;
//!
//! // Pass a closure as a resolver.
//! let is_owner = true;
//! let resolver = |key: &str| -> bool { key == "is_owner" && is_owner };
//!
//! assert!(acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver));
//! # Ok::<(), String>(())
//! ```

/// Resolves an assertion key to `true` / `false` at check time.
///
/// Unknown keys should return `false` (conservative default).
pub trait AssertionResolver {
  /// Return `true` if the assertion identified by `key` holds in the current
  /// context; `false` otherwise (including for unknown keys).
  fn evaluate(&self, key: &str) -> bool;
}

/// Blanket impl — any `Fn(&str) -> bool` is a valid resolver.
impl<F> AssertionResolver for F
where
  F: Fn(&str) -> bool,
{
  fn evaluate(&self, key: &str) -> bool {
    self(key)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn closure_as_resolver() {
    let r = |k: &str| k == "yes";
    assert!(r.evaluate("yes"));
    assert!(!r.evaluate("no"));
  }

  struct Static {
    val: bool,
  }

  impl AssertionResolver for Static {
    fn evaluate(&self, _: &str) -> bool {
      self.val
    }
  }

  #[test]
  fn struct_as_resolver() {
    assert!(Static { val: true }.evaluate("anything"));
    assert!(!Static { val: false }.evaluate("anything"));
  }
}

use crate::simple::types::AssertionKey;

/// A rule governing access to a (resource, role, privilege) triple.
///
/// Two unconditional variants (`Allow`, `Deny`) behave as before.
/// Two conditional variants (`AllowIf`, `DenyIf`) carry an [`AssertionKey`] — an
/// opaque string identifier that a caller-supplied
/// [`AssertionResolver`](crate::simple::AssertionResolver) resolves to a boolean
/// at check time.
///
/// The registry that maps keys to predicates lives in the caller, not in this
/// crate. This keeps the ACL structure serializable and WASM-friendly.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Rule {
  Allow,
  Deny,
  /// Conditional allow: resolves to `Allow` iff the resolver evaluates the key
  /// to `true`, otherwise treated as non-allow.
  AllowIf(AssertionKey),
  /// Conditional deny: resolves to `Deny` iff the resolver evaluates the key
  /// to `true`, otherwise treated as non-deny.
  DenyIf(AssertionKey),
}

impl Rule {
  /// Returns `true` if this rule is an `AllowIf` or `DenyIf` variant.
  pub fn is_conditional(&self) -> bool {
    matches!(self, Rule::AllowIf(_) | Rule::DenyIf(_))
  }

  /// Returns the assertion key for a conditional rule, or `None` for
  /// unconditional rules.
  pub fn assertion_key(&self) -> Option<&str> {
    match self {
      Rule::AllowIf(k) | Rule::DenyIf(k) => Some(k.as_str()),
      _ => None,
    }
  }

  /// Returns `true` if this rule is in the "allowing family" — i.e., either
  /// an unconditional `Allow` or a conditional `AllowIf`.
  pub fn is_allowing_family(&self) -> bool {
    matches!(self, Rule::Allow | Rule::AllowIf(_))
  }

  /// Returns `true` if this rule is in the "denying family" — i.e., either
  /// an unconditional `Deny` or a conditional `DenyIf`.
  pub fn is_denying_family(&self) -> bool {
    matches!(self, Rule::Deny | Rule::DenyIf(_))
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuleContextScope {
  PerSymbol,
  ForAllSymbols,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::prelude::ToString;

  #[test]
  fn is_conditional() {
    assert!(!Rule::Allow.is_conditional());
    assert!(!Rule::Deny.is_conditional());
    assert!(Rule::AllowIf("k".to_string()).is_conditional());
    assert!(Rule::DenyIf("k".to_string()).is_conditional());
  }

  #[test]
  fn assertion_key() {
    assert_eq!(Rule::Allow.assertion_key(), None);
    assert_eq!(Rule::Deny.assertion_key(), None);
    assert_eq!(
      Rule::AllowIf("owner".to_string()).assertion_key(),
      Some("owner")
    );
    assert_eq!(
      Rule::DenyIf("closed".to_string()).assertion_key(),
      Some("closed")
    );
  }

  #[test]
  fn rule_families() {
    assert!(Rule::Allow.is_allowing_family());
    assert!(Rule::AllowIf("k".to_string()).is_allowing_family());
    assert!(!Rule::Deny.is_allowing_family());
    assert!(!Rule::DenyIf("k".to_string()).is_allowing_family());

    assert!(Rule::Deny.is_denying_family());
    assert!(Rule::DenyIf("k".to_string()).is_denying_family());
    assert!(!Rule::Allow.is_denying_family());
    assert!(!Rule::AllowIf("k".to_string()).is_denying_family());
  }
}

use crate::prelude::String;

pub type Role = String;
pub type Resource = String;
pub type Privilege = String;

/// An opaque identifier for a conditional assertion.
///
/// Paired with [`Rule::AllowIf`](crate::simple::Rule::AllowIf) /
/// [`Rule::DenyIf`](crate::simple::Rule::DenyIf). The caller resolves the key
/// to a boolean via an
/// [`AssertionResolver`](crate::simple::AssertionResolver) at check time.
pub type AssertionKey = String;

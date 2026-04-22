// acl_data.rs
use crate::prelude::{String, Vec};
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "std")]
use core::convert::TryFrom;
#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "std")]
use std::io::BufReader;

/// Serializable representation of an [`Acl`](crate::simple::Acl).
///
/// `allow_if` / `deny_if` have the same outer shape as `allow` / `deny`, but the
/// innermost item is a `(privilege, assertion_key)` tuple rather than a plain
/// privilege. Both fields are skipped during serialization when `None`, keeping
/// existing on-disk JSON backwards-compatible.
#[allow(clippy::type_complexity)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclData {
  pub roles: Option<Vec<(String, Option<Vec<String>>)>>,
  pub resources: Option<Vec<(String, Option<Vec<String>>)>>,
  pub allow: Option<Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>>,
  pub deny: Option<Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>>,

  /// Conditional allow rules. Same outer shape as `allow`; innermost list
  /// carries `(privilege, assertion_key)` pairs.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub allow_if: Option<Vec<(String, Option<Vec<(String, Option<Vec<(String, String)>>)>>)>>,

  /// Conditional deny rules. Same outer shape as `deny`; innermost list
  /// carries `(privilege, assertion_key)` pairs.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub deny_if: Option<Vec<(String, Option<Vec<(String, Option<Vec<(String, String)>>)>>)>>,
}

#[cfg(feature = "std")]
impl TryFrom<&mut File> for AclData {
  type Error = serde_json::Error;

  fn try_from(file: &mut File) -> Result<Self, Self::Error> {
    let buf = BufReader::new(file);
    serde_json::from_reader(buf)
  }
}

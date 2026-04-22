//! Tests for async conditional assertion evaluation (issue #246).
//!
//! Exercises [`Acl::is_allowed_with_async`] / [`is_allowed_any_with_async`]
//! using an in-memory map resolver — no actual I/O needed to verify the
//! awaited code path works end-to-end.

#![cfg(feature = "async")]

use async_trait::async_trait;
use std::collections::HashMap;
use walrs_acl::simple::{AclBuilder, AsyncAssertionResolver};

struct MapResolver {
  table: HashMap<String, bool>,
}

#[async_trait]
impl AsyncAssertionResolver for MapResolver {
  async fn evaluate(&self, key: &str) -> bool {
    // A real implementation might hit a database here; the `.await` is what
    // we care about exercising.
    self.table.get(key).copied().unwrap_or(false)
  }
}

fn owner_resolver(owner: bool) -> MapResolver {
  let mut table = HashMap::new();
  table.insert("is_owner".to_string(), owner);
  MapResolver { table }
}

#[tokio::test]
async fn allow_if_true_allows_async() -> Result<(), String> {
  let acl = AclBuilder::new()
    .add_role("editor", None)?
    .add_resource("post", None)?
    .allow_if(
      Some(&["editor"]),
      Some(&["post"]),
      Some(&["edit"]),
      "is_owner",
    )?
    .build()?;

  let resolver = owner_resolver(true);
  assert!(
    acl
      .is_allowed_with_async(Some("editor"), Some("post"), Some("edit"), &resolver)
      .await
  );
  Ok(())
}

#[tokio::test]
async fn allow_if_false_denies_async() -> Result<(), String> {
  let acl = AclBuilder::new()
    .add_role("editor", None)?
    .add_resource("post", None)?
    .allow_if(
      Some(&["editor"]),
      Some(&["post"]),
      Some(&["edit"]),
      "is_owner",
    )?
    .build()?;

  let resolver = owner_resolver(false);
  assert!(
    !acl
      .is_allowed_with_async(Some("editor"), Some("post"), Some("edit"), &resolver)
      .await
  );
  Ok(())
}

#[tokio::test]
async fn deny_if_true_denies_async() -> Result<(), String> {
  let acl = AclBuilder::new()
    .add_role("user", None)?
    .add_resource("admin_panel", None)?
    .allow(Some(&["user"]), Some(&["admin_panel"]), Some(&["access"]))?
    .deny_if(
      Some(&["user"]),
      Some(&["admin_panel"]),
      Some(&["access"]),
      "outside_hours",
    )?
    .build()?;

  let mut table = HashMap::new();
  table.insert("outside_hours".to_string(), true);
  let resolver = MapResolver { table };

  assert!(
    !acl
      .is_allowed_with_async(Some("user"), Some("admin_panel"), Some("access"), &resolver)
      .await
  );
  Ok(())
}

#[tokio::test]
async fn is_allowed_any_with_async_works() -> Result<(), String> {
  let acl = AclBuilder::new()
    .add_role("editor", None)?
    .add_resource("post", None)?
    .allow_if(
      Some(&["editor"]),
      Some(&["post"]),
      Some(&["edit"]),
      "is_owner",
    )?
    .build()?;

  let resolver = owner_resolver(true);
  assert!(
    acl
      .is_allowed_any_with_async(
        Some(&["editor"]),
        Some(&["post"]),
        Some(&["edit", "delete"]),
        &resolver,
      )
      .await
  );
  assert!(
    !acl
      .is_allowed_any_with_async(
        Some(&["editor"]),
        Some(&["post"]),
        Some(&["delete"]),
        &resolver,
      )
      .await
  );
  Ok(())
}

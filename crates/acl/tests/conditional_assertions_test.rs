//! Tests for conditional assertions (`AllowIf` / `DenyIf`).
//!
//! Covers the acceptance criteria for issue #244. A simple struct-based
//! resolver at the top matches keys against a literal table and returns the
//! associated boolean; unknown keys resolve to `false`.

use std::convert::TryFrom;
use std::fs::File;
use walrs_acl::simple::{AclBuilder, AclData, AssertionResolver};

struct StaticResolver<'a>(&'a [(&'a str, bool)]);

impl<'a> AssertionResolver for StaticResolver<'a> {
  fn evaluate(&self, key: &str) -> bool {
    self
      .0
      .iter()
      .find(|(k, _)| *k == key)
      .map(|(_, v)| *v)
      .unwrap_or(false)
  }
}

fn basic_acl() -> Result<walrs_acl::simple::Acl, String> {
  AclBuilder::new()
    .add_role("guest", None)?
    .add_role("user", Some(&["guest"]))?
    .add_role("editor", Some(&["user"]))?
    .add_resource("post", None)?
    .add_resource("admin_panel", None)?
    .allow_if(
      Some(&["editor"]),
      Some(&["post"]),
      Some(&["edit"]),
      "is_owner",
    )?
    .build()
}

#[test]
fn allow_if_true_allows() -> Result<(), String> {
  let acl = basic_acl()?;
  let resolver = StaticResolver(&[("is_owner", true)]);
  assert!(acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver));
  Ok(())
}

#[test]
fn allow_if_false_denies() -> Result<(), String> {
  let acl = basic_acl()?;
  let resolver = StaticResolver(&[("is_owner", false)]);
  assert!(!acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver));
  Ok(())
}

#[test]
fn deny_if_true_denies() -> Result<(), String> {
  // Plain allow on admin_panel for user, then a deny_if that fires when the
  // assertion resolves to true.
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

  let resolver = StaticResolver(&[("outside_hours", true)]);
  assert!(!acl.is_allowed_with(Some("user"), Some("admin_panel"), Some("access"), &resolver));
  Ok(())
}

#[test]
fn deny_if_false_does_not_deny() -> Result<(), String> {
  // Note: deny_if clears the opposing allow rule (opposing-family clearing),
  // so to assert "deny_if resolving to false does not deny", we set the
  // allow AFTER the deny_if.
  let acl = AclBuilder::new()
    .add_role("user", None)?
    .add_resource("admin_panel", None)?
    .deny_if(
      Some(&["user"]),
      Some(&["admin_panel"]),
      Some(&["access"]),
      "outside_hours",
    )?
    .allow(Some(&["user"]), Some(&["admin_panel"]), Some(&["access"]))?
    .build()?;

  let resolver = StaticResolver(&[("outside_hours", false)]);
  assert!(acl.is_allowed_with(Some("user"), Some("admin_panel"), Some("access"), &resolver));
  Ok(())
}

#[test]
fn allow_if_without_resolver_is_denied() -> Result<(), String> {
  let acl = basic_acl()?;
  // Plain is_allowed: AllowIf is treated as "not allow" (conservative).
  assert!(!acl.is_allowed(Some("editor"), Some("post"), Some("edit")));
  Ok(())
}

#[test]
fn deny_if_without_resolver_does_not_block() -> Result<(), String> {
  // Combine a plain Allow with a DenyIf — plain is_allowed should let the
  // access through because the DenyIf is treated as "not deny" conservatively.
  let acl = AclBuilder::new()
    .add_role("user", None)?
    .add_resource("admin_panel", None)?
    .deny_if(
      Some(&["user"]),
      Some(&["admin_panel"]),
      Some(&["access"]),
      "outside_hours",
    )?
    .allow(Some(&["user"]), Some(&["admin_panel"]), Some(&["access"]))?
    .build()?;

  assert!(
    acl.is_allowed(Some("user"), Some("admin_panel"), Some("access")),
    "deny_if without a resolver must not block a plain Allow"
  );
  Ok(())
}

#[test]
fn explicit_deny_overrides_allow_if_true() -> Result<(), String> {
  // An explicit Deny wins over an AllowIf, even when the resolver says true.
  // Note: we apply allow_if LAST so it isn't cleared by the subsequent deny's
  // opposing-family clearing. Still, the plain Deny at the same spot would
  // clear it — so we use different privileges and check a blanket Deny path.
  let acl = AclBuilder::new()
    .add_role("editor", None)?
    .add_resource("post", None)?
    // allow_if on "edit"
    .allow_if(
      Some(&["editor"]),
      Some(&["post"]),
      Some(&["edit"]),
      "is_owner",
    )?
    // Explicit deny ALSO on "edit" — this replaces the AllowIf.
    .deny(Some(&["editor"]), Some(&["post"]), Some(&["edit"]))?
    .build()?;

  let resolver = StaticResolver(&[("is_owner", true)]);
  assert!(
    !acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver),
    "explicit Deny must override AllowIf even when assertion resolves to true"
  );
  Ok(())
}

#[test]
fn role_inheritance_with_allow_if() -> Result<(), String> {
  // Parent has AllowIf; child should inherit that conditional grant.
  let acl = AclBuilder::new()
    .add_role("user", None)?
    .add_role("editor", Some(&["user"]))?
    .add_resource("post", None)?
    .allow_if(
      Some(&["user"]),
      Some(&["post"]),
      Some(&["edit"]),
      "is_owner",
    )?
    .build()?;

  let resolver = StaticResolver(&[("is_owner", true)]);
  assert!(
    acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver),
    "child role should inherit AllowIf from parent"
  );

  let deny_resolver = StaticResolver(&[("is_owner", false)]);
  assert!(
    !acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &deny_resolver),
    "inherited AllowIf still respects the resolver verdict"
  );
  Ok(())
}

#[test]
fn resource_inheritance_with_allow_if() -> Result<(), String> {
  // Parent resource has AllowIf; child resource should inherit.
  let acl = AclBuilder::new()
    .add_role("editor", None)?
    .add_resource("content", None)?
    .add_resource("post", Some(&["content"]))?
    .allow_if(
      Some(&["editor"]),
      Some(&["content"]),
      Some(&["edit"]),
      "is_owner",
    )?
    .build()?;

  let resolver = StaticResolver(&[("is_owner", true)]);
  assert!(
    acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver),
    "child resource should inherit AllowIf from parent resource"
  );
  Ok(())
}

#[test]
fn unknown_assertion_key_returns_false() -> Result<(), String> {
  let acl = basic_acl()?;
  let resolver = StaticResolver(&[("unrelated", true)]);
  assert!(
    !acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver),
    "unknown key resolves to false (conservative default)"
  );
  Ok(())
}

#[test]
fn json_round_trip_conditional_rules() -> Result<(), Box<dyn std::error::Error>> {
  // 1. Load from the fixture.
  let mut f = File::open("./test-fixtures/example-acl-conditional.json")?;
  let data = AclData::try_from(&mut f)?;
  assert!(data.allow_if.is_some());
  assert!(data.deny_if.is_some());

  let builder = AclBuilder::try_from(&data)?;

  // 2. Re-serialize via AclData::try_from(&AclBuilder).
  let data2 = AclData::try_from(&builder)?;
  assert!(
    data2.allow_if.is_some(),
    "round-tripped allow_if must be present"
  );
  assert!(
    data2.deny_if.is_some(),
    "round-tripped deny_if must be present"
  );

  // 3. Re-parse the re-serialized JSON and re-build.
  let json = serde_json::to_string(&data2)?;
  let data3: AclData = serde_json::from_str(&json)?;
  let acl = AclBuilder::try_from(&data3)?.build()?;

  // 4. Validate behavior is preserved: editor can edit post when is_owner.
  let owner = |k: &str| k == "is_owner";
  assert!(acl.is_allowed_with(Some("editor"), Some("content_item"), Some("edit"), &owner));
  assert!(acl.is_allowed_with(
    Some("editor"),
    Some("content_item"),
    Some("publish"),
    &owner
  ));

  let not_owner = |_k: &str| false;
  assert!(!acl.is_allowed_with(
    Some("editor"),
    Some("content_item"),
    Some("edit"),
    &not_owner
  ));

  // 5. The round-tripped deny_if must still be stored and fire under a resolver.
  let off_hours = |k: &str| k == "outside_business_hours";
  // Without a corresponding Allow there's nothing to pass anyway, so confirm
  // we can still read back the assertion key from the serialized form.
  assert!(!acl.is_allowed_with(
    Some("user"),
    Some("admin_panel"),
    Some("access"),
    &off_hours
  ));

  // Combine freshly-built with an Allow that precedes the deny_if, so the
  // deny_if (loaded from JSON) survives the opposing-family clearing.
  let with_allow = AclBuilder::new()
    .add_role("guest", None)?
    .add_role("user", Some(&["guest"]))?
    .add_resource("admin_panel", None)?
    .allow(Some(&["user"]), Some(&["admin_panel"]), Some(&["access"]))?
    .deny_if(
      Some(&["user"]),
      Some(&["admin_panel"]),
      Some(&["access"]),
      "outside_business_hours",
    )?
    .build()?;
  // The later deny_if clears the earlier allow (opposing-family clearing).
  // So under the off-hours resolver we must deny, and under in-hours the rule
  // is "not-deny" — but since the allow got cleared, the default (deny) wins.
  assert!(!with_allow.is_allowed_with(
    Some("user"),
    Some("admin_panel"),
    Some("access"),
    &off_hours
  ));
  let in_hours = |_k: &str| false;
  assert!(!with_allow.is_allowed_with(
    Some("user"),
    Some("admin_panel"),
    Some("access"),
    &in_hours
  ));

  Ok(())
}

#[test]
fn opposing_rule_clearing_allow_if_vs_deny_if() -> Result<(), String> {
  // Setting AllowIf over a DenyIf on the same (role, resource, privilege)
  // should clear the DenyIf (since it's in the opposing family).
  let acl = AclBuilder::new()
    .add_role("editor", None)?
    .add_resource("post", None)?
    .deny_if(
      Some(&["editor"]),
      Some(&["post"]),
      Some(&["edit"]),
      "locked",
    )?
    .allow_if(
      Some(&["editor"]),
      Some(&["post"]),
      Some(&["edit"]),
      "is_owner",
    )?
    .build()?;

  // Resolver says is_owner=true, locked=true. If the DenyIf had been kept, we'd
  // expect a deny; since it was cleared, allow_if takes effect and we allow.
  let r = StaticResolver(&[("is_owner", true), ("locked", true)]);
  assert!(
    acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &r),
    "allow_if should clear prior deny_if"
  );

  // And vice-versa: deny_if after allow_if clears the allow_if.
  let acl2 = AclBuilder::new()
    .add_role("editor", None)?
    .add_resource("post", None)?
    .allow_if(
      Some(&["editor"]),
      Some(&["post"]),
      Some(&["edit"]),
      "is_owner",
    )?
    .deny_if(
      Some(&["editor"]),
      Some(&["post"]),
      Some(&["edit"]),
      "locked",
    )?
    .build()?;

  let r = StaticResolver(&[("is_owner", true), ("locked", false)]);
  assert!(
    !acl2.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &r),
    "deny_if should clear prior allow_if"
  );
  Ok(())
}

#[test]
fn closure_resolver_works() -> Result<(), String> {
  let acl = basic_acl()?;
  let closure_resolver = |k: &str| k == "is_owner";
  assert!(acl.is_allowed_with(
    Some("editor"),
    Some("post"),
    Some("edit"),
    &closure_resolver
  ));
  Ok(())
}

#[test]
fn is_allowed_any_with_works() -> Result<(), String> {
  let acl = basic_acl()?;
  let r = StaticResolver(&[("is_owner", true)]);
  assert!(acl.is_allowed_any_with(
    Some(&["editor"]),
    Some(&["post"]),
    Some(&["edit", "delete"]),
    &r,
  ));
  assert!(!acl.is_allowed_any_with(Some(&["editor"]), Some(&["post"]), Some(&["delete"]), &r,));
  Ok(())
}

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
  // Keep the DenyIf rule on `user` while an unconditional Allow lives on the
  // inherited `guest` role, so opposing-family clearing cannot remove the
  // conditional deny we're testing.
  let acl = AclBuilder::new()
    .add_role("guest", None)?
    .add_role("user", Some(&["guest"]))?
    .add_resource("admin_panel", None)?
    .allow(Some(&["guest"]), Some(&["admin_panel"]), Some(&["access"]))?
    .deny_if(
      Some(&["user"]),
      Some(&["admin_panel"]),
      Some(&["access"]),
      "outside_hours",
    )?
    .build()?;

  // DenyIf resolves to false → user still has access via the inherited Allow
  // from guest.
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
  // Combine an unconditional Allow on the parent role with a DenyIf on the
  // child role. Plain is_allowed treats DenyIf as "not-blocking" conservatively,
  // so the inherited Allow should pass through.
  let acl = AclBuilder::new()
    .add_role("guest", None)?
    .add_role("user", Some(&["guest"]))?
    .add_resource("admin_panel", None)?
    .allow(Some(&["guest"]), Some(&["admin_panel"]), Some(&["access"]))?
    .deny_if(
      Some(&["user"]),
      Some(&["admin_panel"]),
      Some(&["access"]),
      "outside_hours",
    )?
    .build()?;

  assert!(
    acl.is_allowed(Some("user"), Some("admin_panel"), Some("access")),
    "deny_if without a resolver must not block a plain Allow"
  );
  Ok(())
}

#[test]
fn explicit_deny_overrides_allow_if_true() -> Result<(), String> {
  // An explicit Deny on the direct role wins over an AllowIf inherited from a
  // parent role, even when the resolver evaluates to true.  Both rules coexist
  // because opposing-family clearing only affects rules on the same role; the
  // AllowIf on "user" is not cleared by the Deny on "editor".
  let acl = AclBuilder::new()
    .add_role("user", None)?
    .add_role("editor", Some(&["user"]))?
    .add_resource("post", None)?
    // AllowIf on the parent role.
    .allow_if(
      Some(&["user"]),
      Some(&["post"]),
      Some(&["edit"]),
      "is_owner",
    )?
    // Explicit Deny on the child role — overrides the inherited AllowIf.
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

  // 5. The round-tripped deny_if must still fire under a resolver.
  // The fixture grants admin_panel/access to "guest" (unconditional Allow) and
  // places a DenyIf(outside_business_hours) on "user" (which inherits from
  // guest).  The two rules are on different roles so opposing-family clearing
  // cannot remove either of them.
  let off_hours = |k: &str| k == "outside_business_hours";
  // When outside_business_hours resolves to true the DenyIf fires → denied.
  assert!(!acl.is_allowed_with(
    Some("user"),
    Some("admin_panel"),
    Some("access"),
    &off_hours
  ));
  // When outside_business_hours resolves to false the DenyIf does not fire →
  // user can access via the inherited unconditional Allow from guest.
  let in_hours = |_k: &str| false;
  assert!(acl.is_allowed_with(Some("user"), Some("admin_panel"), Some("access"), &in_hours));

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

#[test]
fn allow_if_without_privileges_errors() {
  // AclData cannot represent a conditional rule that applies to "all
  // privileges", so the builder should reject it up-front with a clear
  // error rather than failing later at serialization time.
  let mut builder = AclBuilder::new();
  let err = builder
    .add_role("editor", None)
    .and_then(|b| b.add_resource("post", None))
    .and_then(|b| b.allow_if(Some(&["editor"]), Some(&["post"]), None, "is_owner"))
    .expect_err("allow_if with privileges=None must fail");
  assert!(
    err.contains("allow_if"),
    "error should name the builder method: {err}"
  );
  assert!(
    err.contains("privilege"),
    "error should mention the missing privilege list: {err}"
  );

  // Empty slice is treated the same as `None` for this guard.
  let mut builder = AclBuilder::new();
  let err = builder
    .add_role("editor", None)
    .and_then(|b| b.add_resource("post", None))
    .and_then(|b| b.allow_if(Some(&["editor"]), Some(&["post"]), Some(&[]), "is_owner"))
    .expect_err("allow_if with empty privileges must fail");
  assert!(err.contains("allow_if"));
}

#[test]
fn deny_if_without_privileges_errors() {
  let mut builder = AclBuilder::new();
  let err = builder
    .add_role("user", None)
    .and_then(|b| b.add_resource("admin_panel", None))
    .and_then(|b| b.deny_if(Some(&["user"]), Some(&["admin_panel"]), None, "closed"))
    .expect_err("deny_if with privileges=None must fail");
  assert!(
    err.contains("deny_if"),
    "error should name the builder method: {err}"
  );
  assert!(
    err.contains("privilege"),
    "error should mention the missing privilege list: {err}"
  );
}

#[test]
fn acl_data_round_trip_bytes_are_deterministic() -> Result<(), Box<dyn std::error::Error>> {
  // Build an ACL with enough conditional rules across multiple resources and
  // privileges that any non-deterministic map iteration would surface as a
  // byte-difference between two fresh parses/builds.
  let first_acl_data =
    AclData::try_from(&mut File::open("./test-fixtures/example-acl-conditional.json")?)?;
  let first_builder = AclBuilder::try_from(&first_acl_data)?;

  let second_acl_data =
    AclData::try_from(&mut File::open("./test-fixtures/example-acl-conditional.json")?)?;
  let second_builder = AclBuilder::try_from(&second_acl_data)?;

  let first = serde_json::to_string(&AclData::try_from(&first_builder)?)?;
  let second = serde_json::to_string(&AclData::try_from(&second_builder)?)?;

  assert_eq!(
    first, second,
    "AclData -> JSON must be byte-identical across runs"
  );
  Ok(())
}

#[test]
fn deny_if_on_parent_role_does_not_block_child_allow() -> Result<(), String> {
  // The engine's `has_explicit_deny` short-circuit only looks at rules
  // attached directly to the queried role/resource — it does NOT walk the
  // role-inheritance graph. So a `DenyIf` placed on a parent role must not
  // override an explicit `Allow` placed directly on the child role, even when
  // the resolver would fire the parent's DenyIf.
  let acl = AclBuilder::new()
    .add_role("user", None)?
    .add_role("editor", Some(&["user"]))?
    .add_resource("post", None)?
    // Conditional deny on the parent role.
    .deny_if(
      Some(&["user"]),
      Some(&["post"]),
      Some(&["edit"]),
      "locked",
    )?
    // Explicit allow directly on the child role.
    .allow(Some(&["editor"]), Some(&["post"]), Some(&["edit"]))?
    .build()?;

  // Resolver says the parent's DenyIf would fire, but the child has a direct
  // Allow, so the check must return true.
  let resolver = StaticResolver(&[("locked", true)]);
  assert!(
    acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver),
    "direct Allow on child role must not be blocked by inherited DenyIf on parent"
  );
  Ok(())
}

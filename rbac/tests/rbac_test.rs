use walrs_rbac::{RbacBuilder, RbacError};

#[test]
fn test_basic_rbac() -> Result<(), RbacError> {
  let rbac = RbacBuilder::new()
    .add_role("guest", &["read.public"], None)?
    .add_role("user", &["write.post", "comment.post"], Some(&["guest"]))?
    .add_role("editor", &["edit.post", "publish.post"], Some(&["user"]))?
    .add_role("admin", &["admin.panel", "manage.users"], Some(&["editor"]))?
    .build()?;

  // Guest
  assert!(rbac.is_granted("guest", "read.public"));
  assert!(!rbac.is_granted("guest", "write.post"));
  assert!(!rbac.is_granted("guest", "admin.panel"));

  // User (inherits from guest)
  assert!(rbac.is_granted("user", "read.public"));
  assert!(rbac.is_granted("user", "write.post"));
  assert!(rbac.is_granted("user", "comment.post"));
  assert!(!rbac.is_granted("user", "edit.post"));
  assert!(!rbac.is_granted("user", "admin.panel"));

  // Editor (inherits from user)
  assert!(rbac.is_granted("editor", "read.public"));
  assert!(rbac.is_granted("editor", "write.post"));
  assert!(rbac.is_granted("editor", "edit.post"));
  assert!(rbac.is_granted("editor", "publish.post"));
  assert!(!rbac.is_granted("editor", "admin.panel"));

  // Admin (inherits from editor)
  assert!(rbac.is_granted("admin", "read.public"));
  assert!(rbac.is_granted("admin", "write.post"));
  assert!(rbac.is_granted("admin", "edit.post"));
  assert!(rbac.is_granted("admin", "publish.post"));
  assert!(rbac.is_granted("admin", "admin.panel"));
  assert!(rbac.is_granted("admin", "manage.users"));

  Ok(())
}

#[test]
fn test_multiple_children() -> Result<(), RbacError> {
  let rbac = RbacBuilder::new()
    .add_role("reader", &["read"], None)?
    .add_role("writer", &["write"], None)?
    .add_role("deleter", &["delete"], None)?
    .add_role(
      "superuser",
      &["manage"],
      Some(&["reader", "writer", "deleter"]),
    )?
    .build()?;

  assert!(rbac.is_granted("superuser", "read"));
  assert!(rbac.is_granted("superuser", "write"));
  assert!(rbac.is_granted("superuser", "delete"));
  assert!(rbac.is_granted("superuser", "manage"));

  assert!(!rbac.is_granted("reader", "write"));
  assert!(!rbac.is_granted("writer", "read"));

  Ok(())
}

#[test]
fn test_diamond_inheritance() -> Result<(), RbacError> {
  // base <- left, base <- right, left + right <- top
  let rbac = RbacBuilder::new()
    .add_role("base", &["base.perm"], None)?
    .add_role("left", &["left.perm"], Some(&["base"]))?
    .add_role("right", &["right.perm"], Some(&["base"]))?
    .add_role("top", &["top.perm"], Some(&["left", "right"]))?
    .build()?;

  assert!(rbac.is_granted("top", "top.perm"));
  assert!(rbac.is_granted("top", "left.perm"));
  assert!(rbac.is_granted("top", "right.perm"));
  assert!(rbac.is_granted("top", "base.perm"));

  Ok(())
}

#[test]
fn test_no_false_positives() -> Result<(), RbacError> {
  let rbac = RbacBuilder::new()
    .add_role("user", &["read"], None)?
    .add_role("admin", &["manage"], None)?
    .build()?;

  // They share no inheritance
  assert!(!rbac.is_granted("user", "manage"));
  assert!(!rbac.is_granted("admin", "read"));
  assert!(!rbac.is_granted("user", "nonexistent"));
  assert!(!rbac.is_granted("nonexistent", "read"));

  Ok(())
}

#[test]
fn test_cycle_detection_error() {
  let result = RbacBuilder::new()
    .add_role("a", &[], Some(&["b"]))
    .unwrap()
    .add_role("b", &[], Some(&["a"]))
    .unwrap()
    .build();

  assert!(result.is_err());
  match result {
    Err(RbacError::CycleDetected(_)) => {}
    other => panic!("Expected CycleDetected, got {:?}", other),
  }
}

#[test]
fn test_missing_child_error() {
  let result = RbacBuilder::new()
    .add_role("admin", &["manage"], Some(&["nonexistent"]))
    .unwrap()
    .build();

  assert!(result.is_err());
  match result {
    Err(RbacError::InvalidConfiguration(msg)) => {
      assert!(msg.contains("nonexistent"));
    }
    other => panic!("Expected InvalidConfiguration, got {:?}", other),
  }
}

#[test]
fn test_empty_rbac() -> Result<(), RbacError> {
  let rbac = RbacBuilder::new().build()?;
  assert_eq!(rbac.role_count(), 0);
  assert!(!rbac.has_role("anything"));
  assert!(!rbac.is_granted("anything", "anything"));
  Ok(())
}

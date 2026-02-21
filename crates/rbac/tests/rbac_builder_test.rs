use std::convert::TryFrom;
use std::fs::File;
use walrs_rbac::{RbacBuilder, RbacData, RbacError};

#[test]
fn test_builder_from_rbac_data() -> Result<(), RbacError> {
  let data = RbacData {
    roles: vec![
      ("guest".to_string(), vec!["read".to_string()], None),
      (
        "admin".to_string(),
        vec!["manage".to_string()],
        Some(vec!["guest".to_string()]),
      ),
    ],
  };

  let rbac = RbacBuilder::try_from(&data)?.build()?;
  assert!(rbac.is_granted("admin", "manage"));
  assert!(rbac.is_granted("admin", "read"));
  assert!(!rbac.is_granted("guest", "manage"));

  Ok(())
}

#[test]
fn test_builder_from_json_file() -> Result<(), Box<dyn std::error::Error>> {
  let file_path = "./test-fixtures/example-rbac.json";
  let mut f = File::open(file_path)?;
  let rbac = RbacBuilder::try_from(&mut f)?.build()?;

  assert!(rbac.has_role("guest"));
  assert!(rbac.has_role("user"));
  assert!(rbac.has_role("editor"));
  assert!(rbac.has_role("admin"));

  assert!(rbac.is_granted("guest", "read.public"));
  assert!(rbac.is_granted("admin", "admin.panel"));
  assert!(rbac.is_granted("admin", "read.public"));

  Ok(())
}

#[test]
fn test_rbac_data_json_roundtrip() -> Result<(), RbacError> {
  let data = RbacData {
    roles: vec![
      ("guest".to_string(), vec!["read".to_string()], None),
      (
        "admin".to_string(),
        vec!["manage".to_string()],
        Some(vec!["guest".to_string()]),
      ),
    ],
  };

  let json = data.to_json()?;
  let restored = RbacData::from_json(&json)?;
  assert_eq!(data.roles.len(), restored.roles.len());

  // Ensure we can build from restored data
  let rbac = RbacBuilder::try_from(&restored)?.build()?;
  assert!(rbac.is_granted("admin", "read"));

  Ok(())
}

#[test]
fn test_rebuild_from_rbac() -> Result<(), RbacError> {
  let original = RbacBuilder::new()
    .add_role("guest", &["read"], None)?
    .add_role("user", &["write"], Some(&["guest"]))?
    .build()?;

  // Convert back to builder, add more, and rebuild
  let modified = RbacBuilder::try_from(&original)?
    .add_role("admin", &["manage"], Some(&["user"]))?
    .build()?;

  assert!(modified.is_granted("admin", "read"));
  assert!(modified.is_granted("admin", "write"));
  assert!(modified.is_granted("admin", "manage"));

  // Original should still be available
  assert!(original.is_granted("user", "read"));
  assert!(!original.has_role("admin"));

  Ok(())
}

//! Example demonstrating loading RBAC from a JSON file.
//!
//! Run with: `cargo run --example rbac_try_from_json`

use std::convert::TryFrom;
use std::fs::File;
use walrs_rbac::RbacBuilder;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
  println!("=== RBAC from JSON Example ===\n");

  let file_path = "./test-fixtures/example-rbac.json";
  let mut f = File::open(file_path)?;
  let rbac = RbacBuilder::try_from(&mut f)?.build()?;

  println!("✓ RBAC loaded from JSON successfully!\n");

  // Test permissions
  println!("Permissions:");
  println!("  guest -> read.public: {}", rbac.is_granted("guest", "read.public"));
  println!("  user -> write.post: {}", rbac.is_granted("user", "write.post"));
  println!("  user -> read.public: {}", rbac.is_granted("user", "read.public"));
  println!("  editor -> edit.post: {}", rbac.is_granted("editor", "edit.post"));
  println!("  admin -> admin.panel: {}", rbac.is_granted("admin", "admin.panel"));
  println!("  admin -> read.public: {}", rbac.is_granted("admin", "read.public"));
  println!("  guest -> admin.panel: {}", rbac.is_granted("guest", "admin.panel"));
  println!();

  println!("Total roles: {}", rbac.role_count());

  Ok(())
}

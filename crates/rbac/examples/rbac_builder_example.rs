//! Example demonstrating the use of RbacBuilder to construct an RBAC with a fluent interface.
//!
//! Run with: `cargo run --example rbac_builder_example`

use walrs_rbac::RbacBuilder;

fn main() -> std::result::Result<(), walrs_rbac::RbacError> {
  println!("=== RBAC Builder Example ===\n");

  // Build an RBAC using the fluent builder interface
  let rbac = RbacBuilder::new()
    // Add roles with permissions and children
    .add_role("guest", &["read.public"], None)?
    .add_role("user", &["write.post", "comment.post"], Some(&["guest"]))?
    .add_role("editor", &["edit.post", "publish.post"], Some(&["user"]))?
    .add_role("admin", &["admin.panel", "manage.users"], Some(&["editor"]))?
    .build()?;

  println!("✓ RBAC built successfully!\n");

  // Test guest permissions
  println!("Guest permissions:");
  println!("  read.public: {}", rbac.is_granted("guest", "read.public"));
  println!("  write.post: {}", rbac.is_granted("guest", "write.post"));
  println!();

  // Test user permissions (inherits from guest)
  println!("User permissions (inherits from guest):");
  println!("  read.public: {}", rbac.is_granted("user", "read.public"));
  println!("  write.post: {}", rbac.is_granted("user", "write.post"));
  println!(
    "  comment.post: {}",
    rbac.is_granted("user", "comment.post")
  );
  println!("  edit.post: {}", rbac.is_granted("user", "edit.post"));
  println!();

  // Test editor permissions (inherits from user)
  println!("Editor permissions (inherits from user):");
  println!(
    "  read.public: {}",
    rbac.is_granted("editor", "read.public")
  );
  println!("  write.post: {}", rbac.is_granted("editor", "write.post"));
  println!("  edit.post: {}", rbac.is_granted("editor", "edit.post"));
  println!(
    "  publish.post: {}",
    rbac.is_granted("editor", "publish.post")
  );
  println!(
    "  admin.panel: {}",
    rbac.is_granted("editor", "admin.panel")
  );
  println!();

  // Test admin permissions (inherits from editor)
  println!("Admin permissions (inherits from editor):");
  println!("  read.public: {}", rbac.is_granted("admin", "read.public"));
  println!("  edit.post: {}", rbac.is_granted("admin", "edit.post"));
  println!("  admin.panel: {}", rbac.is_granted("admin", "admin.panel"));
  println!(
    "  manage.users: {}",
    rbac.is_granted("admin", "manage.users")
  );
  println!();

  // Show counts
  println!("RBAC statistics:");
  println!("  Total roles: {}", rbac.role_count());

  Ok(())
}

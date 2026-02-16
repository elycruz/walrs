/// Example demonstrating WASM-compatible ACL usage (no file I/O)
///
/// This example works with both `std` and `no_std` + `alloc` builds.
///
/// To run with std:
/// ```bash
/// cargo run --example wasm_example
/// ```
///
/// To verify WASM compatibility:
/// ```bash
/// cargo build --example wasm_example --no-default-features --features wasm
/// ```
use walrs_acl::simple::{AclBuilder, AclData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("=== WASM-Compatible ACL Example ===\n");

  // Example 1: Build ACL programmatically
  println!("1. Building ACL programmatically:");
  let acl = AclBuilder::new()
    .add_role("guest", None)?
    .add_role("user", Some(&["guest"]))?
    .add_role("admin", Some(&["user"]))?
    .add_resource("blog", None)?
    .add_resource("admin_panel", None)?
    .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
    .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?
    .allow(Some(&["admin"]), None, None)?
    .deny(Some(&["user"]), Some(&["admin_panel"]), None)?
    .build()?;

  println!("   ✓ ACL built successfully");
  println!(
    "   - Guest can read blog: {}",
    acl.is_allowed(Some("guest"), Some("blog"), Some("read"))
  );
  println!(
    "   - User can write blog: {}",
    acl.is_allowed(Some("user"), Some("blog"), Some("write"))
  );
  println!(
    "   - User can access admin_panel: {}",
    acl.is_allowed(Some("user"), Some("admin_panel"), Some("read"))
  );
  println!(
    "   - Admin can access admin_panel: {}",
    acl.is_allowed(Some("admin"), Some("admin_panel"), Some("read"))
  );

  // Example 2: Load from JSON string (WASM-compatible)
  println!("\n2. Loading ACL from JSON string:");
  let json_str = r#"{
  "roles": [
    ["guest", null],
    ["user", ["guest"]],
    ["moderator", ["user"]]
  ],
  "resources": [
    ["index", null],
    ["blog", ["index"]],
    ["comments", ["blog"]]
  ],
  "allow": [
    ["index", [
      ["guest", ["read"]]
    ]],
    ["blog", [
      ["user", ["read", "write"]],
      ["moderator", ["read", "write", "publish"]]
    ]],
    ["comments", [
      ["moderator", ["delete"]]
    ]]
  ],
  "deny": [
    ["blog", [
      ["user", ["publish"]]
    ]]
  ]
}"#;

  let acl_data: AclData = serde_json::from_str(json_str)?;
  let acl = AclBuilder::try_from(&acl_data)?.build()?;

  println!("   ✓ ACL loaded from JSON successfully");
  println!(
    "   - Guest can read index: {}",
    acl.is_allowed(Some("guest"), Some("index"), Some("read"))
  );
  println!(
    "   - User can write blog: {}",
    acl.is_allowed(Some("user"), Some("blog"), Some("write"))
  );
  println!(
    "   - User can publish blog: {}",
    acl.is_allowed(Some("user"), Some("blog"), Some("publish"))
  );
  println!(
    "   - Moderator can publish blog: {}",
    acl.is_allowed(Some("moderator"), Some("blog"), Some("publish"))
  );
  println!(
    "   - Moderator can delete comments: {}",
    acl.is_allowed(Some("moderator"), Some("comments"), Some("delete"))
  );

  // Example 3: Convert ACL back to builder and modify
  println!("\n3. Converting ACL to builder and modifying:");
  let modified_acl = AclBuilder::try_from(&acl)?
    .add_role("super_admin", Some(&["admin"]))?
    .allow(Some(&["super_admin"]), None, None)?
    .build()?;

  println!("   ✓ ACL modified successfully");
  println!(
    "   - Super admin can do anything: {}",
    modified_acl.is_allowed(Some("super_admin"), Some("blog"), Some("delete"))
  );

  println!("\n=== All examples completed successfully! ===");
  println!("\nNote: This example works in WASM because it doesn't use file I/O.");
  println!("To use in a WASM environment, pass JSON strings from JavaScript.");

  Ok(())
}

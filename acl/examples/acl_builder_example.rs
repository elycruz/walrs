/// Example demonstrating the use of AclBuilder to construct an ACL with a fluent interface.
///
/// Run with: `cargo run --example acl_builder_example`

use walrs_acl::simple::AclBuilder;

fn main() -> Result<(), String> {
    println!("=== ACL Builder Example ===\n");

    // Build an ACL using the fluent builder interface
    let acl = AclBuilder::new()
        // Add roles with inheritance
        .add_role("guest", None)?
        .add_role("user", Some(&["guest"]))?
        .add_role("editor", Some(&["user"]))?
        .add_role("admin", Some(&["editor"]))?

        // Add resources
        .add_resource("public", None)?
        .add_resource("blog", None)?
        .add_resource("admin_panel", None)?

        // Set allow rules
        .allow(Some(&["guest"]), Some(&["public"]), Some(&["read"]))?
        .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "comment"]))?
        .allow(Some(&["editor"]), Some(&["blog"]), Some(&["write", "edit"]))?
        .allow(Some(&["admin"]), None, None)? // Admin has all privileges on all resources

        // Set deny rules
        .deny(Some(&["editor"]), Some(&["admin_panel"]), None)?

        // Build the final ACL
        .build()?;

    println!("✓ ACL built successfully!\n");

    // Test guest permissions
    println!("Guest permissions:");
    println!("  Can read public: {}", acl.is_allowed(Some("guest"), Some("public"), Some("read")));
    println!("  Can read blog: {}", acl.is_allowed(Some("guest"), Some("blog"), Some("read")));
    println!("  Can write blog: {}", acl.is_allowed(Some("guest"), Some("blog"), Some("write")));
    println!();

    // Test user permissions (inherits from guest)
    println!("User permissions (inherits from guest):");
    println!("  Can read public: {}", acl.is_allowed(Some("user"), Some("public"), Some("read")));
    println!("  Can read blog: {}", acl.is_allowed(Some("user"), Some("blog"), Some("read")));
    println!("  Can comment on blog: {}", acl.is_allowed(Some("user"), Some("blog"), Some("comment")));
    println!("  Can write blog: {}", acl.is_allowed(Some("user"), Some("blog"), Some("write")));
    println!();

    // Test editor permissions (inherits from user)
    println!("Editor permissions (inherits from user):");
    println!("  Can read blog: {}", acl.is_allowed(Some("editor"), Some("blog"), Some("read")));
    println!("  Can write blog: {}", acl.is_allowed(Some("editor"), Some("blog"), Some("write")));
    println!("  Can edit blog: {}", acl.is_allowed(Some("editor"), Some("blog"), Some("edit")));
    println!("  Can access admin panel: {}", acl.is_allowed(Some("editor"), Some("admin_panel"), Some("read")));
    println!();

    // Test admin permissions (has all privileges)
    println!("Admin permissions (has all privileges):");
    println!("  Can read blog: {}", acl.is_allowed(Some("admin"), Some("blog"), Some("read")));
    println!("  Can delete blog: {}", acl.is_allowed(Some("admin"), Some("blog"), Some("delete")));
    println!("  Can access admin panel: {}", acl.is_allowed(Some("admin"), Some("admin_panel"), Some("read")));
    println!("  Can manage admin panel: {}", acl.is_allowed(Some("admin"), Some("admin_panel"), Some("manage")));
    println!();

    // Test role inheritance
    println!("Role inheritance:");
    println!("  user inherits from guest: {}", acl.inherits_role("user", "guest"));
    println!("  editor inherits from user: {}", acl.inherits_role("editor", "user"));
    println!("  admin inherits from editor: {}", acl.inherits_role("admin", "editor"));
    println!("  admin inherits from guest (transitive): {}", acl.inherits_role("admin", "guest"));
    println!();

    // Show counts
    println!("ACL statistics:");
    println!("  Total roles: {}", acl.role_count());
    println!("  Total resources: {}", acl.resource_count());

    Ok(())
}

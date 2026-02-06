use walrs_acl::simple::AclBuilder;

#[test]
fn test_acl_builder_basic() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("guest", None)?
        .add_role("user", Some(&["guest"]))?
        .add_role("admin", Some(&["user"]))?
        .add_resource("blog", None)?
        .add_resource("admin_panel", None)?
        .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
        .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?
        .allow(Some(&["admin"]), None, None)?
        .build()?;

    // Test guest access
    assert!(
        acl.is_allowed(Some("guest"), Some("blog"), Some("read")),
        "Guest should be able to read blog"
    );
    assert!(
        !acl.is_allowed(Some("guest"), Some("blog"), Some("write")),
        "Guest should not be able to write to blog"
    );
    assert!(
        !acl.is_allowed(Some("guest"), Some("admin_panel"), Some("read")),
        "Guest should not have access to admin panel"
    );

    // Test user access (inherits from guest)
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("read")),
        "User should be able to read blog"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("write")),
        "User should be able to write to blog"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("admin_panel"), Some("read")),
        "User should not have access to admin panel"
    );

    // Test admin access (should have all privileges)
    assert!(
        acl.is_allowed(Some("admin"), Some("blog"), Some("read")),
        "Admin should be able to read blog"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("blog"), Some("write")),
        "Admin should be able to write to blog"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("admin_panel"), Some("read")),
        "Admin should have access to admin panel"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("admin_panel"), Some("delete")),
        "Admin should have all privileges on admin panel"
    );

    Ok(())
}

#[test]
fn test_acl_builder_try_from_acl() -> Result<(), String> {
    use std::convert::TryFrom;

    // First, create an ACL using the builder
    let original_acl = AclBuilder::new()
        .add_role("guest", None)?
        .add_role("user", Some(&["guest"]))?
        .add_role("admin", Some(&["user"]))?
        .add_resource("blog", None)?
        .add_resource("comment", Some(&["blog"]))?
        .add_resource("admin_panel", None)?
        .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
        .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?
        .allow(Some(&["user"]), Some(&["comment"]), Some(&["create"]))?
        .deny(Some(&["user"]), Some(&["admin_panel"]), None)?
        .allow(Some(&["admin"]), None, None)?
        .build()?;

    // Convert the ACL back to a builder
    let builder = AclBuilder::try_from(original_acl)?;

    // Build a new ACL from the builder
    let rebuilt_acl = builder.build()?;

    // Verify that the rebuilt ACL has the same behavior as the original

    // Test guest permissions
    assert!(
        rebuilt_acl.is_allowed(Some("guest"), Some("blog"), Some("read")),
        "Guest should be able to read blog"
    );
    assert!(
        !rebuilt_acl.is_allowed(Some("guest"), Some("blog"), Some("write")),
        "Guest should not be able to write to blog"
    );

    // Test user permissions (inherits from guest)
    assert!(
        rebuilt_acl.is_allowed(Some("user"), Some("blog"), Some("read")),
        "User should be able to read blog"
    );
    assert!(
        rebuilt_acl.is_allowed(Some("user"), Some("blog"), Some("write")),
        "User should be able to write to blog"
    );
    assert!(
        rebuilt_acl.is_allowed(Some("user"), Some("comment"), Some("create")),
        "User should be able to create comments"
    );
    assert!(
        !rebuilt_acl.is_allowed(Some("user"), Some("admin_panel"), Some("read")),
        "User should not have access to admin panel"
    );

    // Test admin permissions (should have all privileges)
    assert!(
        rebuilt_acl.is_allowed(Some("admin"), Some("blog"), Some("read")),
        "Admin should be able to read blog"
    );
    assert!(
        rebuilt_acl.is_allowed(Some("admin"), Some("admin_panel"), Some("read")),
        "Admin should have access to admin panel"
    );
    assert!(
        rebuilt_acl.is_allowed(Some("admin"), Some("comment"), Some("delete")),
        "Admin should be able to delete comments"
    );

    // Verify role inheritance
    assert!(
        rebuilt_acl.inherits_role("user", "guest"),
        "User role should inherit from guest"
    );
    assert!(
        rebuilt_acl.inherits_role("admin", "user"),
        "Admin role should inherit from user"
    );
    assert!(
        rebuilt_acl.inherits_role("admin", "guest"),
        "Admin role should transitively inherit from guest"
    );

    // Verify resource inheritance
    assert!(
        rebuilt_acl.inherits_resource("comment", "blog"),
        "Comment resource should inherit from blog"
    );

    // Verify role and resource counts
    assert_eq!(rebuilt_acl.role_count(), 3, "Should have 3 roles");
    assert_eq!(rebuilt_acl.resource_count(), 3, "Should have 3 resources");

    Ok(())
}

#[test]
fn test_acl_builder_with_deny_rules() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_role("admin", None)?
        .add_resource("public", None)?
        .add_resource("secret", None)?
        .allow(Some(&["user"]), Some(&["public"]), Some(&["read"]))?
        .deny(Some(&["user"]), Some(&["secret"]), None)?
        .allow(Some(&["admin"]), None, None)?
        .build()?;

    // User can read public
    assert!(
        acl.is_allowed(Some("user"), Some("public"), Some("read")),
        "User should be able to read public resource"
    );

    // User is explicitly denied access to secret
    assert!(
        !acl.is_allowed(Some("user"), Some("secret"), Some("read")),
        "User should be denied access to secret resource"
    );

    // Admin has access to everything
    assert!(
        acl.is_allowed(Some("admin"), Some("public"), Some("read")),
        "Admin should have access to public"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("secret"), Some("read")),
        "Admin should have access to secret"
    );

    Ok(())
}

#[test]
fn test_acl_builder_add_roles_batch() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_roles(&[
            ("guest", None),
            ("user", Some(&["guest"])),
            ("admin", Some(&["user"])),
        ])?
        .add_resource("resource", None)?
        .allow(Some(&["admin"]), Some(&["resource"]), Some(&["manage"]))?
        .build()?;

    // Check roles exist
    assert!(acl.has_role("guest"), "Should have guest role");
    assert!(acl.has_role("user"), "Should have user role");
    assert!(acl.has_role("admin"), "Should have admin role");

    // Check inheritance
    assert!(
        acl.inherits_role("user", "guest"),
        "User should inherit from guest"
    );
    assert!(
        acl.inherits_role("admin", "user"),
        "Admin should inherit from user"
    );
    assert!(
        acl.inherits_role("admin", "guest"),
        "Admin should inherit from guest (transitive)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_add_resources_batch() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resources(&[
            ("content", None),
            ("blog", Some(&["content"])),
            ("post", Some(&["blog"])),
        ])?
        .allow(Some(&["user"]), Some(&["post"]), Some(&["read"]))?
        .build()?;

    // Check resources exist
    assert!(acl.has_resource("content"), "Should have content resource");
    assert!(acl.has_resource("blog"), "Should have blog resource");
    assert!(acl.has_resource("post"), "Should have post resource");

    // Check inheritance
    assert!(
        acl.inherits_resource("blog", "content"),
        "Blog should inherit from content"
    );
    assert!(
        acl.inherits_resource("post", "blog"),
        "Post should inherit from blog"
    );
    assert!(
        acl.inherits_resource("post", "content"),
        "Post should inherit from content (transitive)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_with_all_resources() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("admin", None)?
        .add_resource("resource1", None)?
        .add_resource("resource2", None)?
        .allow(Some(&["admin"]), None, None)? // All resources, all privileges
        .build()?;

    // Admin should have access to all resources
    assert!(
        acl.is_allowed(Some("admin"), Some("resource1"), Some("read")),
        "Admin should have read access to resource1"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("resource2"), Some("write")),
        "Admin should have write access to resource2"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("resource1"), None),
        "Admin should have all privileges on resource1"
    );

    Ok(())
}

#[test]
fn test_acl_builder_with_all_privileges() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("blog", None)?
        .allow(Some(&["user"]), Some(&["blog"]), None)? // All privileges on blog
        .build()?;

    // User should have all privileges on blog
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("read")),
        "User should have read privilege"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("write")),
        "User should have write privilege"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("delete")),
        "User should have delete privilege"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), None),
        "User should have all privileges"
    );

    Ok(())
}

#[test]
fn test_acl_builder_empty() -> Result<(), String> {
    let acl = AclBuilder::new().build()?;

    // Empty ACL should deny everything
    assert!(
        !acl.is_allowed(Some("guest"), Some("resource"), Some("read")),
        "Empty ACL should deny all access"
    );

    Ok(())
}

#[test]
fn test_acl_builder_default() {
    let builder = AclBuilder::default();
    let acl = builder.build();

    assert!(acl.is_ok(), "Default builder should build successfully");
}

#[test]
fn test_acl_builder_role_inheritance() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("guest", None)?
        .add_role("user", Some(&["guest"]))?
        .add_role("moderator", Some(&["user"]))?
        .add_role("admin", Some(&["moderator"]))?
        .add_resource("content", None)?
        .allow(Some(&["guest"]), Some(&["content"]), Some(&["read"]))?
        .allow(Some(&["user"]), Some(&["content"]), Some(&["comment"]))?
        .allow(Some(&["moderator"]), Some(&["content"]), Some(&["edit"]))?
        .allow(Some(&["admin"]), Some(&["content"]), Some(&["delete"]))?
        .build()?;

    // Admin should have all inherited privileges
    assert!(
        acl.is_allowed(Some("admin"), Some("content"), Some("read")),
        "Admin should inherit read from guest"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("content"), Some("comment")),
        "Admin should inherit comment from user"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("content"), Some("edit")),
        "Admin should inherit edit from moderator"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("content"), Some("delete")),
        "Admin should have own delete privilege"
    );

    // Moderator should have guest and user privileges but not admin
    assert!(
        acl.is_allowed(Some("moderator"), Some("content"), Some("read")),
        "Moderator should inherit read from guest"
    );
    assert!(
        acl.is_allowed(Some("moderator"), Some("content"), Some("comment")),
        "Moderator should inherit comment from user"
    );
    assert!(
        acl.is_allowed(Some("moderator"), Some("content"), Some("edit")),
        "Moderator should have own edit privilege"
    );
    assert!(
        !acl.is_allowed(Some("moderator"), Some("content"), Some("delete")),
        "Moderator should not have admin's delete privilege"
    );

    Ok(())
}

#[test]
fn test_acl_builder_resource_inheritance() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("content", None)?
        .add_resource("article", Some(&["content"]))?
        .add_resource("blog_post", Some(&["article"]))?
        .allow(Some(&["user"]), Some(&["content"]), Some(&["read"]))?
        .build()?;

    // User should have read access to all resources through inheritance
    assert!(
        acl.is_allowed(Some("user"), Some("content"), Some("read")),
        "User should have read access to content"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("article"), Some("read")),
        "User should have read access to article (inherits from content)"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("blog_post"), Some("read")),
        "User should have read access to blog_post (inherits from article)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_complex_scenario() -> Result<(), String> {
    let acl = AclBuilder::new()
        // Roles
        .add_roles(&[
            ("guest", None),
            ("user", Some(&["guest"])),
            ("editor", Some(&["user"])),
            ("admin", Some(&["editor"])),
        ])?
        // Resources
        .add_resources(&[
            ("public", None),
            ("private", None),
            ("admin_area", None),
        ])?
        // Rules
        .allow(Some(&["guest"]), Some(&["public"]), Some(&["read"]))?
        .allow(Some(&["user"]), Some(&["public"]), Some(&["comment"]))?
        .allow(Some(&["user"]), Some(&["private"]), Some(&["read"]))?
        .allow(Some(&["editor"]), Some(&["public"]), Some(&["write", "edit"]))?
        .allow(Some(&["editor"]), Some(&["private"]), Some(&["write", "edit"]))?
        .deny(Some(&["editor"]), Some(&["admin_area"]), None)?
        .allow(Some(&["admin"]), None, None)?
        .build()?;

    // Guest tests
    assert!(acl.is_allowed(Some("guest"), Some("public"), Some("read")));
    assert!(!acl.is_allowed(Some("guest"), Some("public"), Some("write")));
    assert!(!acl.is_allowed(Some("guest"), Some("private"), Some("read")));

    // User tests (inherits from guest)
    assert!(acl.is_allowed(Some("user"), Some("public"), Some("read")));
    assert!(acl.is_allowed(Some("user"), Some("public"), Some("comment")));
    assert!(acl.is_allowed(Some("user"), Some("private"), Some("read")));
    assert!(!acl.is_allowed(Some("user"), Some("private"), Some("write")));

    // Editor tests (inherits from user)
    assert!(acl.is_allowed(Some("editor"), Some("public"), Some("read")));
    assert!(acl.is_allowed(Some("editor"), Some("public"), Some("write")));
    assert!(acl.is_allowed(Some("editor"), Some("public"), Some("edit")));
    assert!(acl.is_allowed(Some("editor"), Some("private"), Some("write")));
    assert!(!acl.is_allowed(Some("editor"), Some("admin_area"), Some("read")));

    // Admin tests (has all privileges)
    assert!(acl.is_allowed(Some("admin"), Some("public"), Some("read")));
    assert!(acl.is_allowed(Some("admin"), Some("private"), Some("write")));
    assert!(acl.is_allowed(Some("admin"), Some("admin_area"), Some("read")));
    assert!(acl.is_allowed(Some("admin"), Some("admin_area"), Some("delete")));

    Ok(())
}

// ============================
// Tests for add_role
// ============================

#[test]
fn test_acl_builder_add_role_without_parents() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("guest", None)?
        .add_resource("blog", None)?
        .build()?;

    assert!(acl.has_role("guest"), "Should have guest role");
    assert_eq!(acl.role_count(), 1, "Should have exactly 1 role");

    Ok(())
}

#[test]
fn test_acl_builder_add_role_with_single_parent() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("guest", None)?
        .add_role("user", Some(&["guest"]))?
        .build()?;

    assert!(acl.has_role("guest"), "Should have guest role");
    assert!(acl.has_role("user"), "Should have user role");
    assert!(acl.inherits_role("user", "guest"), "User should inherit from guest");
    assert_eq!(acl.role_count(), 2, "Should have exactly 2 roles");

    Ok(())
}

#[test]
fn test_acl_builder_add_role_with_multiple_parents() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("viewer", None)?
        .add_role("editor", None)?
        .add_role("admin", Some(&["viewer", "editor"]))?
        .build()?;

    assert!(acl.has_role("viewer"), "Should have viewer role");
    assert!(acl.has_role("editor"), "Should have editor role");
    assert!(acl.has_role("admin"), "Should have admin role");
    assert!(acl.inherits_role("admin", "viewer"), "Admin should inherit from viewer");
    assert!(acl.inherits_role("admin", "editor"), "Admin should inherit from editor");
    assert_eq!(acl.role_count(), 3, "Should have exactly 3 roles");

    Ok(())
}

#[test]
fn test_acl_builder_add_role_with_nonexistent_parent() -> Result<(), String> {
    // Parent role doesn't exist yet - it should be auto-created
    let acl = AclBuilder::new()
        .add_role("user", Some(&["nonexistent-parent"]))?
        .build()?;

    assert!(acl.has_role("user"), "Should have user role");
    assert!(acl.has_role("nonexistent-parent"), "Should auto-create nonexistent-parent role");
    assert!(acl.inherits_role("user", "nonexistent-parent"), "User should inherit from nonexistent-parent");
    assert_eq!(acl.role_count(), 2, "Should have exactly 2 roles");

    Ok(())
}

#[test]
fn test_acl_builder_add_role_duplicate() -> Result<(), String> {
    // Adding same role twice should be idempotent
    let acl = AclBuilder::new()
        .add_role("guest", None)?
        .add_role("guest", None)?
        .build()?;

    assert!(acl.has_role("guest"), "Should have guest role");
    assert_eq!(acl.role_count(), 1, "Should have exactly 1 role (not duplicated)");

    Ok(())
}

#[test]
fn test_acl_builder_add_role_chaining() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_role("guest", None)?
        .add_role("user", Some(&["guest"]))?
        .add_role("admin", Some(&["user"]))?
        .add_role("super_admin", Some(&["admin"]))?
        .build()?;

    // Verify all roles exist
    assert_eq!(acl.role_count(), 4, "Should have exactly 4 roles");

    // Verify inheritance chain
    assert!(acl.inherits_role("user", "guest"), "User should inherit from guest");
    assert!(acl.inherits_role("admin", "user"), "Admin should inherit from user");
    assert!(acl.inherits_role("super_admin", "admin"), "Super admin should inherit from admin");

    // Verify transitive inheritance
    assert!(acl.inherits_role("super_admin", "guest"), "Super admin should transitively inherit from guest");

    Ok(())
}

// ============================
// Tests for add_roles
// ============================

#[test]
fn test_acl_builder_add_roles_basic() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_roles(&[
            ("guest", None),
            ("user", None),
            ("admin", None),
        ])?
        .build()?;

    assert!(acl.has_role("guest"), "Should have guest role");
    assert!(acl.has_role("user"), "Should have user role");
    assert!(acl.has_role("admin"), "Should have admin role");
    assert_eq!(acl.role_count(), 3, "Should have exactly 3 roles");

    Ok(())
}

#[test]
fn test_acl_builder_add_roles_with_single_parent() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_roles(&[
            ("guest", None),
            ("user", Some(&["guest"])),
            ("admin", Some(&["user"])),
        ])?
        .build()?;

    assert!(acl.inherits_role("user", "guest"), "User should inherit from guest");
    assert!(acl.inherits_role("admin", "user"), "Admin should inherit from user");
    assert!(acl.inherits_role("admin", "guest"), "Admin should transitively inherit from guest");
    assert_eq!(acl.role_count(), 3);

    Ok(())
}

#[test]
fn test_acl_builder_add_roles_with_multiple_parents() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_roles(&[
            ("viewer", None),
            ("editor", None),
            ("moderator", None),
            ("admin", Some(&["editor", "moderator"])),
            ("super_admin", Some(&["admin", "viewer"])),
        ])?
        .build()?;

    assert_eq!(acl.role_count(), 5, "Should have exactly 5 roles");
    assert!(acl.inherits_role("admin", "editor"), "Admin should inherit from editor");
    assert!(acl.inherits_role("admin", "moderator"), "Admin should inherit from moderator");
    assert!(acl.inherits_role("super_admin", "admin"), "Super admin should inherit from admin");
    assert!(acl.inherits_role("super_admin", "viewer"), "Super admin should inherit from viewer");

    // Verify transitive inheritance
    assert!(acl.inherits_role("super_admin", "editor"), "Super admin should transitively inherit from editor");
    assert!(acl.inherits_role("super_admin", "moderator"), "Super admin should transitively inherit from moderator");

    Ok(())
}

#[test]
fn test_acl_builder_add_roles_empty_list() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_roles(&[])?
        .build()?;

    assert_eq!(acl.role_count(), 0, "Should have 0 roles");

    Ok(())
}

#[test]
fn test_acl_builder_add_roles_out_of_order_dependencies() -> Result<(), String> {
    // Add roles in reverse order - children before parents
    let acl = AclBuilder::new()
        .add_roles(&[
            ("super_admin", Some(&["admin"])),  // admin doesn't exist yet
            ("admin", Some(&["user"])),         // user doesn't exist yet
            ("user", Some(&["guest"])),         // guest doesn't exist yet
            ("guest", None),                    // finally add the base role
        ])?
        .build()?;

    assert_eq!(acl.role_count(), 4, "Should have exactly 4 roles");
    assert!(acl.inherits_role("user", "guest"), "User should inherit from guest");
    assert!(acl.inherits_role("admin", "user"), "Admin should inherit from user");
    assert!(acl.inherits_role("super_admin", "admin"), "Super admin should inherit from admin");
    assert!(acl.inherits_role("super_admin", "guest"), "Super admin should transitively inherit from guest");

    Ok(())
}

#[test]
fn test_acl_builder_add_roles_duplicate_roles() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_roles(&[
            ("guest", None),
            ("user", Some(&["guest"])),
        ])?
        .add_roles(&[
            ("guest", None),  // Duplicate
            ("user", Some(&["guest"])),  // Duplicate
        ])?
        .build()?;

    assert!(acl.has_role("guest"), "Should have guest role");
    assert!(acl.has_role("user"), "Should have user role");
    assert_eq!(acl.role_count(), 2, "Should have exactly 2 roles");

    Ok(())
}

#[test]
fn test_acl_builder_add_roles_mixed_with_and_without_parents() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_roles(&[
            ("guest", None),
            ("special", None),
            ("user", Some(&["guest"])),
            ("moderator", None),
            ("admin", Some(&["user", "moderator"])),
        ])?
        .build()?;

    assert_eq!(acl.role_count(), 5, "Should have exactly 5 roles");
    assert!(acl.inherits_role("user", "guest"), "User should inherit from guest");
    assert!(acl.inherits_role("admin", "user"), "Admin should inherit from user");
    assert!(acl.inherits_role("admin", "moderator"), "Admin should inherit from moderator");
    assert!(!acl.inherits_role("special", "guest"), "Special should not inherit from guest");

    Ok(())
}

#[test]
fn test_acl_builder_add_roles_complex_diamond_inheritance() -> Result<(), String> {
    // Create a diamond inheritance pattern:
    //        root
    //       /    \
    //   branch_a  branch_b
    //       \    /
    //        leaf
    let acl = AclBuilder::new()
        .add_roles(&[
            ("root", None),
            ("branch_a", Some(&["root"])),
            ("branch_b", Some(&["root"])),
            ("leaf", Some(&["branch_a", "branch_b"])),
        ])?
        .build()?;

    assert_eq!(acl.role_count(), 4, "Should have exactly 4 roles");
    assert!(acl.inherits_role("branch_a", "root"), "Branch A should inherit from root");
    assert!(acl.inherits_role("branch_b", "root"), "Branch B should inherit from root");
    assert!(acl.inherits_role("leaf", "branch_a"), "Leaf should inherit from branch A");
    assert!(acl.inherits_role("leaf", "branch_b"), "Leaf should inherit from branch B");
    assert!(acl.inherits_role("leaf", "root"), "Leaf should transitively inherit from root");

    Ok(())
}

// ============================
// Tests for add_resource
// ============================

#[test]
fn test_acl_builder_add_resource_without_parents() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resource("blog", None)?
        .build()?;

    assert!(acl.has_resource("blog"), "Should have blog resource");
    assert_eq!(acl.resource_count(), 1, "Should have exactly 1 resource");

    Ok(())
}

#[test]
fn test_acl_builder_add_resource_with_single_parent() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resource("cms", None)?
        .add_resource("blog", Some(&["cms"]))?
        .build()?;

    assert!(acl.has_resource("cms"), "Should have cms resource");
    assert!(acl.has_resource("blog"), "Should have blog resource");
    assert_eq!(acl.resource_count(), 2, "Should have exactly 2 resources");
    assert!(acl.inherits_resource("blog", "cms"), "Blog should inherit from cms");
    assert!(!acl.inherits_resource("cms", "blog"), "CMS should not inherit from blog");

    Ok(())
}

#[test]
fn test_acl_builder_add_resource_with_multiple_parents() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resource("readable", None)?
        .add_resource("writable", None)?
        .add_resource("document", Some(&["readable", "writable"]))?
        .build()?;

    assert_eq!(acl.resource_count(), 3, "Should have exactly 3 resources");
    assert!(acl.inherits_resource("document", "readable"), "Document should inherit from readable");
    assert!(acl.inherits_resource("document", "writable"), "Document should inherit from writable");

    Ok(())
}

#[test]
fn test_acl_builder_add_resource_with_transitive_inheritance() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resource("base", None)?
        .add_resource("intermediate", Some(&["base"]))?
        .add_resource("leaf", Some(&["intermediate"]))?
        .build()?;

    assert!(acl.inherits_resource("leaf", "intermediate"), "Leaf should inherit from intermediate");
    assert!(acl.inherits_resource("leaf", "base"), "Leaf should inherit from base (transitively)");
    assert!(acl.inherits_resource("intermediate", "base"), "Intermediate should inherit from base");

    Ok(())
}

#[test]
fn test_acl_builder_add_resource_chained_calls() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resource("level1", None)?
        .add_resource("level2", Some(&["level1"]))?
        .add_resource("level3", Some(&["level2"]))?
        .add_resource("level4", Some(&["level3"]))?
        .build()?;

    assert_eq!(acl.resource_count(), 4, "Should have exactly 4 resources");
    assert!(acl.inherits_resource("level4", "level3"), "Level4 should inherit from level3");
    assert!(acl.inherits_resource("level4", "level2"), "Level4 should inherit from level2 (transitively)");
    assert!(acl.inherits_resource("level4", "level1"), "Level4 should inherit from level1 (transitively)");

    Ok(())
}

#[test]
fn test_acl_builder_add_resource_duplicate() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resource("blog", None)?
        .add_resource("blog", None)?
        .build()?;

    assert_eq!(acl.resource_count(), 1, "Should have exactly 1 resource (not duplicated)");

    Ok(())
}

// ============================
// Tests for add_resources
// ============================

#[test]
fn test_acl_builder_add_resources_basic() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resources(&[
            ("blog", None),
            ("forum", None),
            ("wiki", None),
        ])?
        .build()?;

    assert!(acl.has_resource("blog"), "Should have blog resource");
    assert!(acl.has_resource("forum"), "Should have forum resource");
    assert!(acl.has_resource("wiki"), "Should have wiki resource");
    assert_eq!(acl.resource_count(), 3, "Should have exactly 3 resources");

    Ok(())
}

#[test]
fn test_acl_builder_add_resources_with_single_parent() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resources(&[
            ("content", None),
            ("article", Some(&["content"])),
            ("blog_post", Some(&["article"])),
        ])?
        .build()?;

    assert!(acl.inherits_resource("article", "content"), "Article should inherit from content");
    assert!(acl.inherits_resource("blog_post", "article"), "Blog post should inherit from article");
    assert!(acl.inherits_resource("blog_post", "content"), "Blog post should transitively inherit from content");

    Ok(())
}

#[test]
fn test_acl_builder_add_resources_with_multiple_parents() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resources(&[
            ("viewable", None),
            ("editable", None),
            ("deletable", None),
            ("admin_resource", Some(&["viewable", "editable", "deletable"])),
        ])?
        .build()?;

    assert_eq!(acl.resource_count(), 4, "Should have exactly 4 resources");
    assert!(acl.inherits_resource("admin_resource", "viewable"), "Admin resource should inherit from viewable");
    assert!(acl.inherits_resource("admin_resource", "editable"), "Admin resource should inherit from editable");
    assert!(acl.inherits_resource("admin_resource", "deletable"), "Admin resource should inherit from deletable");

    Ok(())
}

#[test]
fn test_acl_builder_add_resources_empty_list() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resources(&[])?
        .build()?;

    assert_eq!(acl.resource_count(), 0, "Should have 0 resources");

    Ok(())
}

#[test]
fn test_acl_builder_add_resources_out_of_order_dependencies() -> Result<(), String> {
    // Add resources in reverse order - children before parents
    let acl = AclBuilder::new()
        .add_resources(&[
            ("level4", Some(&["level3"])),
            ("level3", Some(&["level2"])),
            ("level2", Some(&["level1"])),
            ("level1", None),
        ])?
        .build()?;

    assert_eq!(acl.resource_count(), 4, "Should have exactly 4 resources");
    assert!(acl.inherits_resource("level2", "level1"), "Level2 should inherit from level1");
    assert!(acl.inherits_resource("level3", "level2"), "Level3 should inherit from level2");
    assert!(acl.inherits_resource("level4", "level3"), "Level4 should inherit from level3");
    assert!(acl.inherits_resource("level4", "level1"), "Level4 should transitively inherit from level1");

    Ok(())
}

#[test]
fn test_acl_builder_add_resources_duplicate() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resources(&[
            ("blog", None),
            ("forum", None),
        ])?
        .add_resources(&[
            ("blog", None),  // Duplicate
            ("forum", None), // Duplicate
        ])?
        .build()?;

    assert!(acl.has_resource("blog"), "Should have blog resource");
    assert!(acl.has_resource("forum"), "Should have forum resource");
    assert_eq!(acl.resource_count(), 2, "Should have exactly 2 resources");

    Ok(())
}

#[test]
fn test_acl_builder_add_resources_mixed_with_and_without_parents() -> Result<(), String> {
    let acl = AclBuilder::new()
        .add_resources(&[
            ("public", None),
            ("private", None),
            ("blog", Some(&["public"])),
            ("secret", None),
            ("admin_blog", Some(&["blog", "private"])),
        ])?
        .build()?;

    assert_eq!(acl.resource_count(), 5, "Should have exactly 5 resources");
    assert!(acl.inherits_resource("blog", "public"), "Blog should inherit from public");
    assert!(acl.inherits_resource("admin_blog", "blog"), "Admin blog should inherit from blog");
    assert!(acl.inherits_resource("admin_blog", "private"), "Admin blog should inherit from private");
    assert!(!acl.inherits_resource("secret", "public"), "Secret should not inherit from public");

    Ok(())
}

#[test]
fn test_acl_builder_add_resources_complex_diamond_inheritance() -> Result<(), String> {
    // Create a diamond inheritance pattern:
    //        root
    //       /    \
    //   path_a  path_b
    //       \    /
    //        leaf
    let acl = AclBuilder::new()
        .add_resources(&[
            ("root", None),
            ("path_a", Some(&["root"])),
            ("path_b", Some(&["root"])),
            ("leaf", Some(&["path_a", "path_b"])),
        ])?
        .build()?;

    assert_eq!(acl.resource_count(), 4, "Should have exactly 4 resources");
    assert!(acl.inherits_resource("path_a", "root"), "Path A should inherit from root");
    assert!(acl.inherits_resource("path_b", "root"), "Path B should inherit from root");
    assert!(acl.inherits_resource("leaf", "path_a"), "Leaf should inherit from path A");
    assert!(acl.inherits_resource("leaf", "path_b"), "Leaf should inherit from path B");
    assert!(acl.inherits_resource("leaf", "root"), "Leaf should transitively inherit from root");

    Ok(())
}

#[test]
fn test_acl_builder_add_resources_with_auto_parent_creation() -> Result<(), String> {
    // Add a resource with a parent that doesn't exist yet
    let acl = AclBuilder::new()
        .add_resource("child", Some(&["nonexistent_parent"]))?
        .build()?;

    assert!(acl.has_resource("child"), "Should have child resource");
    assert!(acl.has_resource("nonexistent_parent"), "Should auto-create nonexistent_parent resource");
    assert!(acl.inherits_resource("child", "nonexistent_parent"), "Child should inherit from nonexistent_parent");
    assert_eq!(acl.resource_count(), 2, "Should have exactly 2 resources");

    Ok(())
}

// ============================
// Tests for cycle detection
// ============================

#[test]
fn test_acl_builder_detects_role_self_cycle() {
    let result = AclBuilder::new()
        .add_role("self_ref", None).unwrap()
        .add_role("self_ref", Some(&["self_ref"])).unwrap()
        .build();

    assert!(result.is_err(), "Should detect self-referencing cycle in roles");
}

#[test]
fn test_acl_builder_detects_role_circular_dependency() {
    let result = AclBuilder::new()
        .add_role("role_a", Some(&["role_b"])).unwrap()
        .add_role("role_b", Some(&["role_c"])).unwrap()
        .add_role("role_c", Some(&["role_a"])).unwrap()
        .build();

    assert!(result.is_err(), "Should detect circular dependency in roles");
}

#[test]
fn test_acl_builder_detects_resource_self_cycle() {
    let result = AclBuilder::new()
        .add_resource("self_ref", None).unwrap()
        .add_resource("self_ref", Some(&["self_ref"])).unwrap()
        .build();

    assert!(result.is_err(), "Should detect self-referencing cycle in resources");
}

#[test]
fn test_acl_builder_detects_resource_circular_dependency() {
    let result = AclBuilder::new()
        .add_resource("res_a", Some(&["res_b"])).unwrap()
        .add_resource("res_b", Some(&["res_c"])).unwrap()
        .add_resource("res_c", Some(&["res_a"])).unwrap()
        .build();

    assert!(result.is_err(), "Should detect circular dependency in resources");
}


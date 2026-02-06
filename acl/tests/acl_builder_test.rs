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

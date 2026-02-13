use std::convert::TryFrom;
use walrs_acl::simple::{AclBuilder, AclData};

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
fn test_acl_builder_none_resources_clears_by_role_id() -> Result<(), String> {
    // When resources=None is passed, the by_role_id entries for the given role
    // should be cleared across all resources (per-resource rules for that role are removed)

    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resources(&[("doc1", None), ("doc2", None)])?
        // Set specific privilege rules on specific resources for user
        .allow(Some(&["user"]), Some(&["doc1"]), Some(&["read"]))?
        .allow(Some(&["user"]), Some(&["doc2"]), Some(&["write"]))?
        // Now set a rule for all resources for this user - should clear per-resource rules
        .deny(Some(&["user"]), None, Some(&["delete"]))?
        .build()?;

    // The per-resource rules for user were cleared, so now only the for_all_resources rule applies
    // Since we only set delete denial, other privileges fall back to default (deny)
    assert!(
        !acl.is_allowed(Some("user"), Some("doc1"), Some("read")),
        "User should be denied read on doc1 (per-resource rules cleared)"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("doc2"), Some("write")),
        "User should be denied write on doc2 (per-resource rules cleared)"
    );

    // Delete should be denied on both (explicitly set via resources=None)
    assert!(
        !acl.is_allowed(Some("user"), Some("doc1"), Some("delete")),
        "User should be denied delete on doc1"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("doc2"), Some("delete")),
        "User should be denied delete on doc2"
    );

    Ok(())
}

#[test]
fn test_acl_builder_none_roles_clears_by_resource_id() -> Result<(), String> {
    // When roles=None is passed for a specific resource, the by_role_id map
    // for that resource should be cleared (per-role rules removed)

    let acl = AclBuilder::new()
        .add_roles(&[("user", None), ("admin", None)])?
        .add_resource("document", None)?
        // Set specific rules for specific roles on this resource
        .allow(Some(&["user"]), Some(&["document"]), Some(&["read"]))?
        .allow(Some(&["admin"]), Some(&["document"]), Some(&["write"]))?
        // Now set a rule for all roles on this resource - should clear per-role rules
        .deny(None, Some(&["document"]), Some(&["delete"]))?
        .build()?;

    // The per-role rules were cleared, so now only the for_all_roles rule applies
    // Since we only set delete denial, other privileges fall back to default (deny)
    assert!(
        !acl.is_allowed(Some("user"), Some("document"), Some("read")),
        "User should be denied read (per-role rules cleared)"
    );
    assert!(
        !acl.is_allowed(Some("admin"), Some("document"), Some("write")),
        "Admin should be denied write (per-role rules cleared)"
    );

    // Both should be denied delete (explicitly set via roles=None)
    assert!(
        !acl.is_allowed(Some("user"), Some("document"), Some("delete")),
        "User should be denied delete"
    );
    assert!(
        !acl.is_allowed(Some("admin"), Some("document"), Some("delete")),
        "Admin should be denied delete"
    );

    Ok(())
}

#[test]
fn test_acl_builder_all_none_resets_rules() -> Result<(), String> {
    // When all three parameters (roles, resources, privileges) are None,
    // the entire _rules structure should be reset

    let acl = AclBuilder::new()
        .add_roles(&[("user", None), ("admin", None)])?
        .add_resources(&[("doc1", None), ("doc2", None)])?
        // Set various specific rules
        .allow(Some(&["user"]), Some(&["doc1"]), Some(&["read"]))?
        .allow(Some(&["admin"]), Some(&["doc2"]), Some(&["write"]))?
        .deny(Some(&["user"]), Some(&["doc2"]), Some(&["delete"]))?
        // Now reset everything by passing all None and deny
        .deny(None, None, None)?
        .build()?;

    // All previous rules should be cleared, and the for_all_* deny rule should apply
    assert!(
        !acl.is_allowed(Some("user"), Some("doc1"), Some("read")),
        "User should be denied read on doc1 (rules were reset)"
    );
    assert!(
        !acl.is_allowed(Some("admin"), Some("doc2"), Some("write")),
        "Admin should be denied write on doc2 (rules were reset)"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("doc2"), Some("delete")),
        "User should be denied delete on doc2 (rules were reset)"
    );
    assert!(
        !acl.is_allowed(Some("admin"), Some("doc1"), Some("anything")),
        "Admin should be denied anything on doc1 (for_all deny)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_all_none_resets_rules_then_allow() -> Result<(), String> {
    // Test that after resetting with all None, we can set new rules

    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("document", None)?
        // Set some initial rules
        .deny(Some(&["user"]), Some(&["document"]), Some(&["write"]))?
        // Reset everything with allow
        .allow(None, None, None)?
        .build()?;

    // Everything should be allowed now
    assert!(
        acl.is_allowed(Some("user"), Some("document"), Some("read")),
        "User should be allowed to read"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("document"), Some("write")),
        "User should be allowed to write (deny was cleared by reset)"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("document"), Some("delete")),
        "User should be allowed to delete"
    );

    Ok(())
}

#[test]
fn test_acl_builder_none_resources_with_all_privileges() -> Result<(), String> {
    // Test combining resources=None with privileges=None

    let acl = AclBuilder::new()
        .add_role("editor", None)?
        .add_resources(&[("article", None), ("comment", None)])?
        // Set specific rules on specific resources
        .allow(Some(&["editor"]), Some(&["article"]), Some(&["read", "write"]))?
        .allow(Some(&["editor"]), Some(&["comment"]), Some(&["read"]))?
        // Now deny all privileges on all resources for this role
        .deny(Some(&["editor"]), None, None)?
        .build()?;

    // All privileges on all resources should now be denied for editor
    assert!(
        !acl.is_allowed(Some("editor"), Some("article"), Some("read")),
        "Editor should be denied read on article"
    );
    assert!(
        !acl.is_allowed(Some("editor"), Some("article"), Some("write")),
        "Editor should be denied write on article"
    );
    assert!(
        !acl.is_allowed(Some("editor"), Some("comment"), Some("read")),
        "Editor should be denied read on comment"
    );

    Ok(())
}

#[test]
fn test_acl_builder_none_roles_with_all_privileges() -> Result<(), String> {
    // Test combining roles=None with privileges=None

    let acl = AclBuilder::new()
        .add_roles(&[("user", None), ("admin", None)])?
        .add_resource("system", None)?
        // Set specific rules for specific roles
        .allow(Some(&["user"]), Some(&["system"]), Some(&["read"]))?
        .allow(Some(&["admin"]), Some(&["system"]), Some(&["read", "write"]))?
        // Now allow all roles all privileges on this resource
        .allow(None, Some(&["system"]), None)?
        .build()?;

    // All roles should have all privileges on system
    assert!(
        acl.is_allowed(Some("user"), Some("system"), Some("read")),
        "User should be allowed to read"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("system"), Some("write")),
        "User should be allowed to write"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("system"), Some("read")),
        "Admin should be allowed to read"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("system"), Some("delete")),
        "Admin should be allowed to delete"
    );

    Ok(())
}

#[test]
fn test_acl_builder_complex_clearing_scenario() -> Result<(), String> {
    // Complex test combining various clearing behaviors

    let acl = AclBuilder::new()
        .add_roles(&[("guest", None), ("user", Some(&["guest"])), ("admin", Some(&["user"]))])?
        .add_resources(&[("blog", None), ("admin_panel", None)])?
        // Set various specific rules
        .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
        .allow(Some(&["user"]), Some(&["blog"]), Some(&["write"]))?
        .allow(Some(&["admin"]), Some(&["admin_panel"]), Some(&["manage"]))?
        // Clear per-privilege rules for user on blog
        .deny(Some(&["user"]), Some(&["blog"]), None)?
        // Clear per-role rules for admin_panel
        .allow(None, Some(&["admin_panel"]), Some(&["view"]))?
        .build()?;

    // User should be denied everything on blog (per-privilege rules cleared)
    // except what they inherit from guest
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("read")),
        "User should be allowed to read blog (inherited from guest)"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("blog"), Some("write")),
        "User should be denied write on blog (cleared by deny-all)"
    );

    // All roles should be able to view admin_panel (roles=None)
    assert!(
        acl.is_allowed(Some("guest"), Some("admin_panel"), Some("view")),
        "Guest should be allowed to view admin_panel"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("admin_panel"), Some("view")),
        "User should be allowed to view admin_panel"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("admin_panel"), Some("view")),
        "Admin should be allowed to view admin_panel"
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

    // Convert the ACL back to a builder and build new Acl from it
    let rebuilt_acl = AclBuilder::try_from(original_acl)?.build()?;

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
    let acl = AclBuilder::default().build();

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
    assert!(acl.inherits_resource("level4", "level2"), "Level4 should inherit from level2 (transitive)");
    assert!(acl.inherits_resource("level4", "level1"), "Level4 should inherit from level1 (transitive)");

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
fn test_acl_builder_to_acl_data_conversion() -> Result<(), String> {
    // Create a comprehensive ACL
    let mut builder = AclBuilder::new();
    builder
        .add_role("guest", None)?
        .add_role("user", Some(&["guest"]))?
        .add_role("admin", Some(&["user"]))?
        .add_resource("blog", None)?
        .add_resource("comment", Some(&["blog"]))?
        .add_resource("admin_panel", None)?
        .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
        .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?
        .allow(Some(&["admin"]), None, None)?
        .deny(Some(&["user"]), Some(&["admin_panel"]), None)?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify roles were extracted
    let roles = acl_data.roles.as_ref().expect("Should have roles");
    assert!(roles.iter().any(|(name, _)| name == "guest"), "Should contain guest role");
    assert!(roles.iter().any(|(name, _)| name == "user"), "Should contain user role");
    assert!(roles.iter().any(|(name, _)| name == "admin"), "Should contain admin role");

    // Verify role hierarchies
    let user_role = roles.iter().find(|(name, _)| name == "user").expect("Should find user role");
    assert!(
        user_role.1.as_ref().map(|parents| parents.contains(&"guest".to_string())).unwrap_or(false),
        "User should inherit from guest"
    );

    // Verify resources were extracted
    let resources = acl_data.resources.as_ref().expect("Should have resources");
    assert!(resources.iter().any(|(name, _)| name == "blog"), "Should contain blog resource");
    assert!(resources.iter().any(|(name, _)| name == "comment"), "Should contain comment resource");
    assert!(resources.iter().any(|(name, _)| name == "admin_panel"), "Should contain admin_panel resource");

    // Verify resource hierarchies
    let comment = resources.iter().find(|(name, _)| name == "comment").expect("Should find comment resource");
    assert!(
        comment.1.as_ref().map(|parents| parents.contains(&"blog".to_string())).unwrap_or(false),
        "Comment should inherit from blog"
    );

    // Verify allow rules were extracted
    assert!(acl_data.allow.is_some(), "Should have allow rules");

    // Verify deny rules were extracted
    assert!(acl_data.deny.is_some(), "Should have deny rules");

    // Now convert back to builder and build to verify round-trip conversion
    let mut rebuilt_builder = AclBuilder::try_from(&acl_data)?;
    let rebuilt_acl = rebuilt_builder.build()?;

    // Verify the rebuilt ACL behaves the same
    assert!(
        rebuilt_acl.is_allowed(Some("guest"), Some("blog"), Some("read")),
        "Guest should be able to read blog"
    );
    assert!(
        rebuilt_acl.is_allowed(Some("user"), Some("blog"), Some("write")),
        "User should be able to write to blog"
    );
    assert!(
        rebuilt_acl.is_allowed(Some("admin"), Some("blog"), Some("delete")),
        "Admin should have all privileges in rebuilt ACL"
    );
    assert!(
        !rebuilt_acl.is_allowed(Some("user"), Some("admin_panel"), Some("read")),
        "User should be denied admin_panel in rebuilt ACL"
    );

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_empty() -> Result<(), String> {
    // Create an empty builder
    let builder = AclBuilder::new();

    // Convert to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify all fields are None
    assert!(acl_data.roles.is_none(), "Empty builder should have no roles");
    assert!(acl_data.resources.is_none(), "Empty builder should have no resources");
    assert!(acl_data.allow.is_none(), "Empty builder should have no allow rules");
    assert!(acl_data.deny.is_none(), "Empty builder should have no deny rules");

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_deny_for_all_resources() -> Result<(), String> {
    // Create a builder with deny rules for all resources
    let mut builder = AclBuilder::new();
    builder
        .add_role("restricted", None)?
        .add_resource("blog", None)?
        .deny(Some(&["restricted"]), None, Some(&["delete", "admin"]))?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify deny rules were extracted for "all resources" (*)
    let deny = acl_data.deny.as_ref().expect("Should have deny rules");
    let all_resources_deny = deny.iter().find(|(resource, _)| resource == "*");
    assert!(
        all_resources_deny.is_some(),
        "Should have deny rules for all resources (*)"
    );

    // Verify round-trip works
    let mut rebuilt_builder = AclBuilder::try_from(&acl_data)?;
    let rebuilt_acl = rebuilt_builder.build()?;
    assert!(
        !rebuilt_acl.is_allowed(Some("restricted"), Some("blog"), Some("delete")),
        "Restricted should be denied delete on blog in rebuilt ACL"
    );

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_for_all_roles() -> Result<(), String> {
    // Create a builder with rules for all roles (None roles)
    let mut builder = AclBuilder::new();
    builder
        .add_role("guest", None)?
        .add_resource("public_page", None)?
        .allow(None, Some(&["public_page"]), Some(&["read", "view"]))?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify allow rules were extracted with "*" for all roles
    let allow = acl_data.allow.as_ref().expect("Should have allow rules");
    let public_page_allow = allow.iter().find(|(resource, _)| resource == "public_page");
    assert!(
        public_page_allow.is_some(),
        "Should have allow rules for public_page"
    );

    if let Some((_, role_privileges)) = public_page_allow {
        let all_roles = role_privileges
            .as_ref()
            .and_then(|rp| rp.iter().find(|(role, _)| role == "*"));
        assert!(
            all_roles.is_some(),
            "Should have allow rules for all roles (*)"
        );
    }

    // Verify round-trip works
    let mut rebuilt_builder = AclBuilder::try_from(&acl_data)?;
    let rebuilt_acl = rebuilt_builder.build()?;
    assert!(
        rebuilt_acl.is_allowed(Some("guest"), Some("public_page"), Some("read")),
        "Guest should be able to read public_page in rebuilt ACL"
    );

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_for_all_privileges() -> Result<(), String> {
    // Create a builder with rules for all privileges (None privileges)
    let mut builder = AclBuilder::new();
    builder
        .add_role("superadmin", None)?
        .add_resource("everything", None)?
        .allow(Some(&["superadmin"]), Some(&["everything"]), None)?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify allow rules were extracted with None privileges (all privileges)
    let allow = acl_data.allow.as_ref().expect("Should have allow rules");
    let everything_allow = allow.iter().find(|(resource, _)| resource == "everything");
    assert!(
        everything_allow.is_some(),
        "Should have allow rules for everything resource"
    );

    if let Some((_, role_privileges)) = everything_allow {
        let superadmin_rules = role_privileges
            .as_ref()
            .and_then(|rp| rp.iter().find(|(role, _)| role == "superadmin"));
        assert!(
            superadmin_rules.is_some(),
            "Should have allow rules for superadmin"
        );
        // Privileges should be None indicating all privileges
        if let Some((_, privileges)) = superadmin_rules {
            assert!(
                privileges.is_none(),
                "Privileges should be None (all privileges)"
            );
        }
    }

    // Verify round-trip works
    let mut rebuilt_builder = AclBuilder::try_from(&acl_data)?;
    let rebuilt_acl = rebuilt_builder.build()?;
    assert!(
        rebuilt_acl.is_allowed(Some("superadmin"), Some("everything"), Some("any_privilege")),
        "Superadmin should have all privileges on everything in rebuilt ACL"
    );

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_deny_for_all_privileges() -> Result<(), String> {
    // Create a builder with deny rules for all privileges
    let mut builder = AclBuilder::new();
    builder
        .add_role("banned", None)?
        .add_resource("forum", None)?
        .deny(Some(&["banned"]), Some(&["forum"]), None)?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify deny rules were extracted
    let deny = acl_data.deny.as_ref().expect("Should have deny rules");
    let forum_deny = deny.iter().find(|(resource, _)| resource == "forum");
    assert!(
        forum_deny.is_some(),
        "Should have deny rules for forum resource"
    );

    if let Some((_, role_privileges)) = forum_deny {
        let banned_rules = role_privileges
            .as_ref()
            .and_then(|rp| rp.iter().find(|(role, _)| role == "banned"));
        assert!(
            banned_rules.is_some(),
            "Should have deny rules for banned role"
        );
        // Privileges should be None indicating all privileges denied
        if let Some((_, privileges)) = banned_rules {
            assert!(
                privileges.is_none(),
                "Privileges should be None (all privileges denied)"
            );
        }
    }

    // Verify round-trip works
    let mut rebuilt_builder = AclBuilder::try_from(&acl_data)?;
    let rebuilt_acl = rebuilt_builder.build()?;
    assert!(
        !rebuilt_acl.is_allowed(Some("banned"), Some("forum"), Some("post")),
        "Banned should be denied posting on forum in rebuilt ACL"
    );

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_mixed_allow_deny_same_resource() -> Result<(), String> {
    // Create a builder with both allow and deny rules on the same resource
    let mut builder = AclBuilder::new();
    builder
        .add_role("editor", None)?
        .add_role("viewer", None)?
        .add_resource("document", None)?
        .allow(Some(&["editor"]), Some(&["document"]), Some(&["read", "write", "delete"]))?
        .allow(Some(&["viewer"]), Some(&["document"]), Some(&["read"]))?
        .deny(Some(&["viewer"]), Some(&["document"]), Some(&["write", "delete"]))?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify both allow and deny rules exist
    assert!(acl_data.allow.is_some(), "Should have allow rules");
    assert!(acl_data.deny.is_some(), "Should have deny rules");

    // Verify round-trip works
    let mut rebuilt_builder = AclBuilder::try_from(&acl_data)?;
    let rebuilt_acl = rebuilt_builder.build()?;

    // Editor should have all privileges
    assert!(
        rebuilt_acl.is_allowed(Some("editor"), Some("document"), Some("delete")),
        "Editor should be able to delete document"
    );

    // Viewer should only have read
    assert!(
        rebuilt_acl.is_allowed(Some("viewer"), Some("document"), Some("read")),
        "Viewer should be able to read document"
    );
    assert!(
        !rebuilt_acl.is_allowed(Some("viewer"), Some("document"), Some("write")),
        "Viewer should not be able to write document"
    );

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_multiple_resources_multiple_roles() -> Result<(), String> {
    // Create a complex builder with multiple resources and roles
    let mut builder = AclBuilder::new();
    builder
        .add_role("guest", None)?
        .add_role("member", Some(&["guest"]))?
        .add_role("moderator", Some(&["member"]))?
        .add_resource("posts", None)?
        .add_resource("comments", None)?
        .add_resource("users", None)?
        .allow(Some(&["guest"]), Some(&["posts"]), Some(&["read"]))?
        .allow(Some(&["guest"]), Some(&["comments"]), Some(&["read"]))?
        .allow(Some(&["member"]), Some(&["posts"]), Some(&["create"]))?
        .allow(Some(&["member"]), Some(&["comments"]), Some(&["create", "edit"]))?
        .allow(Some(&["moderator"]), Some(&["posts"]), Some(&["delete"]))?
        .allow(Some(&["moderator"]), Some(&["comments"]), Some(&["delete"]))?
        .allow(Some(&["moderator"]), Some(&["users"]), Some(&["ban"]))?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify all resources have allow rules
    let allow = acl_data.allow.as_ref().expect("Should have allow rules");
    assert!(
        allow.iter().any(|(r, _)| r == "posts"),
        "Should have rules for posts"
    );
    assert!(
        allow.iter().any(|(r, _)| r == "comments"),
        "Should have rules for comments"
    );
    assert!(
        allow.iter().any(|(r, _)| r == "users"),
        "Should have rules for users"
    );

    // Verify round-trip works
    let mut rebuilt_builder = AclBuilder::try_from(&acl_data)?;
    let rebuilt_acl = rebuilt_builder.build()?;

    // Test various combinations
    assert!(
        rebuilt_acl.is_allowed(Some("guest"), Some("posts"), Some("read")),
        "Guest should read posts"
    );
    assert!(
        rebuilt_acl.is_allowed(Some("member"), Some("comments"), Some("edit")),
        "Member should edit comments"
    );
    assert!(
        rebuilt_acl.is_allowed(Some("moderator"), Some("users"), Some("ban")),
        "Moderator should ban users"
    );

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_only_roles_no_resources() -> Result<(), String> {
    // Create a builder with only roles (no resources)
    let mut builder = AclBuilder::new();
    builder
        .add_role("admin", None)?
        .add_role("user", Some(&["admin"]))?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify roles exist but resources don't
    assert!(acl_data.roles.is_some(), "Should have roles");
    assert!(acl_data.resources.is_none(), "Should not have resources");
    assert!(acl_data.allow.is_none(), "Should not have allow rules");
    assert!(acl_data.deny.is_none(), "Should not have deny rules");

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_only_resources_no_roles() -> Result<(), String> {
    // Create a builder with only resources (no roles)
    let mut builder = AclBuilder::new();
    builder
        .add_resource("api", None)?
        .add_resource("api_v2", Some(&["api"]))?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify resources exist but roles don't
    assert!(acl_data.roles.is_none(), "Should not have roles");
    assert!(acl_data.resources.is_some(), "Should have resources");
    assert!(acl_data.allow.is_none(), "Should not have allow rules");
    assert!(acl_data.deny.is_none(), "Should not have deny rules");

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_global_allow_all() -> Result<(), String> {
    // Create a builder with global allow (None, None, None)
    let mut builder = AclBuilder::new();
    builder
        .add_role("superuser", None)?
        .add_resource("system", None)?
        .allow(None, None, None)?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify global allow rule exists
    let allow = acl_data.allow.as_ref().expect("Should have allow rules");
    let global_allow = allow.iter().find(|(resource, _)| resource == "*");
    assert!(
        global_allow.is_some(),
        "Should have global allow rules (*)"
    );

    // Verify round-trip works
    let mut rebuilt_builder = AclBuilder::try_from(&acl_data)?;
    let rebuilt_acl = rebuilt_builder.build()?;
    assert!(
        rebuilt_acl.is_allowed(Some("superuser"), Some("system"), Some("anything")),
        "Superuser should have global access in rebuilt ACL"
    );

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_specific_privileges_only() -> Result<(), String> {
    // Create a builder with only specific privilege rules (no for_all_privileges)
    let mut builder = AclBuilder::new();
    builder
        .add_role("api_user", None)?
        .add_resource("api", None)?
        .allow(Some(&["api_user"]), Some(&["api"]), Some(&["GET", "POST"]))?
        .deny(Some(&["api_user"]), Some(&["api"]), Some(&["DELETE"]))?;

    // Convert the builder to AclData
    let acl_data = AclData::try_from(&builder)?;

    // Verify both allow and deny have specific privileges
    let allow = acl_data.allow.as_ref().expect("Should have allow rules");
    let deny = acl_data.deny.as_ref().expect("Should have deny rules");

    // Find the api resource in allow rules
    let api_allow = allow.iter().find(|(r, _)| r == "api");
    assert!(api_allow.is_some(), "Should have allow rules for api");

    // Find the api resource in deny rules
    let api_deny = deny.iter().find(|(r, _)| r == "api");
    assert!(api_deny.is_some(), "Should have deny rules for api");

    // Verify round-trip works
    let mut rebuilt_builder = AclBuilder::try_from(&acl_data)?;
    let rebuilt_acl = rebuilt_builder.build()?;

    assert!(
        rebuilt_acl.is_allowed(Some("api_user"), Some("api"), Some("GET")),
        "API user should be allowed GET"
    );
    assert!(
        rebuilt_acl.is_allowed(Some("api_user"), Some("api"), Some("POST")),
        "API user should be allowed POST"
    );
    assert!(
        !rebuilt_acl.is_allowed(Some("api_user"), Some("api"), Some("DELETE")),
        "API user should be denied DELETE"
    );

    Ok(())
}

#[test]
fn test_acl_builder_to_acl_data_deny_all_roles_all_privileges_on_resource() -> Result<(), String> {
    use std::convert::TryFrom;
    use walrs_acl::simple::{AclBuilder, AclData};

    // Create AclData directly with a deny rule that has None for role_privileges
    // This means: deny all roles, all privileges on the specified resource
    let acl_data = AclData {
        roles: Some(vec![
            ("user".to_string(), None),
            ("admin".to_string(), None),
        ]),
        resources: Some(vec![
            ("secret_resource".to_string(), None),
            ("public_resource".to_string(), None),
        ]),
        allow: Some(vec![
            ("public_resource".to_string(), Some(vec![
                ("*".to_string(), Some(vec!["read".to_string()])),
            ])),
        ]),
        // This deny rule applies to all roles and all privileges on secret_resource
        deny: Some(vec![
            ("secret_resource".to_string(), None),
        ]),
    };

    // Convert AclData to AclBuilder
    let mut builder = AclBuilder::try_from(&acl_data)?;
    let acl = builder.build()?;

    // Verify the deny rule was applied - all roles should be denied all privileges on secret_resource
    assert!(
        !acl.is_allowed(Some("user"), Some("secret_resource"), Some("read")),
        "User should be denied read on secret_resource"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("secret_resource"), Some("write")),
        "User should be denied write on secret_resource"
    );
    assert!(
        !acl.is_allowed(Some("admin"), Some("secret_resource"), Some("read")),
        "Admin should be denied read on secret_resource"
    );
    assert!(
        !acl.is_allowed(Some("admin"), Some("secret_resource"), Some("delete")),
        "Admin should be denied delete on secret_resource"
    );

    // Verify the allow rule on public_resource still works
    assert!(
        acl.is_allowed(Some("user"), Some("public_resource"), Some("read")),
        "User should be allowed read on public_resource"
    );
    assert!(
        acl.is_allowed(Some("admin"), Some("public_resource"), Some("read")),
        "Admin should be allowed read on public_resource"
    );

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

#[test]
fn test_acl_builder_allow_then_deny_all_privileges_overwrites() -> Result<(), String> {
    // This test demonstrates that setting a "for all privileges" rule after setting
    // per-privilege rules clears out the per-privilege rules.
    // When privileges=None is passed, all per-privilege rules are removed.

    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("document", None)?
        // First, allow specific privileges
        .allow(Some(&["user"]), Some(&["document"]), Some(&["read", "write"]))?
        // Then, deny all privileges (this sets for_all_privileges = Deny and clears by_privilege_id)
        .deny(Some(&["user"]), None, Some(&["delete"]))?
        .build()?;

    // The per-privilege allow rules for "read" and "write" are cleared,
    // so the for_all_privileges deny rule now applies to all privileges
    assert!(
        !acl.is_allowed(Some("user"), Some("document"), Some("read")),
        "User should be denied read (per-privilege rules were cleared)"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("document"), Some("write")),
        "User should be denied write (per-privilege rules were cleared)"
    );

    // Other privileges also fall back to for_all_privileges deny rule
    assert!(
        !acl.is_allowed(Some("user"), Some("document"), Some("delete")),
        "User should be denied delete (for_all_privileges deny rule)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_deny_then_allow_all_privileges_overwrites() -> Result<(), String> {
    // This test demonstrates the opposite: deny specific privileges first,
    // then allow all privileges - per-privilege deny rules should be cleared

    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("document", None)?
        // First, deny specific privileges
        .deny(Some(&["user"]), Some(&["document"]), Some(&["delete", "admin"]))?
        // Then, allow all privileges (this sets for_all_privileges = Allow and clears by_privilege_id)
        .allow(Some(&["user"]), Some(&["document"]), None)?
        .build()?;

    // The per-privilege deny rules for "delete" and "admin" are cleared,
    // so the for_all_privileges allow rule now applies to all privileges
    assert!(
        acl.is_allowed(Some("user"), Some("document"), Some("delete")),
        "User should be allowed delete (per-privilege deny rules were cleared)"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("document"), Some("admin")),
        "User should be allowed admin (per-privilege deny rules were cleared)"
    );

    // All other privileges should also be allowed
    assert!(
        acl.is_allowed(Some("user"), Some("document"), Some("read")),
        "User should be allowed to read (for_all_privileges allow rule)"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("document"), Some("write")),
        "User should be allowed to write (for_all_privileges allow rule)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_multiple_per_privilege_rules_then_all_privileges() -> Result<(), String> {
    // Test with multiple per-privilege rules set before setting for_all_privileges rule

    let acl = AclBuilder::new()
        .add_role("editor", None)?
        .add_resource("article", None)?
        // Allow some specific privileges
        .allow(Some(&["editor"]), Some(&["article"]), Some(&["read", "write"]))?
        // Deny another specific privilege
        .deny(Some(&["editor"]), Some(&["article"]), Some(&["delete"]))?
        // Now deny all privileges (should clear all the above per-privilege rules)
        .deny(Some(&["editor"]), Some(&["article"]), None)?
        .build()?;

    // All per-privilege rules are cleared, so for_all_privileges deny applies
    assert!(
        !acl.is_allowed(Some("editor"), Some("article"), Some("read")),
        "Editor should be denied read (per-privilege rules cleared)"
    );
    assert!(
        !acl.is_allowed(Some("editor"), Some("article"), Some("write")),
        "Editor should be denied write (per-privilege rules cleared)"
    );
    assert!(
        !acl.is_allowed(Some("editor"), Some("article"), Some("delete")),
        "Editor should be denied delete (for_all_privileges deny)"
    );
    assert!(
        !acl.is_allowed(Some("editor"), Some("article"), Some("publish")),
        "Editor should be denied publish (for_all_privileges deny)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_overwrite_same_privilege_rule() -> Result<(), String> {
    // Test that setting the same privilege rule multiple times overwrites the previous one

    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("file", None)?
        // First allow read
        .allow(Some(&["user"]), Some(&["file"]), Some(&["read"]))?
        // Then deny read (should overwrite the allow)
        .deny(Some(&["user"]), Some(&["file"]), Some(&["read"]))?
        .build()?;

    assert!(
        !acl.is_allowed(Some("user"), Some("file"), Some("read")),
        "User should be denied read (deny rule overwrites allow rule)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_all_privileges_then_per_privilege() -> Result<(), String> {
    // Test setting for_all_privileges first, then adding per-privilege rules

    let acl = AclBuilder::new()
        .add_role("admin", None)?
        .add_resource("system", None)?
        // First deny all privileges
        .deny(Some(&["admin"]), Some(&["system"]), None)?
        // Then allow a specific privilege (this recreates by_privilege_id)
        .allow(Some(&["admin"]), Some(&["system"]), Some(&["read"]))?
        .build()?;

    // The specific privilege "read" should be allowed
    assert!(
        acl.is_allowed(Some("admin"), Some("system"), Some("read")),
        "Admin should be allowed to read (per-privilege rule)"
    );

    // Other privileges should fall back to for_all_privileges deny
    assert!(
        !acl.is_allowed(Some("admin"), Some("system"), Some("write")),
        "Admin should be denied write (for_all_privileges deny)"
    );
    assert!(
        !acl.is_allowed(Some("admin"), Some("system"), Some("restart")),
        "Admin should be denied restart (for_all_privileges deny)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_complex_allow_deny_interaction() -> Result<(), String> {
    // Complex scenario with multiple roles and resources

    let acl = AclBuilder::new()
        .add_roles(&[
            ("guest", None),
            ("user", Some(&["guest"])),
            ("moderator", Some(&["user"])),
        ])?
        .add_resource("post", None)?
        // Guest: allow read
        .allow(Some(&["guest"]), Some(&["post"]), Some(&["read"]))?
        // User: allow read and comment
        .allow(Some(&["user"]), Some(&["post"]), Some(&["read", "comment"]))?
        // User: deny all privileges (clears the read and comment per-privilege rules)
        .deny(Some(&["user"]), Some(&["post"]), None)?
        // User: now allow only edit (re-adds a per-privilege rule)
        .allow(Some(&["user"]), Some(&["post"]), Some(&["edit"]))?
        // Moderator: allow edit and delete
        .allow(Some(&["moderator"]), Some(&["post"]), Some(&["edit", "delete"]))?
        .build()?;

    // User should have edit permission (explicitly allowed after deny-all)
    assert!(
        acl.is_allowed(Some("user"), Some("post"), Some("edit")),
        "User should be allowed to edit"
    );
    // User should have read permission (inherited from guest role)
    assert!(
        acl.is_allowed(Some("user"), Some("post"), Some("read")),
        "User should be allowed to read (inherited from guest)"
    );
    // User should be denied comment (per-privilege rule was cleared and not in guest)
    assert!(
        !acl.is_allowed(Some("user"), Some("post"), Some("comment")),
        "User should be denied comment (per-privilege rule was cleared)"
    );

    // Moderator should have edit and delete (plus inherited read from guest)
    assert!(
        acl.is_allowed(Some("moderator"), Some("post"), Some("edit")),
        "Moderator should be allowed to edit"
    );
    assert!(
        acl.is_allowed(Some("moderator"), Some("post"), Some("delete")),
        "Moderator should be allowed to delete"
    );
    assert!(
        acl.is_allowed(Some("moderator"), Some("post"), Some("read")),
        "Moderator should be allowed to read (inherited from guest via user)"
    );

    Ok(())
}

#[test]
fn test_acl_builder_per_privilege_allow_initially_cleared_by_deny_all() -> Result<(), String> {
    // This is the key test case requested: per-privilege allow rules should be
    // cleared when a for_all_privileges deny rule is set

    let acl = AclBuilder::new()
        .add_role("developer", None)?
        .add_resource("database", None)?
        // Set initial allow rules for specific privileges
        .allow(Some(&["developer"]), Some(&["database"]), Some(&["read", "write", "backup"]))?
        // Now deny all privileges - this should clear the above per-privilege rules
        .deny(Some(&["developer"]), Some(&["database"]), None)?
        .build()?;

    // All previously allowed privileges should now be denied
    assert!(
        !acl.is_allowed(Some("developer"), Some("database"), Some("read")),
        "Developer should be denied read after deny-all clears per-privilege allow rules"
    );
    assert!(
        !acl.is_allowed(Some("developer"), Some("database"), Some("write")),
        "Developer should be denied write after deny-all clears per-privilege allow rules"
    );
    assert!(
        !acl.is_allowed(Some("developer"), Some("database"), Some("backup")),
        "Developer should be denied backup after deny-all clears per-privilege allow rules"
    );
    assert!(
        !acl.is_allowed(Some("developer"), Some("database"), Some("delete")),
        "Developer should be denied delete (for_all_privileges deny)"
    );

    Ok(())
}



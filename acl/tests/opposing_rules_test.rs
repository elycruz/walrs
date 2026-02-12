use walrs_acl::simple::{AclBuilder};

#[test]
fn test_deny_clears_allow_rules() -> Result<(), String> {
    // Test that setting a deny rule clears opposing allow rules
    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("blog", None)?
        // First, allow read and write
        .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?
        // Then deny write - should clear the allow for write but keep read
        .deny(Some(&["user"]), Some(&["blog"]), Some(&["write"]))?
        .build()?;

    // User should still be able to read (allow was not cleared)
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("read")),
        "User should be able to read blog (allow rule should remain)"
    );

    // User should NOT be able to write (deny should have cleared the allow)
    assert!(
        !acl.is_allowed(Some("user"), Some("blog"), Some("write")),
        "User should be denied write to blog (deny should clear allow)"
    );

    Ok(())
}

#[test]
fn test_allow_clears_deny_rules() -> Result<(), String> {
    // Test that setting an allow rule clears opposing deny rules
    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("blog", None)?
        // First, deny read and write
        .deny(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?
        // Then allow write - should clear the deny for write but keep read denied
        .allow(Some(&["user"]), Some(&["blog"]), Some(&["write"]))?
        .build()?;

    // User should NOT be able to read (deny was not cleared)
    assert!(
        !acl.is_allowed(Some("user"), Some("blog"), Some("read")),
        "User should be denied read on blog (deny rule should remain)"
    );

    // User should be able to write (allow should have cleared the deny)
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("write")),
        "User should be allowed to write to blog (allow should clear deny)"
    );

    Ok(())
}

#[test]
fn test_deny_all_clears_all_allow_rules() -> Result<(), String> {
    // Test that setting deny for all privileges clears all opposing allow rules
    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("blog", None)?
        // First, allow specific privileges
        .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write", "delete"]))?
        // Then deny all privileges - should clear all allows
        .deny(Some(&["user"]), Some(&["blog"]), None)?
        .build()?;

    // User should be denied all privileges
    assert!(
        !acl.is_allowed(Some("user"), Some("blog"), Some("read")),
        "User should be denied read (deny-all should clear specific allow)"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("blog"), Some("write")),
        "User should be denied write (deny-all should clear specific allow)"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("blog"), Some("delete")),
        "User should be denied delete (deny-all should clear specific allow)"
    );
    assert!(
        !acl.is_allowed(Some("user"), Some("blog"), None),
        "User should be denied all privileges"
    );

    Ok(())
}

#[test]
fn test_allow_all_clears_all_deny_rules() -> Result<(), String> {
    // Test that setting allow for all privileges clears all opposing deny rules
    let acl = AclBuilder::new()
        .add_role("user", None)?
        .add_resource("blog", None)?
        // First, deny specific privileges
        .deny(Some(&["user"]), Some(&["blog"]), Some(&["read", "write", "delete"]))?
        // Then allow all privileges - should clear all denies
        .allow(Some(&["user"]), Some(&["blog"]), None)?
        .build()?;

    // User should be allowed all privileges
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("read")),
        "User should be allowed read (allow-all should clear specific deny)"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("write")),
        "User should be allowed write (allow-all should clear specific deny)"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), Some("delete")),
        "User should be allowed delete (allow-all should clear specific deny)"
    );
    assert!(
        acl.is_allowed(Some("user"), Some("blog"), None),
        "User should be allowed all privileges"
    );

    Ok(())
}

#[test]
fn test_opposing_rule_clearing_with_inheritance() -> Result<(), String> {
    // Test that deny on a role should override inherited allow from parent role
    let acl = AclBuilder::new()
        .add_role("guest", None)?
        .add_role("user", Some(&["guest"]))?
        .add_resource("blog", None)?
        // Allow guest to read
        .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
        // Deny user (child of guest) to read - should override inherited allow
        .deny(Some(&["user"]), Some(&["blog"]), Some(&["read"]))?
        .build()?;

    // Guest should be able to read
    assert!(
        acl.is_allowed(Some("guest"), Some("blog"), Some("read")),
        "Guest should be allowed to read blog"
    );

    // User should be denied read (deny should block inherited allow from guest)
    assert!(
        !acl.is_allowed(Some("user"), Some("blog"), Some("read")),
        "User should be denied read on blog (explicit deny should override inherited allow)"
    );

    Ok(())
}


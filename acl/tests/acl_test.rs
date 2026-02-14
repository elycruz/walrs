use std::convert::TryFrom;
use std::fs::File;
use walrs_acl::simple::{Acl, AclBuilder, AclData};
use std::io::BufReader;

#[test]
pub fn test_acl_data_from_file_ref() -> Result<(), Box<dyn std::error::Error>> {
  let file_path = "./test-fixtures/example-extensive-acl-array.json";

  // Get digraph data
  let mut f = File::open(&file_path)?;

  let acl_data = AclData::try_from(&mut f)?;
  let acl: Acl = AclBuilder::try_from(&acl_data)?.build()?;

  // Tests
  // ----

  // For currently loaded file
  assert_ne!(
    acl.role_count(),
    0,
    "Parsed *acl.json result should contain roles"
  );
  assert_ne!(
    acl.resource_count(),
    0,
    "Parsed *acl.json result should contain resources"
  );

  // Validate roles
  // ----
  let roles = acl_data.roles.expect("acl_data should have roles");
  let role_count = acl.role_count();

  assert_eq!(role_count, roles.len(), "Role lengths do not match");

  // Check role inheritance relationships
  for (role, inherited) in roles.iter() {
    assert!(acl.has_role(role), "acl should contain role \"{}\"", role);

    // Check inherited relationships
    if let Some(inherited_roles) = inherited.as_deref() {
      for role2 in inherited_roles.iter() {
        assert!(
          acl.inherits_role(&**role, &**role2),
          "\"{}\" role should inherit role \"{}\"",
          role,
          role2
        );
      }
    }
  }

  // Validate resources
  // ----
  let resources = acl_data.resources.expect("acl_data should have resources");
  let resource_count = acl.resource_count();

  assert_eq!(resource_count, resources.len(), "Resource lengths do not match");

  // Check resource inheritance relationships
  for (resource, inherited) in resources.iter() {
    assert!(
      acl.has_resource(resource),
      "acl should contain resource \"{}\"",
      resource
    );

    // Check inherited relationships
    if let Some(inherited_resources) = inherited.as_deref() {
      for resource2 in inherited_resources.iter() {
        assert!(
          acl.inherits_resource(&**resource, &**resource2),
          "\"{}\" resource should inherit resource \"{}\"",
          resource,
          resource2
        );
      }
    }
  }

  // Check "allow" rules
  // ----
  println!("Check \"allow\" rules...");
  let allow_rules = acl_data.allow.expect("acl_data should have allow rules");
  for (resource, role_privileges) in allow_rules.iter() {
    if let Some(rps) = role_privileges.as_ref() {
      for (role, privileges) in rps.iter() {
        if let Some(privilege_list) = privileges.as_ref() {
          // Allow specific privileges
          for privilege in privilege_list.iter() {
            eprintln!(
              "Testing acl.is_allowed({:?}, {:?}, {:?})",
              role, resource, privilege
            );
            assert!(
              acl.is_allowed(
                Some(role.as_ref()),
                Some(resource.as_ref()),
                Some(privilege.as_ref())
              ),
              "Role {:?} should be allowed privilege {:?} on resource {:?}",
              role, privilege, resource
            );
          }
        } else {
          // Allow all privileges (privileges is None)
          eprintln!(
            "Testing acl.is_allowed({:?}, {:?}, None)",
            role, resource
          );
          assert!(
            acl.is_allowed(
              Some(role.as_ref()),
              Some(resource.as_ref()),
              None
            ),
            "Role {:?} should be allowed all privileges on resource {:?}",
            role, resource
          );
        }
      }
    }
  }

  // Check "deny" rules
  // ----
  println!("Check \"deny\" rules...");
  let deny_rules = acl_data.deny.expect("acl_data should have deny rules");
  for (resource, role_privileges) in deny_rules.iter() {
    if let Some(rps) = role_privileges.as_ref() {
      for (role, privileges) in rps.iter() {
        if let Some(privilege_list) = privileges.as_ref() {
          // Deny specific privileges
          for privilege in privilege_list.iter() {
            eprintln!(
              "Testing !acl.is_allowed({:?}, {:?}, {:?})",
              role, resource, privilege
            );
            assert!(
              !acl.is_allowed(
                Some(role.as_ref()),
                Some(resource.as_ref()),
                Some(privilege.as_ref())
              ),
              "Role {:?} should be denied privilege {:?} on resource {:?}",
              role, privilege, resource
            );
          }
        } else {
          // Deny all privileges (privileges is None)
          eprintln!(
            "Testing !acl.is_allowed({:?}, {:?}, None)",
            role, resource
          );
          assert!(
            !acl.is_allowed(
              Some(role.as_ref()),
              Some(resource.as_ref()),
              None
            ),
            "Role {:?} should be denied all privileges on resource {:?}",
            role, resource
          );
        }
      }
    }
  }

  Ok(())
}

#[test]
pub fn test_acl_data_from_mut_file_ref() -> Result<(), Box<dyn std::error::Error>> {
  let file_path = "./test-fixtures/example-extensive-acl-array.json";

  // Get digraph data
  let mut f = File::open(&file_path)?;

  let _: AclData = AclData::try_from(&mut f)?;
  let _ = BufReader::new(f);

  // println!("{:?}", &acl_data);
  Ok(())
}

#[test]
pub fn test_e2e_example_one() -> Result<(), Box<dyn std::error::Error>> {
  use walrs_acl::simple::Acl;

  let file_path = "./test-fixtures/example-acl-allow-and-deny-rules.json";

  // Load ACL data from JSON file
  let mut f = File::open(&file_path)?;
  let acl_data: AclData = AclData::try_from(&mut f)?;

  // Convert AclData to Acl
  let acl: Acl = AclBuilder::try_from(acl_data)?.build()?;

  // ----
  // Test roles are loaded correctly
  // ----
  let expected_roles = ["guest", "user", "admin", "special-user", "super-admin"];
  for role in expected_roles {
    assert!(acl.has_role(role), "ACL should contain role: {:?}", role);
  }

  // Test role inheritance
  assert!(acl.inherits_role("user", "guest"), "user should inherit from guest");
  assert!(acl.inherits_role("admin", "user"), "admin should inherit from user");
  assert!(acl.inherits_role("admin", "guest"), "admin should inherit from guest (transitively)");
  assert!(acl.inherits_role("super-admin", "admin"), "super-admin should inherit from admin");
  assert!(acl.inherits_role("super-admin", "special-user"), "super-admin should inherit from special-user");

  // ----
  // Test resources are loaded correctly
  // ----
  let expected_resources = ["index", "blog", "account", "user", "users", "rest-resource"];
  for resource in expected_resources {
    assert!(acl.has_resource(resource), "ACL should contain resource: {:?}", resource);
  }

  // Test resource inheritance
  assert!(acl.inherits_resource("blog", "index"), "blog should inherit from index");

  // ----
  // Test allow rules
  // ----

  // Test "index" resource - allow all roles all privileges
  assert!(acl.is_allowed(Some("guest"), Some("index"), None), "guest should be allowed all privileges on index");
  assert!(acl.is_allowed(Some("user"), Some("index"), None), "user should be allowed all privileges on index");

  // Test "user" resource privileges
  assert!(acl.is_allowed(Some("guest"), Some("user"), Some("index")), "guest should have index privilege on user resource");
  assert!(acl.is_allowed(Some("guest"), Some("user"), Some("read")), "guest should have read privilege on user resource");
  assert!(acl.is_allowed(Some("user"), Some("user"), Some("update")), "user should have update privilege on user resource");
  assert!(acl.is_allowed(Some("admin"), Some("user"), Some("create")), "admin should have create privilege on user resource");
  assert!(acl.is_allowed(Some("super-admin"), Some("user"), Some("delete")), "super-admin should have delete privilege on user resource");

  // Test "account" resource privileges
  assert!(acl.is_allowed(Some("guest"), Some("account"), Some("index")), "guest should have index privilege on account");
  assert!(acl.is_allowed(Some("guest"), Some("account"), Some("read")), "guest should have read privilege on account");
  assert!(acl.is_allowed(Some("user"), Some("account"), Some("update")), "user should have update privilege on account");
  assert!(acl.is_allowed(Some("admin"), Some("account"), Some("create")), "admin should have create privilege on account");
  assert!(acl.is_allowed(Some("super-admin"), Some("account"), Some("delete")), "super-admin should have delete privilege on account");

  // Test "blog" resource privileges
  assert!(acl.is_allowed(Some("guest"), Some("blog"), Some("index")), "guest should have index privilege on blog");
  assert!(acl.is_allowed(Some("guest"), Some("blog"), Some("read")), "guest should have read privilege on blog");
  assert!(acl.is_allowed(Some("user"), Some("blog"), None), "user should have all privileges on blog");

  // Test "users" resource - admin has all privileges
  assert!(acl.is_allowed(Some("admin"), Some("users"), None), "admin should have all privileges on users");

  // Test "rest-resource" privileges
  assert!(acl.is_allowed(Some("guest"), Some("rest-resource"), Some("get")), "guest should have get privilege on rest-resource");
  assert!(acl.is_allowed(Some("special-user"), Some("rest-resource"), Some("get")), "special-user should have get privilege on rest-resource");
  assert!(acl.is_allowed(Some("user"), Some("rest-resource"), Some("post")), "user should have post privilege on rest-resource");
  assert!(acl.is_allowed(Some("admin"), Some("rest-resource"), Some("put")), "admin should have put privilege on rest-resource");
  assert!(acl.is_allowed(Some("super-admin"), Some("rest-resource"), Some("delete")), "super-admin should have delete privilege on rest-resource");

  // Test non-conforming pathname
  assert!(acl.is_allowed(Some("special-user"), Some("non/conforming/pathname"), Some("further/sub/path")),
          "special-user should have further/sub/path privilege on non/conforming/pathname");

  // ----
  // Test deny rules
  // ----

  // Test "users" resource deny rules - guest and user should be denied
  assert!(!acl.is_allowed(Some("guest"), Some("users"), None), "guest should be denied all privileges on users");
  assert!(!acl.is_allowed(Some("user"), Some("users"), None), "user should be denied all privileges on users");

  // Test non-existent roles/resources
  // Note: The "index" resource allows ALL roles, so non-existent roles would be allowed on it.
  // We test against "account" which only allows specific roles.
  assert!(!acl.is_allowed(Some("non-existent-role"), Some("account"), Some("read")),
          "non-existent role should not be allowed on account resource");
  assert!(!acl.is_allowed(Some("guest"), Some("non-existent-resource"), None),
          "non-existent resource should not be allowed");

  Ok(())
}

#[test]
#[should_panic]
pub fn test_from_invalid_acl_json_file() {
  let file_path = "./test-fixtures/invalid-acl.json";
  let mut f = File::open(&file_path).unwrap();
  let _acl_data: AclData = AclData::try_from(&mut f).unwrap();
}

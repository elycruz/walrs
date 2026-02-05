use std::convert::TryFrom;
use std::fs::File;
use walrs_acl::simple::{Acl, AclData};
use std::io::BufReader;

#[test]
pub fn test_acl_data_from_file_ref() -> Result<(), Box<dyn std::error::Error>> {
  let file_path = "./test-fixtures/example-acl.json";

  // Get digraph data
  let mut f = File::open(&file_path)?;

  let acl_data = AclData::try_from(&mut f)?;
  let acl: Acl = Acl::try_from(&acl_data)?;

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
  if let Some(roles) = acl_data.roles {
    let role_count = acl.role_count();

    assert_eq!(role_count, roles.len(), "Role lengths do not match");

    // Check role inheritance relationships
    roles.iter().for_each(|(role, inherited)| {
      assert!(acl.has_role(role), "acl should contain role \"{}\"", role);

      // Check inherited relationships
      if let Some(inherited_roles) = inherited.as_deref() {
        // For role ensure role1 inherits incoming role
        inherited_roles.iter().for_each(|role2| {
          assert_eq!(
            acl.inherits_role(&**role, &**role2),
            true,
            "\"{}\" role should inherit roles \"{:?}\"",
            role,
            inherited.as_ref()
          );
        });
      }
    });
  }

  // Validate resources
  // ----
  if let Some(resources) = acl_data.resources {
    let resource_count = acl.resource_count();

    assert_eq!(resource_count, resources.len(), "Role lengths do not match");

    // Check resource inheritance relationships
    resources.iter().for_each(|(resource, inherited)| {
      assert!(
        acl.has_resource(resource),
        "acl should contain resource \"{}\"",
        resource
      );

      // Check inherited relationships
      if let Some(inherited_resources) = inherited.as_deref() {
        // For resource ensure resource1 inherits incoming resource
        inherited_resources.iter().for_each(|resource2| {
          assert_eq!(
            acl.inherits_resource(&**resource, &**resource2),
            true,
            "\"{}\" resource should inherit resources \"{:?}\"",
            resource,
            inherited.as_ref()
          );
        });
      }
    });
  }

  // Check "allow" rules
  // ----
  println!("Check \"allow\" rules...");
  if let Some(allow) = acl_data.allow {
    allow.iter().for_each(|(resource, role_privileges)| {
      role_privileges.as_deref().iter().map(|rps| {
        rps.iter().for_each(|(role, privileges)| {
          if let Some(_privileges) = privileges.as_deref() {
            _privileges.iter().for_each(|xs| {
              eprintln!(
                "Testing acl.is_allowed({:?}, {:?}, {:?})",
                role, resource, xs
              );
              assert!(acl.is_allowed(
                Some(role.as_ref()),
                Some(resource.as_ref()),
                Some(xs.as_ref())
              ));
            });
          }
        });
      });
    });
  }

  // Check "deny" rules

  // println!("{:?}", &acl);

  Ok(())
}

#[test]
pub fn test_acl_data_from_mut_file_ref() -> Result<(), Box<dyn std::error::Error>> {
  let file_path = "./test-fixtures/example-acl.json";

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
  let acl: Acl = Acl::try_from(acl_data)?;

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

use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;

use walrs_acl::simple::AclData;

#[test]
pub fn test_from_file_ref() -> Result<(), Box<dyn std::error::Error>> {
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

  // Convert AclData to Acl
  let acl: Acl = Acl::try_from(&mut f)?;

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

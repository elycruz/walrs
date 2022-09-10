use std::fs::File;
use std::io::BufReader;

use ecms_acl::simple::{Acl, AclData};

#[test]
pub fn test_from_file_ref() -> Result<(), std::io::Error> {
  let file_path = "./test-fixtures/example-acl.json";

  // Get digraph data
  let mut f = File::open(&file_path)?;

  let acl_data = AclData::from(&mut f);
  let acl: Acl = Acl::from(&acl_data);

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

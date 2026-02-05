use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;

use serde_json;
use walrs_graph::digraph::{DigraphDFSShape, DirectedCycle, DirectedPathsDFS, DisymGraph};

use crate::simple::acl_data::AclData;
use crate::simple::rule::{Rule};
use crate::simple::privilege_rules::PrivilegeRules;
use crate::simple::resource_role_rules::ResourceRoleRules;
use crate::simple::role_privilege_rules::RolePrivilegeRules;

// Note: Rules structure:
// Resources contain roles, roles contain privileges,
// privileges contain allow/deny rules, and/or, assertion functions,
// Privilege, Role, and Resource Ids are string slices - See relevant imports
// for more.
// ----

/// Lite-weight Access Control List (ACL) structure - Provides a structure
/// that can be queried for allow/deny rules for given roles, resources, and privilege,
/// combinations.
///
/// Note: This implementation does not expose any `*remove*` methods as both 'allow', and 'deny',
/// rules can be set for any given role, resource, and/or privilege, and, additionally, any
/// conditional logic can be performed at declaration time.
///
/// Note: If you require the above-mentioned functionality please open an issue ticket for it.
///
/// ```rust
/// // TODO.
/// ```
#[derive(Debug)]
pub struct Acl {
  _roles: DisymGraph,
  _resources: DisymGraph,
  _rules: ResourceRoleRules,
}

impl Acl {
  /// Creates a new Acl instance.
  pub fn new() -> Self {
    Acl {
      _roles: DisymGraph::new(),
      _resources: DisymGraph::new(),
      _rules: ResourceRoleRules::new(),
    }
  }

  /// Returns the number roles in the Acl.
  pub fn role_count(&self) -> usize {
    self._roles.vert_count()
  }

  /// Returns the number resources in the Acl.
  pub fn resource_count(&self) -> usize {
    self._resources.vert_count()
  }

  /// Adds a `Role` to acl.
  /// ```rust
  /// use std::ops::Deref;
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new() as Acl;
  ///
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  /// let tester = "tester";
  /// let developer = "developer";
  ///
  /// // Add roles, and their relationships to the acl:
  /// acl .add_role(developer, Some(&[tester]))?
  ///     .add_role(admin, Some(&[developer]))?
  ///     .add_role(super_admin, Some(&[admin]))?;
  ///
  /// // Assert existence
  /// for r in [admin, super_admin, tester, developer] {
  ///     assert!(acl.has_role(r), "Should contain {:?} role", r);
  /// }
  ///
  /// // Assert inheritance
  /// assert_eq!(acl.inherits_role_safe(super_admin, admin).unwrap(), true,
  ///   "{:?} should have `child -> parent` relationship`with {:?}", super_admin, admin);
  ///
  /// assert_eq!(acl.inherits_role_safe(developer, tester).unwrap(), true,
  ///   "{:?} should have `child -> parent` relationship`with {:?}", developer, tester);
  /// # Ok::<(), String>(())
  /// ```
  pub fn add_role(&mut self, role: &str, parents: Option<&[&str]>) -> Result<&mut Self, String> {
    if let Some(parents) = parents {
      self._roles.add_edge(role, parents)?;
    }
    self._roles.add_vertex(role);
    Ok(self)
  }

  /// Adds multiple `Role`s to acl at once.
  ///
  /// Example:
  /// ```rust
  /// use std::ops::Deref;
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new() as Acl;
  ///
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  /// let tester = "tester";
  /// let developer = "developer";
  ///
  /// // Add roles, and their relationships to the acl:
  /// acl.add_roles(&[
  ///     (developer, Some(&[tester])),
  ///     (admin, Some(&[developer])),
  ///     (super_admin, Some(&[admin])),
  /// ])?;
  ///
  /// // Assert existence
  /// for r in [admin, super_admin, tester, developer] {
  ///     assert!(acl.has_role(r), "Should contain {:?} role", r);
  /// }
  ///
  /// // Assert inheritance
  /// assert_eq!(acl.inherits_role_safe(super_admin, admin).unwrap(), true,
  ///   "{:?} should have `child -> parent` relationship`with {:?}", super_admin, admin);
  ///
  /// assert_eq!(acl.inherits_role_safe(developer, tester).unwrap(), true,
  ///   "{:?} should have `child -> parent` relationship`with {:?}", developer, tester);
  /// # Ok::<(), String>(())
  /// ```
  pub fn add_roles(&mut self, roles: &[(&str, Option<&[&str]>)]) -> Result<&mut Self, String> {
    for &(role, parents) in roles {
      self.add_role(role, parents)?;
    }
    Ok(self)
  }

  /// Returns a boolean indicating whether Acl contains given role or not.
  pub fn has_role(&self, role: &str) -> bool {
    self._roles.has_vertex(role.as_ref())
  }

  /// Returns a boolean indicating whether `role` inherits `inherits` (... extends it etc.).
  ///
  /// ```rust
  /// use std::ops::Deref;
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new() as Acl;
  /// let guest = "guest";
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  ///
  /// // Add roles, and their relationships to the acl:
  /// acl.add_role(&guest, None)?
  ///     .add_role(&admin, Some(&[&guest]))?
  ///     .add_role(&super_admin, Some(&[&admin]))?;
  ///
  /// // Test created relationships (DAG edges)
  /// assert_eq!(acl.inherits_role_safe(&guest, &admin).is_ok(), true, "result should be `Ok(...)`");
  /// assert_eq!(acl.inherits_role_safe(&guest, &admin).unwrap(), false,
  ///   "{:?} role should not inherit from {:?} role",
  ///   guest,
  ///   admin
  /// );
  ///
  /// assert!(acl.inherits_role_safe(&admin, &guest).unwrap(), "\"admin\" role should inherit \"guest\" role");
  /// assert!(acl.inherits_role_safe(&super_admin, &guest).unwrap(), "\"super_admin\" role should inherit \"guess\" role");
  /// # Ok::<(), String>(())
  /// ```
  pub fn inherits_role_safe(&self, role: &str, inherits: &str) -> Result<bool, String> {
    if let Some((v1, v2)) = self._roles.index(role).zip(self._roles.index(inherits)) {
      return DirectedPathsDFS::new(self._roles.graph(), v1).and_then(|dfs| dfs.has_path_to(v2));
    }
    Err(format!("{} is not in symbol graph", inherits))
  }

  /// Returns a boolean indicating whether `role` inherits `inherits` (... extends it etc.).
  /// Note: Method panics if `role`, and/or `inherits`, is not registered/added on acl;  For safe version use
  ///  `#Acl.inherits_role_safe`.
  ///
  /// ```rust
  /// use std::ops::Deref;
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new();
  /// let guest = "guest";
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  ///
  /// // Add roles, and their relationships
  /// acl.add_role(&guest, None)?
  ///     .add_role(&admin, Some(&[&guest]))?
  ///     .add_role(&super_admin, Some(&[&admin]))?;
  ///
  /// // Test relationships
  /// assert_eq!(acl.inherits_role(&guest, &admin), false,
  ///   "{:?} role should not inherit from {:?} role",
  ///   guest,
  ///   admin
  /// );
  /// assert!(acl.inherits_role(&admin, &guest), "\"{}\" role should inherit \"{}\" role", &admin, &guest);
  /// assert!(acl.inherits_role(&super_admin, &guest), "\"{}\" role should inherit \"{}\" role", &super_admin, &guest);
  /// assert!(acl.inherits_role(&super_admin, &admin), "\"{}\" role should inherit \"{}\" role", &super_admin, &admin);
  /// # Ok::<(), String>(())
  /// ```
  pub fn inherits_role(&self, role: &str, inherits: &str) -> bool {
    match self.inherits_role_safe(role, inherits) {
      Ok(is_inherited) => is_inherited,
      Err(err) => panic!("{}", err),
    }
  }

  /// Adds a `Resource` to acl.
  ///
  /// ```rust
  /// use std::ops::Deref;
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new();
  /// let term = "term";
  /// let post = "post";
  /// let post_categories = "post_categories";
  ///
  /// // Add resources, and their relationships
  /// acl.add_resource(&term, None)?
  ///     .add_resource(&post, Some(&[&term]))?
  ///     .add_resource(&post_categories, Some(&[&term]))?;
  ///
  /// // Test existence resources
  /// assert!(acl.has_resource(&term), "Should contain {:?} resource", &term);
  /// assert!(acl.has_resource(&post), "Should contain {:?} resource", &post);
  /// assert!(acl.has_resource(&post_categories), "Should contain {:?} resource", &post_categories);
  ///
  /// // Test inheritance
  /// assert!(acl.inherits_resource(&post, &term),
  ///   "{:?} should have `child -> parent` relationship`with {:?}", &post, &term);
  /// assert!(acl.inherits_resource(&post_categories, &term),
  ///   "{:?} should have `child -> parent` relationship`with {:?}", &post_categories, &term);
  /// # Ok::<(), String>(())
  /// ```
  pub fn add_resource(&mut self, resource: &str, parents: Option<&[&str]>) -> Result<&mut Self, String> {
    if let Some(parents) = parents {
      self._resources.add_edge(resource, parents)?;
    }
    self._resources.add_vertex(resource);
    Ok(self)
  }

  /// Returns a `bool` indicating whether Acl contains given "resource" symbol or not.
  pub fn has_resource(&self, resource: &str) -> bool {
    self._resources.contains(resource)
  }

  /// Returns a `Result` containing a boolean indicating whether `resource` inherits
  /// `inherits` (... extends it etc.). Returns `Result::Err` if any of the given vertices
  /// do not exists in the `Acl`.
  ///
  /// ```rust
  /// use std::ops::Deref;
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new() as Acl;
  /// let guest = "guest";
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  ///
  /// // Add resources, and their relationships
  /// acl.add_resource(&guest, None)?
  ///     .add_resource(&admin, Some(&[&guest]))?
  ///     .add_resource(&super_admin, Some(&[&admin]))?;
  ///
  /// // Test created relationships
  /// assert_eq!(acl.inherits_resource_safe(&guest, &admin).is_ok(), true, "result should be `Ok(...)`");
  /// assert_eq!(acl.inherits_resource_safe(&guest, &admin).unwrap(), false,
  ///   "{:?} resource should not inherit from {:?} resource",
  ///   guest,
  ///   admin
  /// );
  ///
  /// assert!(acl.inherits_resource_safe(&admin, &guest).unwrap(), "\"admin\" resource should inherit \"guest\" resource");
  /// assert!(acl.inherits_resource_safe(&super_admin, &guest).unwrap(), "\"super_admin\" resource should inherit \"guess\" resource");
  /// # Ok::<(), String>(())
  /// ```
  /// @todo Remove '*_safe' suffix.
  pub fn inherits_resource_safe(&self, resource: &str, inherits: &str) -> Result<bool, String> {
    if let Some((v1, v2)) = self
      ._resources
      .index(resource)
      .zip(self._resources.index(inherits))
    {
      return DirectedPathsDFS::new(self._resources.graph(), v1).and_then(|dfs| dfs.marked(v2));
    }
    Err(format!("{} is not in symbol graph", inherits))
  }

  /// Returns a boolean indicating whether `resource` inherits `inherits` (... extends it etc.).
  /// Note: This method panics if `resource`, and/or `inherits`, don't exist in the ACL;
  /// For non "panic" version use `#Acl.inherits_resource_safe`.
  ///
  /// ```rust
  /// use std::ops::Deref;
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new();
  /// let guest = "guest";
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  ///
  /// // Add resources, and their relationships
  /// acl.add_resource(&guest, None)?
  ///     .add_resource(&admin, Some(&[&guest]))?
  ///     .add_resource(&super_admin, Some(&[&admin]))?;
  ///
  /// // Test created relationships
  /// assert_eq!(acl.inherits_resource(&guest, &admin), false,
  ///   "{:?} resource should not inherit from {:?} resource",
  ///   guest,
  ///   admin
  /// );
  ///
  /// assert!(acl.inherits_resource(&admin, &guest), "\"admin\" resource should inherit \"guest\" resource");
  /// assert!(acl.inherits_resource(&super_admin, &guest), "\"super_admin\" resource should inherit \"guess\" resource");
  /// # Ok::<(), String>(())
  /// ```
  pub fn inherits_resource(&self, resource: &str, inherits: &str) -> bool {
    match self.inherits_resource_safe(resource, inherits) {
      Ok(is_inherited) => is_inherited,
      Err(err) => panic!("{}", err),
    }
  }

  pub fn check_roles_for_cycles(&self) -> Result<(), String> {
    if let Some(cycles) = DirectedCycle::new(self._roles.graph()).cycle() {
      let cycles_repr = self._roles.names(cycles).unwrap()
          .join(" <- ");
      return Err(format!("Acl contains cyclic edges in \"roles\" graph: {:?}", cycles_repr));
    }
    Ok(())
  }

  pub fn check_resources_for_cycles(&self) -> Result<(), String> {
    if let Some(cycles) = DirectedCycle::new(self._resources.graph()).cycle() {
      let cycles_repr = self._resources.names(cycles).unwrap()
          .join(" <- ");
      return Err(format!("Acl contains cycles in 'resources' graph: {:?}", cycles_repr));
    }
    Ok(())
  }

  pub fn check_for_cycles(&self) -> Result<(), String> {
    self.check_roles_for_cycles()?;
    self.check_resources_for_cycles()?;
    Ok(())
  }

  /// Sets the 'allow' rule for given roles, resources, and/or, privileges, combinations; E.g.,
  ///
  /// ```rust
  /// use std::ops::Deref;
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new();
  ///
  ///  // Roles
  /// let guest_role = "guest";
  /// let user_role = "user"; // will inherit from "guest"
  /// let admin_role = "admin"; // will inherit from "user"
  ///
  /// // Resources
  /// let index_resource = "index"; // guest can access
  /// let blog_resource = "post"; // ""
  /// let account_resource = "account"; // user can access
  /// let users_resource = "users"; // admin can access
  ///
  /// // Privileges - For this example, assume these can exist for any `resource`.
  /// let index_privilege = "index";
  /// let create_privilege = "create";
  /// let read_privilege = "read";
  /// let update_privilege = "update";
  /// let delete_privilege = "delete";
  ///
  /// // Add Roles
  /// // ----
  /// acl
  ///   .add_role(guest_role, None)? // Inherits from none.
  ///   .add_role(user_role, Some(&[guest_role]))? // 'user' role inherits rules applied to 'guest' role
  ///   .add_role(admin_role, Some(&[user_role]))? // ...
  ///
  ///   // Add Resources
  ///   // ----
  ///   .add_resource(index_resource, None)? // 'index' resource has inherits from none.
  ///   .add_resource(blog_resource, Some(&[index_resource]))? // 'blog' resource inherits rules applied to 'index' resource
  ///   .add_resource(account_resource, None)?
  ///   .add_resource(users_resource, None)?
  ///
  ///   // Add 'allow' rules - **Note:** base rule is "deny all to all", E.g.,
  ///   // "deny all privileges to all roles on all resources" etc.
  ///   .allow(Some(&[guest_role]), Some(&[index_resource, blog_resource]), Some(&[index_privilege, read_privilege]))
  ///   .allow(Some(&[user_role]), Some(&[account_resource]), Some(&[index_privilege, read_privilege, update_privilege]))
  ///   .allow(Some(&[user_role]), Some(&[blog_resource]), None)
  ///   .allow(Some(&[admin_role]), None, None)  // Here we give 'admin' role all privileges to all resources
  ///   // ...
  /// ;
  ///
  /// // Check privileges
  /// // ----
  /// assert_eq!(acl.is_allowed(Some(guest_role), None, None), false,
  ///     "\"{}\" role should not have access to all privileges on all resources",
  ///     guest_role
  /// );
  /// assert_eq!(acl.is_allowed(Some(guest_role), Some(index_resource), None), false,
  ///     "\"{}\" role should not have access to all privileges on \"{}\" resource",
  ///     guest_role, index_resource
  /// );
  /// assert_eq!(acl.is_allowed(Some(guest_role), Some(index_resource), Some(index_privilege)), true,
  ///     "\"{}\" role should have \"{}\" privilege on \"{}\" resource", guest_role, index_privilege, index_resource
  /// );
  ///
  /// // Since 'user' role inherits from 'guest' role, it should have access to the same resources/privileges as 'guest' role.
  /// // ----
  /// assert_eq!(acl.inherits_role(user_role, guest_role), true,
  ///     "\"{}\" role should inherit from \"{}\" role", user_role, guest_role
  /// );
  /// assert_eq!(acl.is_allowed(Some(user_role), Some(index_resource), Some(index_privilege)), true,
  ///     "\"{}\" role should have privilege \"{}\" on \"{}\" resource",
  ///     user_role, index_privilege, index_resource
  /// );
  ///
  /// // 'user' role should have required access to 'account' resource
  /// for privilege in [index_privilege, read_privilege, update_privilege] {
  ///     assert_eq!(acl.is_allowed(Some(user_role), Some(account_resource), Some(privilege)), true,
  ///         "\"{}\" role should have privilege \"{}\" on \"{}\" resource",
  ///         user_role, index_privilege, index_resource
  ///     );
  /// }
  ///
  /// // Our 'admin' role should have access to all privileges on all resources
  /// assert_eq!(acl.is_allowed(Some(admin_role), None, None), true,
  ///     "\"{}\" role should have all privileges to all resources",
  ///     admin_role
  /// );
  ///
  /// // And lastly non-existent, roles, and/or resources, should have no privileges
  /// // ----
  /// assert_eq!(acl.is_allowed(Some("non-existent"), None, None), false,
  ///     "\"{}\" role should not have any privileges",
  ///     "non-existent"
  /// );
  /// assert_eq!(acl.is_allowed(None, Some("non-existent"), None), false,
  ///     "All privileges on \"{}\" resource should not be allowed, for all roles",
  ///     "non-existent"
  /// );
  /// assert_eq!(acl.is_allowed(None, None, Some("non-existent")), false,
  ///     "Privilege \"{}\" should not be allowed for roles, on all resource",
  ///     "non-existent"
  /// );
  /// # Ok::<(), String>(())
  /// ```
  ///
  /// ## On `None`, and/or empty list, argument values
  ///
  /// - If `privileges` is `None`, or an empty list, rule is set "for all privileges", on given roles.
  /// - If `roles` is `None`, or an empty list, rule is set "for all roles", on given resources.
  /// - If `resources` is `None`, or an empty list, rule is set "for all resources" on the acl.
  ///
  pub fn allow(
    &mut self,
    roles: Option<&[&str]>,
    resources: Option<&[&str]>,
    privileges: Option<&[&str]>,
  ) -> &mut Self {
    self._add_rule(Rule::Allow, roles, resources, privileges)
  }

  /// Sets `Deny` rule for given `roles`, `resources`, and `privileges`, combinations.
  ///
  /// ```rust
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new();
  ///
  /// // Setup roles and resources
  /// acl.add_role("guest", None)?;
  /// acl.add_role("admin", None)?;
  /// acl.add_resource("blog", None)?;
  /// acl.add_resource("secret", None)?;
  ///
  /// // Allow guest to read blog
  /// acl.allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]));
  /// assert!(acl.is_allowed(Some("guest"), Some("blog"), Some("read")));
  ///
  /// // Explicitly deny guest from accessing secret resource
  /// acl.deny(Some(&["guest"]), Some(&["secret"]), None);
  /// assert!(!acl.is_allowed(Some("guest"), Some("secret"), Some("read")));
  /// assert!(!acl.is_allowed(Some("guest"), Some("secret"), None));
  ///
  /// // Deny admin from deleting blog
  /// acl.deny(Some(&["admin"]), Some(&["blog"]), Some(&["delete"]));
  /// assert!(!acl.is_allowed(Some("admin"), Some("blog"), Some("delete")));
  /// # Ok::<(), String>(())
  /// ```
  pub fn deny(
    &mut self,
    roles: Option<&[&str]>,
    resources: Option<&[&str]>,
    privileges: Option<&[&str]>,
  ) -> &mut Self {
    self._add_rule(Rule::Deny, roles, resources, privileges)
  }

  /// Returns a boolean indicating whether given role is allowed access to given privilege on given resource.
  /// If any of the args are `None` the "all" variant is checked for that `None` value;  E.g.,
  ///
  /// ```rust
  /// use walrs_acl::{ simple::Acl };
  ///
  /// // Acl struct
  /// let mut acl = Acl::new();
  ///
  /// // Roles
  /// let guest = "guest";
  /// let user = "user";
  /// let special = "special";
  /// let admin = "admin";
  ///
  /// acl.add_roles(&[
  ///   (guest, None),
  ///   (user, Some(&[guest])),
  ///   (special, None),
  ///   (admin, Some(&[user, special]))
  /// ]);
  ///
  /// // Resources
  /// let index = "index";
  /// let protected = "protected";
  ///
  /// acl.add_resource(index, None);
  /// acl.add_resource(protected, None);
  ///
  /// // Privilege
  /// let read = "read";
  ///
  /// // All access is "denied" by default
  /// for role in [guest, user, special, admin] {
  ///   assert_eq!(
  ///     acl.is_allowed(Some(role), Some(index), None), // Checks "all privileges" access on "index"
  ///     false,
  ///     "\"{}\" role should not have privileges on \"index\" resource",
  ///     role
  ///   );
  /// }
  ///
  /// // Add "read" privilege for "guest" role, on "index" resource
  /// acl.allow(Some(&[guest]), Some(&[index]), Some(&[read]));
  /// // Perform check
  /// assert_eq!(acl.is_allowed(Some(guest), Some(index), Some(read)), true, "Has \"read\" privilege on \"index\"");
  ///
  /// // Add "all privileges" for "user", on "index" resource
  /// acl.allow(Some(&[user]), Some(&[index]), None);
  ///
  /// // Checks
  /// assert!(acl.is_allowed(Some(user), Some(index), None));
  /// assert!(acl.is_allowed(Some(admin), Some(index), None)); // inherits access from "user" role
  ///
  /// // Check random resource and priv, on "admin"
  /// assert!(!acl.is_allowed(Some(admin), Some(protected), Some("GET")));
  ///
  /// // Add "all privileges" for "admin", on all resources
  /// acl.allow(Some(&[admin]), None, None);
  ///
  /// // Checks
  /// assert!(acl.is_allowed(Some(admin), Some(index), Some(read)), "Should have \"read\" privilege on \"index\"");
  /// assert!(acl.is_allowed(Some(admin), Some(index), None), "Should have all privileges on \"index\"");
  /// assert!(acl.is_allowed(Some(admin), Some(protected), Some("GET")));
  /// assert!(acl.is_allowed(Some(admin), Some(protected), Some("POST")));
  /// assert!(acl.is_allowed(Some(admin), Some(protected), Some("PUT")));
  /// assert!(acl.is_allowed(Some(admin), Some(protected), None));
  ///
  /// // "special" checks
  /// assert_eq!(acl.is_allowed(Some(special), Some(index), Some(read)), false, "Should not \"read\" privileges on \"index\"");
  /// assert_eq!(acl.is_allowed(Some(special), Some(index), None), false, "Should not have any privileges on \"index\"");
  ///
  /// acl.allow(Some(&[special]), Some(&[index]), Some(&["report"]));
  ///
  /// // Checks
  /// assert!(acl.is_allowed(Some(special), Some(index), Some("report")), "Should have \"report\" privilege on \"index\"");
  /// ```
  pub fn is_allowed(
    &self,
    role: Option<&str>,
    resource: Option<&str>,
    privilege: Option<&str>,
  ) -> bool {
    // Select given `role`'s inherited symbols lists
    let _roles = role
      .and_then(|_role| self._roles.adj(_role))
      .and_then(|xs| if xs.is_empty() { None } else { Some(xs) });

    // Select given `resource`'s inherited symbols list
    let _resources = resource
      .and_then(|_resource| self._resources.adj(_resource))
      .and_then(|xs| if xs.is_empty() { None } else { Some(xs) });

    // Callback for returning `allow` check result, or checking if current parameter set has `allow` permission
    //  Helps dry up the code, below, a bit
    let rslt_or_check_direct = |rslt| {
      if rslt {
        rslt
      } else {
        self._is_directly_allowed(role, resource, privilege)
      }
    };

    // println!("Inherited roles and resources {:?}, {:?}", &_roles, &_resources);

    // If inherited `resources`, and `roles`, found loop through them and check for `Allow` rule
    _resources
      .as_ref()
      .zip(_roles.as_ref())
      .map(|(_resources, _roles2)| {
        _resources.iter().rev().any(|_resource| {
          _roles2
            .iter()
            .rev()
            .any(|_role| self._is_directly_allowed(Some(_role), Some(_resource), privilege))
        })
      })
      // If no inherited roles/resources directly allowed check direct allow on incoming (role, resource, privilege)
      .map(rslt_or_check_direct)
      // If only `roles`, only `resources`, or neither of the two, check for `Allow` rule
      .or_else(|| {
        // If only `roles`
        if _resources.is_none() && _roles.is_some() {
          _roles
            .map(|_rs| {
              _rs
                .iter()
                .rev()
                .any(|r| self._is_directly_allowed(Some(r), resource, privilege))
            })
            .map(rslt_or_check_direct)
        }
        // Else inherited resources is set, but not inherited roles
        else if _resources.is_some() && _roles.is_none() {
          _resources
            .map(|_rs| {
              _rs
                .iter()
                .rev()
                .any(|r| self._is_directly_allowed(role, Some(*r), privilege))
            })
            .map(rslt_or_check_direct)
        } else {
          self._is_directly_allowed(role, resource, privilege).into()
        }
      })
      .unwrap()
  }

  /// Same as `is_allowed` but checks all given role, resource, and privilege, combinations
  ///  for a match. Returns `true` if any of the combinations are allowed, `false` otherwise.
  ///
  /// This method is useful when you want to check if a user with any of several roles has access
  /// to any of several resources with any of several privileges. It short-circuits on the first
  /// allowed combination.
  ///
  /// ```rust
  /// use walrs_acl::{ simple::Acl };
  ///
  /// let mut acl = Acl::new();
  ///
  /// // Define roles
  /// let guest = "guest";
  /// let user = "user";
  /// let admin = "admin";
  ///
  /// // Add roles with inheritance: admin -> user -> guest
  /// acl.add_roles(&[
  ///   (guest, None),
  ///   (user, Some(&[guest])),
  ///   (admin, Some(&[user]))
  /// ])?;
  ///
  /// // Define resources
  /// let blog = "blog";
  /// let account = "account";
  /// let admin_panel = "admin-panel";
  ///
  /// acl.add_resource(blog, None)?;
  /// acl.add_resource(account, None)?;
  /// acl.add_resource(admin_panel, None)?;
  ///
  /// // Define privileges
  /// let read = "read";
  /// let write = "write";
  /// let delete = "delete";
  ///
  /// // Set up permissions
  /// // Guest can read blog
  /// acl.allow(Some(&[guest]), Some(&[blog]), Some(&[read]));
  ///
  /// // User can write to blog and account
  /// acl.allow(Some(&[user]), Some(&[blog, account]), Some(&[write]));
  ///
  /// // Admin has delete privilege on admin-panel
  /// acl.allow(Some(&[admin]), Some(&[admin_panel]), Some(&[delete]));
  ///
  /// // Test 1: Check if any of the guest/user roles can read blog (should be true - guest can)
  /// assert!(
  ///   acl.is_allowed_any(Some(&[guest, user]), Some(&[blog]), Some(&[read])),
  ///   "Guest or user should be able to read blog"
  /// );
  ///
  /// // Test 2: Check if user can do read OR write on blog (should be true - user can write)
  /// assert!(
  ///   acl.is_allowed_any(Some(&[user]), Some(&[blog]), Some(&[read, write])),
  ///   "User should have read or write privilege on blog"
  /// );
  ///
  /// // Test 3: Check if admin can access any of blog/account/admin-panel with any privilege
  /// // (should be true - admin inherits blog/account write, plus has admin-panel delete)
  /// assert!(
  ///   acl.is_allowed_any(Some(&[admin]), Some(&[blog, account, admin_panel]), Some(&[read, write, delete])),
  ///   "Admin should have some access to the resources"
  /// );
  ///
  /// // Test 4: Check if guest can delete anything (should be false)
  /// assert!(
  ///   !acl.is_allowed_any(Some(&[guest]), Some(&[blog, account, admin_panel]), Some(&[delete])),
  ///   "Guest should not have delete privilege on any resource"
  /// );
  ///
  /// // Test 5: Check non-existent combinations (should be false)
  /// assert!(
  ///   !acl.is_allowed_any(Some(&[guest]), Some(&[admin_panel]), Some(&[read])),
  ///   "Guest should not have access to admin-panel"
  /// );
  ///
  /// // Test 6: Empty arrays (should be false - no combinations to check)
  /// assert!(
  ///   !acl.is_allowed_any(Some(&[]), Some(&[blog]), Some(&[read])),
  ///   "Empty roles array should return false"
  /// );
  ///
  /// // Test 7: Multiple roles where only one has access
  /// assert!(
  ///   acl.is_allowed_any(Some(&[guest, admin]), Some(&[admin_panel]), Some(&[delete])),
  ///   "Admin (one of the roles) should have delete privilege on admin-panel"
  /// );
  ///
  /// // Test 8: Check with None privilege (any privilege)
  /// acl.allow(Some(&[user]), Some(&[account]), None); // Give user all privileges on account
  /// assert!(
  ///   acl.is_allowed_any(Some(&[user]), Some(&[account]), None),
  ///   "User should have any privilege on account"
  /// );
  /// # Ok::<(), String>(())
  /// ```
  pub fn is_allowed_any(
    &self,
    roles: Option<&[&str]>,
    resources: Option<&[&str]>,
    privileges: Option<&[&str]>,
  ) -> bool {
    for resource in
      self._filter_vec_option_to_options_vec1(&|xs: &str| self.has_resource(xs), resources)
    {
      for role in self._filter_vec_option_to_options_vec1(&|xs: &str| self.has_role(xs), roles) {
        for privilege in self._filter_vec_option_to_options_vec1(&|_| true, privileges) {
          if self.is_allowed(role, resource, privilege) {
            return true;
          }
        }
      }
    }
    false
  }

  /// Filters resources, roles, and/or privileges, against passed in predicate.
  /// **Note:** `vec![None]` is returned if passed in list is empty; allows `do ... while` format for `for` loops;  E.g.
  ///  "Loop while items", even though item is `None` etc..
  fn _filter_vec_option_to_options_vec1<'a>(
    &self,
    pred: &dyn Fn(&str) -> bool,
    xss: Option<&[&'a str]>,
  ) -> Vec<Option<&'a str>> {
    xss.map_or(vec![None], |_xss| {
      if _xss.is_empty() {
        return vec![None];
      }
      _xss
        .iter()
        .filter(|xs| pred(xs))
        .map(|xs| Some(*xs))
        .collect()
    })
  }

  /// Filters strings list against ones contained in given `graph`.
  /// **Note:** `vec![None]` if passed in list is empty; allows `do ... while` format for `for` loops;  E.g.
  ///  "Loop while items", even though item is `None` etc..
  fn _get_keys_in_graph<'a>(
    &self,
    graph: &'a DisymGraph,
    xss: Option<&[&'a str]>,
  ) -> Vec<Option<String>> {
    xss.map_or(vec![None], |_xss| {
      if _xss.is_empty() {
        return vec![None];
      }
      _xss
        .iter()
        .filter_map(|xs| graph.index((*xs).as_ref()).map(|idx| graph.name(idx)))
        .collect()
    })
  }

  /// Returns a boolean indicating whether the given rule is allowed, or not -
  /// Doesn't check symbol's inheritance chains; E.g., Inherited roles, and/or resources,
  /// are not checked for "allowance" (use `is_allowed(...)` for that); only 'direct' role,
  /// resource, and privilege, combination is checked.
  fn _is_directly_allowed(
    &self,
    role: Option<&str>,
    resource: Option<&str>,
    privilege: Option<&str>,
  ) -> bool {
    self
      ._rules
      .get_role_privilege_rules(resource)
      .get_privilege_rules(role)
      .get_rule(privilege)
      == &Rule::Allow
  }

  /// Gets mutable privilege rules for given `role`, and `resource` combination -
  /// If privilege rule struct doesn't exist, for given `role` and `resource` combination, one
  /// is created, inserted (for current symbol combination) and returned.
  fn _get_role_rules_mut(
    &mut self,
    resource: Option<&str>,
    role: Option<&str>,
  ) -> &mut PrivilegeRules {
    if resource.is_none() && role.is_none() {
      &mut self._rules.for_all_resources.for_all_roles
    } else if resource.is_some() && role.is_none() {
      &mut self
        ._rules
        .by_resource_id
        .entry(resource.unwrap().to_string())
        .or_insert(RolePrivilegeRules::new(false))
        .for_all_roles
    } else if resource.is_none() && role.is_some() {
      self
        ._rules
        .for_all_resources
        .by_role_id
        .get_or_insert(HashMap::new())
        .entry(role.unwrap().to_string())
        .or_insert(PrivilegeRules::new(false))
    } else {
      resource
        .zip(role)
        .map(|(_resource, _role)| {
          self
            ._rules
            .by_resource_id
            .entry(_resource.to_string())
            .or_insert(RolePrivilegeRules::new(false))
            .by_role_id
            .get_or_insert(HashMap::new())
            .entry(_role.to_string())
            .or_insert(PrivilegeRules::new(false))
        })
        .unwrap()
    }
  }

  /// Adds rule for given roles, resources, and privileges, combinations.
  fn _add_rule<'a>(
    &mut self,
    rule_type: Rule,
    roles: Option<&[&'a str]>,
    resources: Option<&[&'a str]>,
    privileges: Option<&[&'a str]>,
  ) -> &mut Self {
    // Filter out non-existent roles, and return `vec![None]` if result is empty list, or `None`.
    let _roles: Vec<Option<String>> = self._get_keys_in_graph(&self._roles, roles);

    // Filter out non-existent resources, and return `vec![None]` if result is empty list, or `None`
    let _resources: Vec<Option<String>> = self._get_keys_in_graph(&self._resources, resources);

    for resource in _resources.iter() {
      for role in _roles.iter() {
        // If role_rules found
        let role_rules = self._get_role_rules_mut(resource.as_deref(), role.as_deref());

        // If privileges is `None`, set rule type for "all privileges"
        if privileges.is_none() {
          role_rules.for_all_privileges = rule_type;
          continue;
        }
        // Else loop through privileges, and insert rule type for each privilege
        privileges.unwrap().iter().for_each(|privilege| {
          // Get privilege map for role and insert rule
          let p_map = role_rules.by_privilege_id.get_or_insert(HashMap::new());

          // Insert rule
          p_map.insert(privilege.to_string(), rule_type);
        });
      }
    }
    self
  }
}

impl Default for Acl {
  fn default() -> Self {
    Self::new()
  }
}

impl<'a> TryFrom<&'a AclData> for Acl {
  type Error = String;

  fn try_from(data: &'a AclData) -> Result<Self, Self::Error> {
    let mut acl: Acl = Acl::new();

    // Add `roles` to `acl`
    if let Some(roles) = data.roles.as_ref() {
      // Loop through role entries
      for (role, parents) in roles.iter() {
        // Convert `parents` to `Option<&[&str]>`
        let parents = parents
          .as_deref()
          .map(|xs| -> Vec<&str> { xs.iter().map(|x: &String| x.as_str()).collect() });

        // Add role(s);  If parent roles aren't in the acl, they get added via `acl.add_role`
        acl.add_role(role, parents.as_deref())?;
      }
      acl.check_roles_for_cycles()?;
    }

    // Add `resources` to `acl`
    if let Some(resources) = data.resources.as_ref() {
      // Loop through resource entries
      for (resource, parents) in resources.iter() {
        // Convert `parents` to `Option<&[&str]>`
        let parents = parents
          .as_deref()
          .map(|xs| -> Vec<&str> { xs.iter().map(|x: &String| x.as_str()).collect() });

        // Add resource(s);  If parent resources aren't in the acl, they get added via `acl.add_resource`
        acl.add_resource(resource, parents.as_deref())?;
      }
      acl.check_resources_for_cycles()?;
    }

    // Add `allow` rules to `acl`, if any
    if let Some(allow) = data.allow.as_ref() {
      // For entry in allow rules
      allow
        .iter()
        .for_each(|(resource, roles_and_privileges_assoc_list)| {
          // If `(roles, privileges)` associative list loop through it`
          if let Some(rs_and_ps_list) = roles_and_privileges_assoc_list {
            // For each entry in `role -> privilege` list
            rs_and_ps_list.iter().for_each(|(role, privileges)| {
              let ps: Option<Vec<&str>> = privileges
                .as_deref()
                .map(|ps| ps.iter().map(|p| &**p).collect());
              // Apply `allow` rule
              acl.allow(
                Some([role.as_str()].as_slice()),
                Some([resource.as_str()].as_slice()),
                ps.as_deref(),
              );
            });
          }
          // Else add allow rule for all `roles`, on all `privileges`, for given `resource`
          else {
            acl.allow(None, Some([resource.as_str()].as_slice()), None);
          }
        });
    }

    // Add `deny` rules to `acl`, if any
    if let Some(deny) = data.deny.as_ref() {
      deny
          .iter()
          .for_each(|(resource, roles_and_privileges_assoc_list)| {
            if let Some(rs_and_ps_list) = roles_and_privileges_assoc_list {
              rs_and_ps_list.iter().for_each(|(role, privileges)| {
                let ps: Option<Vec<&str>> = privileges
                    .as_deref()
                    .map(|ps| ps.iter().map(|p| &**p).collect());
                acl.deny(
                  Some([role.as_str()].as_slice()),
                  Some([resource.as_str()].as_slice()),
                  ps.as_deref(),
                );
              });
            } else {
              acl.deny(None, Some([resource.as_str()].as_slice()), None);
            }
          });
    }

    // println!("{:#?}", &acl);

    Ok(acl)
  }
}

impl TryFrom<AclData> for Acl {
  type Error = String;

  fn try_from(data: AclData) -> Result<Self, Self::Error> {
    Acl::try_from(&data)
  }
}

impl<'a> TryFrom<&'a mut File> for Acl {
  type Error = serde_json::Error;

  fn try_from(file: &mut File) -> Result<Self, Self::Error> {
    AclData::try_from(file).and_then(|data| {
      Acl::try_from(&data).map_err(|e| {
        serde_json::Error::io(
          std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        )
      })
    })
  }
}

#[cfg(test)]
mod test_acl {
  use crate::simple::acl::{Acl};
  use crate::simple::privilege_rules::PrivilegeRules;
  use crate::simple::rule::Rule;

  #[test]
  fn test_default_and_new() {
    let acl = Acl::default();
    assert_eq!(acl.has_resource("index"), false);
    assert_eq!(acl.has_role("admin"), false);
    assert_eq!(acl._rules.for_all_resources.for_all_roles.for_all_privileges, Rule::Deny);

    let acl2 = Acl::new();
    assert_eq!(acl2.has_resource("index"), false);
    assert_eq!(acl2.has_role("admin"), false);
    assert_eq!(acl2._rules.for_all_resources.for_all_roles.for_all_privileges, Rule::Deny);
  }

  #[test]
  fn test_has_resource() {
    let mut acl = Acl::new();
    let index = "index";
    let users = "users";
    let non_existent_resource = "non-existent-resource";

    // Add resources, and their relationships to the acl:
    acl.add_resource(users, Some([index].as_slice())).unwrap();

    assert!(
      acl.has_resource(index),
      "Should contain {:?} resource",
      index
    );
    assert!(
      acl.has_resource(users),
      "Should contain {:?} resource",
      users
    );
    assert_eq!(
      acl.has_resource(non_existent_resource),
      false,
      "Should \"not\" contain {:?} resource",
      non_existent_resource
    );
  }

  #[test]
  fn test_has_role() {
    let mut acl = Acl::new();
    let admin = "admin";
    let super_admin = "super_admin";
    let non_existent_role = "non-existent-role";

    // Add roles, and their relationships to the acl:
    acl.add_role(admin, Some([super_admin].as_slice())).unwrap();

    assert!(acl.has_role(admin), "Should contain {:?} role", admin);
    assert!(
      acl.has_role(super_admin),
      "Should contain {:?} role",
      super_admin
    );
    assert_eq!(
      acl.has_role(non_existent_role),
      false,
      "Should \"not\" contain {:?} role",
      non_existent_role
    );
  }

  #[test]
  fn test_get_privilege_rules_for_populated() {
    let account_index_privilege = "account-index";
    let index_privilege = "index";
    let mut privilege_rules = PrivilegeRules::new(true);

    for (privilege, expected_rule) in [
      (index_privilege, Rule::Allow),
      (account_index_privilege, Rule::Deny),
    ] {
      // Set privilege rules
      privilege_rules
        .by_privilege_id
        .as_mut()
        .and_then(|privilege_id_map| {
          privilege_id_map.insert(privilege.to_string(), expected_rule);
          Some(())
        })
        .expect("Expecting a `privilege_id_map`;  None found");

      // Test for expected (1)
      assert_eq!(
        &privilege_rules.get_rule(Some(privilege)),
        privilege_rules
          .by_privilege_id
          .as_ref()
          .unwrap()
          .get(privilege)
          .as_ref()
          .unwrap(),
        "Expected returned `RuleType` to equal {:?}",
        expected_rule
      );

      assert_eq!(
        privilege_rules.get_rule(Some(privilege)),
        &expected_rule,
        "Expected returned `RuleType` to equal `{:#?}`, for \"{:?}\"",
        expected_rule,
        privilege
      );
    }
  }

  #[test]
  fn test_acl_allow() {
    // Roles
    let guest_role = "guest";
    let user_role = "user"; // Inherits from "guest"
    let admin_role = "admin"; // Inherits from "user"

    // Resources
    let index_resource = "index"; // guest can access
    let blog_resource = "blog"; // ""
    let account_resource = "account"; // user can access
    let users_resource = "users"; // admin can access

    // Privileges
    let index_privilege = "index";
    let create_privilege = "create";
    let read_privilege = "read";
    let update_privilege = "update";
    let delete_privilege = "delete";

    let populate_acl_symbols = |acl: &mut Acl| {
      // Add Roles
      acl.add_role(guest_role, None).unwrap();
      acl.add_role(user_role, Some(&[guest_role])).unwrap();
      acl.add_role(admin_role, Some(&[user_role])).unwrap();

      // Add Resources
      acl.add_resource(index_resource, None).unwrap();
      acl.add_resource(blog_resource, Some(&[index_resource])).unwrap();
      acl.add_resource(account_resource, None).unwrap();
      acl.add_resource(users_resource, None).unwrap();
    };

    // Ensure default expected default rule is set
    assert_eq!(
      Acl::new()
        ._rules
        .for_all_resources
        .for_all_roles
        .for_all_privileges,
      Rule::Deny,
      "Expected default rule to equal `Rule::Deny`"
    );

    for (roles, resources, privileges, expected) in [
      (
        Some([guest_role].as_slice()),
        Some([index_resource].as_slice()),
        None,
        true,
      ),
      (
        Some([user_role].as_slice()),
        Some([account_resource].as_slice()),
        Some([index_privilege, update_privilege].as_slice()),
        true,
      ),
      (
        Some([admin_role].as_slice()),
        Some([users_resource, account_resource].as_slice()),
        None,
        true,
      ),
      (
        Some([admin_role].as_slice()),
        Some([users_resource, account_resource].as_slice()),
        None,
        true,
      ),
      (
        Some([admin_role].as_slice()),
        Some([users_resource, account_resource].as_slice()),
        Some(
          [
            index_privilege,
            create_privilege,
            read_privilege,
            update_privilege,
          ]
          .as_slice(),
        ),
        true,
      ),
      (
        None,
        Some([users_resource, account_resource].as_slice()),
        Some([index_privilege, read_privilege].as_slice()),
        true,
      ),
      (
        None,
        None,
        Some([index_privilege, read_privilege].as_slice()),
        true,
      ),
      (None, None, None, true),
      (None, None, None, false),
      (
        Some([admin_role].as_slice()),
        Some([users_resource, account_resource].as_slice()),
        None,
        false,
      ),
      (
        Some([admin_role].as_slice()),
        Some([users_resource, account_resource].as_slice()),
        Some(
          [
            index_privilege,
            create_privilege,
            read_privilege,
            update_privilege,
          ]
          .as_slice(),
        ),
        false,
      ),
      (
        None,
        Some([users_resource, account_resource].as_slice()),
        Some([index_privilege, read_privilege].as_slice()),
        false,
      ),
      (
        None,
        None,
        Some([index_privilege, read_privilege].as_slice()),
        false,
      ),
    ] {
      let mut acl = Acl::new();

      populate_acl_symbols(&mut acl);

      // If we're testing for 'allow' set allow rule result to test
      if expected {
        acl.allow(roles, resources, privileges);
      }

      // println!("`#Acl._rules`: {:#?}", &acl._rules);

      assert_eq!(
        acl.is_allowed_any(roles, resources, privileges),
        expected,
        "Expected `acl.is_allowed_any({:?}, {:?}, {:?}) == {}`",
        roles,
        resources,
        privileges,
        expected
      );
    }
  }

  #[test]
  fn test_acl_deny() {
    // Roles
    let guest_role = "guest";
    let user_role = "user"; // Inherits from "guest"
    let admin_role = "admin"; // Inherits from "user"

    // Resources
    let index_resource = "index"; // guest can access
    let blog_resource = "blog"; // ""
    let account_resource = "account"; // user can access
    let users_resource = "users"; // admin can access

    // Privileges
    let index_privilege = "index";
    let create_privilege = "create";
    let read_privilege = "read";
    let update_privilege = "update";
    let delete_privilege = "delete";

    let populate_acl_symbols = |acl: &mut Acl| {
      // Add Roles
      acl.add_role(guest_role, None).unwrap();
      acl.add_role(user_role, Some(&[guest_role])).unwrap();
      acl.add_role(admin_role, Some(&[user_role])).unwrap();

      // Add Resources
      acl.add_resource(index_resource, None);
      acl.add_resource(blog_resource, Some(&[index_resource]));
      acl.add_resource(account_resource, None);
      acl.add_resource(users_resource, None);
    };

    // Ensure default expected rule is set
    assert_eq!(
      Acl::new()
        ._rules
        .for_all_resources
        .for_all_roles
        .for_all_privileges,
      Rule::Deny,
      "Expected default rule to equal `Rule::Deny`"
    );

    for (roles, resources, privileges) in [
      (
        Some([guest_role].as_slice()),
        Some([index_resource].as_slice()),
        None,
      ),
      (
        Some([user_role].as_slice()),
        Some([account_resource].as_slice()),
        Some([index_privilege, update_privilege].as_slice()),
      ),
      (
        Some([admin_role].as_slice()),
        Some([users_resource, account_resource].as_slice()),
        None,
      ),
      (
        Some([admin_role].as_slice()),
        Some([users_resource, account_resource].as_slice()),
        None,
      ),
      (
        Some([admin_role].as_slice()),
        Some([users_resource, account_resource].as_slice()),
        Some(
          [
            index_privilege,
            create_privilege,
            read_privilege,
            update_privilege,
          ]
          .as_slice(),
        ),
      ),
      (
        None,
        Some([users_resource, account_resource].as_slice()),
        Some([index_privilege, read_privilege].as_slice()),
      ),
      (
        None,
        None,
        Some([index_privilege, read_privilege].as_slice()),
      ),
      (None, None, None),
    ] {
      let mut acl = Acl::new();

      populate_acl_symbols(&mut acl);

      acl.deny(roles, resources, privileges);

      // println!("`#Acl._rules`: {:#?}", &acl._rules);

      assert_eq!(
        acl.is_allowed_any(roles, resources, privileges),
        false,
        "Expected `acl.is_allowed_any({:?}, {:?}, {:?}) == {}`",
        roles,
        resources,
        privileges,
        false
      );
    }
  }

  #[test]
  fn test_acl_deny_comprehensive() -> Result<(), String> {
    let mut acl = Acl::new();

    // Define roles with inheritance
    let guest = "guest";
    let user = "user";
    let moderator = "moderator";
    let admin = "admin";

    acl.add_roles(&[
      (guest, None),
      (user, Some(&[guest])),
      (moderator, Some(&[user])),
      (admin, Some(&[moderator]))
    ])?;

    // Define resources
    let blog = "blog";
    let account = "account";
    let admin_panel = "admin-panel";
    let secret = "secret";

    acl.add_resource(blog, None)?;
    acl.add_resource(account, None)?;
    acl.add_resource(admin_panel, None)?;
    acl.add_resource(secret, None)?;

    // Define privileges
    let read = "read";
    let write = "write";
    let delete = "delete";
    let publish = "publish";

    // Test 1: Deny specific privilege on specific resource for specific role
    acl.deny(Some(&[guest]), Some(&[admin_panel]), Some(&[read]));
    assert!(
      !acl.is_allowed(Some(guest), Some(admin_panel), Some(read)),
      "Guest should be denied read access to admin-panel"
    );

    // Test 2: Deny all privileges on a resource (None for privileges)
    acl.deny(Some(&[user]), Some(&[secret]), None);
    assert!(
      !acl.is_allowed(Some(user), Some(secret), Some(read)),
      "User should be denied all access to secret resource"
    );
    assert!(
      !acl.is_allowed(Some(user), Some(secret), Some(write)),
      "User should be denied all access to secret resource"
    );
    assert!(
      !acl.is_allowed(Some(user), Some(secret), None),
      "User should be denied all access to secret resource"
    );

    // Test 3: Deny multiple privileges at once
    acl.deny(Some(&[guest]), Some(&[blog]), Some(&[write, delete, publish]));
    assert!(
      !acl.is_allowed(Some(guest), Some(blog), Some(write)),
      "Guest should be denied write on blog"
    );
    assert!(
      !acl.is_allowed(Some(guest), Some(blog), Some(delete)),
      "Guest should be denied delete on blog"
    );
    assert!(
      !acl.is_allowed(Some(guest), Some(blog), Some(publish)),
      "Guest should be denied publish on blog"
    );

    // Test 4: Deny across multiple roles
    acl.deny(Some(&[guest, user]), Some(&[admin_panel]), None);
    assert!(
      !acl.is_allowed(Some(guest), Some(admin_panel), None),
      "Guest should be denied all access to admin-panel"
    );
    assert!(
      !acl.is_allowed(Some(user), Some(admin_panel), Some(write)),
      "User should be denied all access to admin-panel"
    );

    // Test 5: Deny across multiple resources
    acl.deny(Some(&[moderator]), Some(&[secret, admin_panel]), Some(&[delete]));
    assert!(
      !acl.is_allowed(Some(moderator), Some(secret), Some(delete)),
      "Moderator should be denied delete on secret"
    );
    assert!(
      !acl.is_allowed(Some(moderator), Some(admin_panel), Some(delete)),
      "Moderator should be denied delete on admin-panel"
    );

    // Test 6: Allow and then deny for the same role (explicit deny takes precedence)
    acl.allow(Some(&[user]), Some(&[blog]), Some(&[write]));
    assert!(
      acl.is_allowed(Some(user), Some(blog), Some(write)),
      "User should be allowed to write to blog"
    );

    // Now explicitly deny user from writing to blog
    acl.deny(Some(&[user]), Some(&[blog]), Some(&[write]));
    assert!(
      !acl.is_allowed(Some(user), Some(blog), Some(write)),
      "User should now be denied write access to blog (deny overrides allow)"
    );

    // Test 7: Deny all roles on a resource (None for roles)
    acl.deny(None, Some(&[secret]), Some(&[read]));
    assert!(
      !acl.is_allowed(Some(guest), Some(secret), Some(read)),
      "All roles (including guest) should be denied read on secret"
    );
    assert!(
      !acl.is_allowed(Some(admin), Some(secret), Some(read)),
      "All roles (including admin) should be denied read on secret"
    );

    // Test 8: Deny role on all resources (None for resources)
    acl.deny(Some(&[guest]), None, Some(&[delete]));
    assert!(
      !acl.is_allowed(Some(guest), Some(blog), Some(delete)),
      "Guest should be denied delete on all resources (blog)"
    );
    assert!(
      !acl.is_allowed(Some(guest), Some(account), Some(delete)),
      "Guest should be denied delete on all resources (account)"
    );

    // Test 9: Method chaining
    acl.deny(Some(&[user]), Some(&[account]), Some(&[delete]))
       .deny(Some(&[user]), Some(&[blog]), Some(&[publish]))
       .deny(Some(&[moderator]), Some(&[secret]), None);

    assert!(!acl.is_allowed(Some(user), Some(account), Some(delete)));
    assert!(!acl.is_allowed(Some(user), Some(blog), Some(publish)));
    assert!(!acl.is_allowed(Some(moderator), Some(secret), Some(read)));

    // Test 10: Empty arrays behave like None (all)
    acl.deny(Some(&[]), Some(&[admin_panel]), Some(&[write]));
    // Empty roles array means "all roles"
    assert!(
      !acl.is_allowed(Some(admin), Some(admin_panel), Some(write)),
      "Empty roles array should deny all roles"
    );

    Ok(())
  }

  #[test]
  #[should_panic(expected = "d is not in symbol graph")]
  fn test_inherits_role() {
    let mut acl = Acl::new();
    acl.add_role("a", Some(["b", "c"].as_slice())).unwrap();
    assert!(acl.inherits_role("a", "b"));
    assert!(acl.inherits_role("a", "c"));
    assert!(acl.inherits_role("a", "d"));
  }

  #[test]
  #[should_panic(expected = "d is not in symbol graph")]
  fn test_inherits_resource() {
    let mut acl = Acl::new();
    acl.add_resource("a", Some(["b", "c"].as_slice()));
    assert!(acl.inherits_resource("a", "b"));
    assert!(acl.inherits_resource("a", "c"));
    assert!(acl.inherits_resource("a", "d"));
  }

  // ============================
  // Tests for add_roles
  // ============================

  #[test]
  fn test_add_roles_basic() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add multiple roles without parents
    acl.add_roles(&[
      ("guest", None),
      ("user", None),
      ("admin", None),
    ])?;

    // Verify all roles were added
    assert!(acl.has_role("guest"), "ACL should contain 'guest' role");
    assert!(acl.has_role("user"), "ACL should contain 'user' role");
    assert!(acl.has_role("admin"), "ACL should contain 'admin' role");
    assert_eq!(acl.role_count(), 3, "ACL should contain exactly 3 roles");

    Ok(())
  }

  #[test]
  fn test_add_roles_with_single_parent() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add roles with parent relationships
    acl.add_roles(&[
      ("guest", None),
      ("user", Some(&["guest"])),
      ("admin", Some(&["user"])),
    ])?;

    // Verify all roles were added
    assert!(acl.has_role("guest"), "ACL should contain 'guest' role");
    assert!(acl.has_role("user"), "ACL should contain 'user' role");
    assert!(acl.has_role("admin"), "ACL should contain 'admin' role");

    // Verify inheritance relationships
    assert!(acl.inherits_role("user", "guest"), "user should inherit from guest");
    assert!(acl.inherits_role("admin", "user"), "admin should inherit from user");
    assert!(acl.inherits_role("admin", "guest"), "admin should transitively inherit from guest");

    Ok(())
  }

  #[test]
  fn test_add_roles_with_multiple_parents() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add roles with multiple parent relationships
    acl.add_roles(&[
      ("viewer", None),
      ("editor", None),
      ("moderator", None),
      ("admin", Some(&["editor", "moderator"])),
      ("super-admin", Some(&["admin", "viewer"])),
    ])?;

    // Verify all roles were added
    assert!(acl.has_role("viewer"), "ACL should contain 'viewer' role");
    assert!(acl.has_role("editor"), "ACL should contain 'editor' role");
    assert!(acl.has_role("moderator"), "ACL should contain 'moderator' role");
    assert!(acl.has_role("admin"), "ACL should contain 'admin' role");
    assert!(acl.has_role("super-admin"), "ACL should contain 'super-admin' role");
    assert_eq!(acl.role_count(), 5, "ACL should contain exactly 5 roles");

    // Verify inheritance relationships
    assert!(acl.inherits_role("admin", "editor"), "admin should inherit from editor");
    assert!(acl.inherits_role("admin", "moderator"), "admin should inherit from moderator");
    assert!(acl.inherits_role("super-admin", "admin"), "super-admin should inherit from admin");
    assert!(acl.inherits_role("super-admin", "viewer"), "super-admin should inherit from viewer");

    // Verify transitive inheritance
    assert!(acl.inherits_role("super-admin", "editor"), "super-admin should transitively inherit from editor");
    assert!(acl.inherits_role("super-admin", "moderator"), "super-admin should transitively inherit from moderator");

    Ok(())
  }

  #[test]
  fn test_add_roles_empty_list() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add empty list of roles - should succeed
    acl.add_roles(&[])?;

    assert_eq!(acl.role_count(), 0, "ACL should contain 0 roles");

    Ok(())
  }

  #[test]
  fn test_add_roles_chaining() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Test method chaining
    acl.add_roles(&[
      ("guest", None),
      ("user", Some(&["guest"])),
    ])?
        .add_roles(&[
          ("admin", Some(&["user"])),
          ("super-admin", Some(&["admin"])),
        ])?;

    // Verify all roles were added
    assert_eq!(acl.role_count(), 4, "ACL should contain exactly 4 roles");
    assert!(acl.has_role("guest"), "ACL should contain 'guest' role");
    assert!(acl.has_role("user"), "ACL should contain 'user' role");
    assert!(acl.has_role("admin"), "ACL should contain 'admin' role");
    assert!(acl.has_role("super-admin"), "ACL should contain 'super-admin' role");

    // Verify inheritance chain
    assert!(acl.inherits_role("super-admin", "admin"), "super-admin should inherit from admin");
    assert!(acl.inherits_role("super-admin", "user"), "super-admin should transitively inherit from user");
    assert!(acl.inherits_role("super-admin", "guest"), "super-admin should transitively inherit from guest");

    Ok(())
  }

  #[test]
  fn test_add_roles_duplicate_roles() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add roles first time
    acl.add_roles(&[
      ("guest", None),
      ("user", Some(&["guest"])),
    ])?;

    // Attempt adding same roles again - should succeed (idempotent behavior;
    //  E.g., roles are only added once):
    acl.add_roles(&[
      ("guest", None),
      ("user", Some(&["guest"])),
    ])?;

    // Verify roles exist and count is still correct
    assert!(acl.has_role("guest"), "ACL should contain 'guest' role");
    assert!(acl.has_role("user"), "ACL should contain 'user' role");
    assert_eq!(acl.role_count(), 2);

    Ok(())
  }

  #[test]
  fn test_add_roles_with_nonexistent_parent() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add a role with a parent that doesn't exist yet
    // The system automatically creates the parent role (first)
    acl.add_roles(&[
      ("user", Some(&["nonexistent-parent"])),
    ])?;

    // Both the role and its parent should now exist
    assert!(acl.has_role("user"), "ACL should contain 'user' role");
    assert!(acl.has_role("nonexistent-parent"), "ACL should automatically create 'nonexistent-parent' role");
    assert!(acl.inherits_role("user", "nonexistent-parent"), "user should inherit from nonexistent-parent");

    // Assert role count
    assert_eq!(acl.role_count(), 2, "ACL should contain exactly 2 roles");

    Ok(())
  }

  #[test]
  fn test_add_roles_mixed_with_and_without_parents() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add a mix of roles with and without parents
    acl.add_roles(&[
      ("guest", None),
      ("special", None),
      ("user", Some(&["guest"])),
      ("moderator", None),
      ("admin", Some(&["user", "moderator"])),
    ])?;

    // Verify all roles exist
    assert_eq!(acl.role_count(), 5, "ACL should contain exactly 5 roles");

    // Verify specific roles
    assert!(acl.has_role("guest"), "ACL should contain 'guest' role");
    assert!(acl.has_role("special"), "ACL should contain 'special' role");
    assert!(acl.has_role("user"), "ACL should contain 'user' role");
    assert!(acl.has_role("moderator"), "ACL should contain 'moderator' role");
    assert!(acl.has_role("admin"), "ACL should contain 'admin' role");

    // Verify inheritance
    assert!(acl.inherits_role("user", "guest"), "user should inherit from guest");
    assert!(acl.inherits_role("admin", "user"), "admin should inherit from user");
    assert!(acl.inherits_role("admin", "moderator"), "admin should inherit from moderator");

    // Verify non-inheritance
    assert!(!acl.inherits_role("special", "guest"), "special should not inherit from guest");
    assert!(!acl.inherits_role("moderator", "guest"), "moderator should not inherit from guest");

    Ok(())
  }

  #[test]
  fn test_add_roles_out_of_order_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add roles in "reverse" order - children before parents
    // This works because disymgraph auto-creates parent vertices (first)
    acl.add_roles(&[
      ("super-admin", Some(&["admin"])),  // admin doesn't exist yet
      ("admin", Some(&["user"])),         // user doesn't exist yet
      ("user", Some(&["guest"])),         // guest doesn't exist yet
      ("guest", None),                    // finally add the base role
    ])?;

    // All roles should exist
    assert_eq!(acl.role_count(), 4, "ACL should contain exactly 4 roles");
    assert!(acl.has_role("guest"), "ACL should contain 'guest' role");
    assert!(acl.has_role("user"), "ACL should contain 'user' role");
    assert!(acl.has_role("admin"), "ACL should contain 'admin' role");
    assert!(acl.has_role("super-admin"), "ACL should contain 'super-admin' role");

    // Verify inheritance chain works correctly
    assert!(acl.inherits_role("user", "guest"), "user should inherit from guest");
    assert!(acl.inherits_role("admin", "user"), "admin should inherit from user");
    assert!(acl.inherits_role("super-admin", "admin"), "super-admin should inherit from admin");

    // Verify transitive inheritance
    assert!(acl.inherits_role("super-admin", "guest"), "super-admin should transitively inherit from guest");

    Ok(())
  }

  #[test]
  fn test_add_roles_self_reference_in_parents() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Try to add a role that references itself as a parent
    // This creates a self-loop which would be a cycle
    acl.add_roles(&[
      ("recursive-role", Some(&["recursive-role"])),
    ])?;

    // The role should exist and count should be `1`
    assert!(acl.has_role("recursive-role"), "ACL should contain 'recursive-role'");
    assert_eq!(acl.role_count(), 1);

    // Should return error as the [digraph] edge for 'recursive-role' -> 'recursive-role' will not
    // be added to the graph.
    assert!(acl.inherits_role("recursive-role", "recursive-role"),
            "recursive-role should inherit from itself (self-loop)");

    // Check for [roles] cycle
    let rslt = acl.check_roles_for_cycles();
    eprintln!("{}", rslt.as_ref().unwrap_err());
    assert!(rslt.is_err(), "ACL should contain a cycle");

    Ok(())
  }

  #[test]
  fn test_add_roles_circular_dependency() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create a circular dependency: A -> B -> C -> A
    // First add them individually
    acl.add_role("role-a", Some(&["role-b"]))?;
    acl.add_role("role-b", Some(&["role-c"]))?;

    // This would create a cycle, but the system allows it (user is expected to
    //  run `acl.check_for_cycles()` before using the structure for validation (currently)).
    // Note: In a proper ACL, cycles might be problematic for permission resolution
    acl.add_role("role-c", Some(&["role-a"]))?;

    // All roles should exist
    assert!(acl.has_role("role-a"), "ACL should contain 'role-a'");
    assert!(acl.has_role("role-b"), "ACL should contain 'role-b'");
    assert!(acl.has_role("role-c"), "ACL should contain 'role-c'");

    // Due to the cycle, each role should inherit from all others
    assert!(acl.inherits_role("role-a", "role-b"), "role-a should inherit from role-b");
    assert!(acl.inherits_role("role-b", "role-c"), "role-b should inherit from role-c");
    assert!(acl.inherits_role("role-c", "role-a"), "role-c should inherit from role-a");

    // Due to transitivity through the cycle
    assert!(acl.inherits_role("role-a", "role-c"), "role-a should inherit from role-c (via cycle)");
    assert!(acl.inherits_role("role-b", "role-a"), "role-b should inherit from role-a (via cycle)");
    assert!(acl.inherits_role("role-c", "role-b"), "role-c should inherit from role-b (via cycle)");

    // Check for [roles] cycle
    let rslt = acl.check_roles_for_cycles();
    eprintln!("{}", rslt.as_ref().unwrap_err());
    assert!(rslt.is_err(), "ACL should contain a cycle");

    Ok(())
  }

  #[test]
  fn test_add_roles_adding_parent_to_existing_role() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add a role without parents
    acl.add_role("user", None)?;
    assert_eq!(acl.role_count(), 1, "ACL should contain 1 role");

    // Now add the same role again with parents
    // This should add the parent relationship to the existing role
    acl.add_roles(&[
      ("guest", None),
      ("user", Some(&["guest"])),
    ])?;

    // Verify both roles exist (role count should be 2, not 3)
    assert_eq!(acl.role_count(), 2, "ACL should contain exactly 2 roles");
    assert!(acl.has_role("user"), "ACL should contain 'user' role");
    assert!(acl.has_role("guest"), "ACL should contain 'guest' role");

    // Verify the inheritance was added
    assert!(acl.inherits_role("user", "guest"), "user should now inherit from guest");

    Ok(())
  }

  #[test]
  fn test_add_roles_complex_diamond_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create a diamond inheritance pattern:
    //        root
    //       /    \
    //   branch-a  branch-b
    //       \    /
    //        leaf
    acl.add_roles(&[
      ("root", None),
      ("branch-a", Some(&["root"])),
      ("branch-b", Some(&["root"])),
      ("leaf", Some(&["branch-a", "branch-b"])),
    ])?;

    // Verify all roles exist
    assert_eq!(acl.role_count(), 4, "ACL should contain exactly 4 roles");

    // Verify direct inheritance
    assert!(acl.inherits_role("branch-a", "root"), "branch-a should inherit from root");
    assert!(acl.inherits_role("branch-b", "root"), "branch-b should inherit from root");
    assert!(acl.inherits_role("leaf", "branch-a"), "leaf should inherit from branch-a");
    assert!(acl.inherits_role("leaf", "branch-b"), "leaf should inherit from branch-b");

    // Verify transitive inheritance (leaf inherits from root through both branches)
    assert!(acl.inherits_role("leaf", "root"), "leaf should transitively inherit from root");

    Ok(())
  }

  // ============================
  // Tests for add_resource
  // ============================

  #[test]
  fn test_add_resource_without_parents() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add a resource without parents
    acl.add_resource("blog", None)?;

    // Verify resource was added
    assert!(acl.has_resource("blog"), "ACL should contain the 'blog' resource");
    assert_eq!(acl.resource_count(), 1, "ACL should have exactly 1 resource");

    Ok(())
  }

  #[test]
  fn test_add_resource_with_single_parent() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add parent resource first
    acl.add_resource("cms", None)?;

    // Add child resource with parent
    acl.add_resource("blog", Some(&["cms"]))?;

    // Verify both resources were added
    assert!(acl.has_resource("cms"), "ACL should contain 'cms' resource");
    assert!(acl.has_resource("blog"), "ACL should contain 'blog' resource");
    assert_eq!(acl.resource_count(), 2, "ACL should have exactly 2 resources");

    // Verify inheritance relationship
    assert!(acl.inherits_resource("blog", "cms"), "blog should inherit from cms");
    assert!(!acl.inherits_resource("cms", "blog"), "cms should not inherit from blog");

    Ok(())
  }

  #[test]
  fn test_add_resource_with_multiple_parents() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add parent resources
    acl.add_resource("readable", None)?;
    acl.add_resource("writable", None)?;

    // Add child resource with multiple parents
    acl.add_resource("document", Some(&["readable", "writable"]))?;

    // Verify all resources were added
    assert_eq!(acl.resource_count(), 3, "ACL should have exactly 3 resources");

    // Verify inheritance relationships
    assert!(acl.inherits_resource("document", "readable"), "document should inherit from readable");
    assert!(acl.inherits_resource("document", "writable"), "document should inherit from writable");

    Ok(())
  }

  #[test]
  fn test_add_resource_with_transitive_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create a hierarchy: base -> intermediate -> leaf
    acl.add_resource("base", None)?;
    acl.add_resource("intermediate", Some(&["base"]))?;
    acl.add_resource("leaf", Some(&["intermediate"]))?;

    // Verify transitive inheritance
    assert!(acl.inherits_resource("leaf", "intermediate"), "leaf should inherit from intermediate");
    assert!(acl.inherits_resource("leaf", "base"), "leaf should inherit from base (transitively)");
    assert!(acl.inherits_resource("intermediate", "base"), "intermediate should inherit from base");

    Ok(())
  }

  #[test]
  fn test_add_resource_chained_calls() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Test method chaining
    acl
        .add_resource("guest", None)?
        .add_resource("user", Some(&["guest"]))?
        .add_resource("admin", Some(&["user"]))?
        .add_resource("super-admin", Some(&["admin"]))?;

    // Verify all resources were added
    assert_eq!(acl.resource_count(), 4, "ACL should have exactly 4 resources");

    // Verify inheritance chain
    assert!(acl.inherits_resource("super-admin", "admin"), "super-admin should inherit from admin");
    assert!(acl.inherits_resource("super-admin", "user"), "super-admin should inherit from user (transitively)");
    assert!(acl.inherits_resource("super-admin", "guest"), "super-admin should inherit from guest (transitively)");

    Ok(())
  }

  #[test]
  fn test_add_resource_duplicate_without_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add same resource multiple times
    acl.add_resource("blog", None)?;
    acl.add_resource("blog", None)?;

    // Should still have only 1 unique resource
    assert_eq!(acl.resource_count(), 1, "ACL should have exactly 1 resource");

    Ok(())
  }

  // ============================
  // Tests for check_resources_for_cycles
  // ============================

  #[test]
  fn test_check_resources_for_cycles_no_cycles() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create a valid DAG structure
    acl.add_resource("guest", None)?;
    acl.add_resource("user", Some(&["guest"]))?;
    acl.add_resource("admin", Some(&["user"]))?;

    // Should not detect any cycles
    let result = acl.check_resources_for_cycles();
    assert!(result.is_ok(), "Should not detect cycles in valid DAG");

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_empty_acl() -> Result<(), Box<dyn std::error::Error>> {
    let acl = Acl::new();

    // Empty ACL should not have cycles
    let result = acl.check_resources_for_cycles();
    assert!(result.is_ok(), "Empty ACL should not have cycles");

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_single_resource() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    acl.add_resource("blog", None)?;

    // Single resource should not have cycles
    let result = acl.check_resources_for_cycles();
    assert!(result.is_ok(), "Single resource should not have cycles");

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_detects_simple_cycle() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create vertices first
    acl.add_resource("a", None)?;
    acl.add_resource("b", None)?;

    // Create a simple cycle: a -> b -> a
    acl.add_resource("a", Some(&["b"]))?;
    acl.add_resource("b", Some(&["a"]))?;

    // Should detect the cycle
    let result = acl.check_resources_for_cycles();
    assert!(result.is_err(), "Should detect simple cycle");

    if let Err(msg) = result {
      assert!(msg.contains("cycles"), "Error message should mention cycles");
      assert!(msg.contains("resources"), "Error message should mention resources");
    }

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_detects_self_cycle() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create a self-referencing resource
    acl.add_resource("self-ref", None)?;
    acl.add_resource("self-ref", Some(&["self-ref"]))?;

    // Should detect the self-cycle
    let result = acl.check_resources_for_cycles();
    assert!(result.is_err(), "Should detect self-referencing cycle");

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_detects_complex_cycle() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create a complex cycle: a -> b -> c -> d -> b
    acl.add_resource("a", None)?;
    acl.add_resource("b", None)?;
    acl.add_resource("c", None)?;
    acl.add_resource("d", None)?;

    acl.add_resource("a", Some(&["b"]))?;
    acl.add_resource("b", Some(&["c"]))?;
    acl.add_resource("c", Some(&["d"]))?;
    acl.add_resource("d", Some(&["b"]))?; // Creates cycle

    // Should detect the cycle
    let result = acl.check_resources_for_cycles();
    assert!(result.is_err(), "Should detect complex cycle");

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_with_diamond_structure() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create a diamond structure (not a cycle)
    //       top
    //      /   \
    //   left   right
    //      \   /
    //     bottom
    acl.add_resource("top", None)?;
    acl.add_resource("left", Some(&["top"]))?;
    acl.add_resource("right", Some(&["top"]))?;
    acl.add_resource("bottom", Some(&["left", "right"]))?;

    // Diamond structure is valid (no cycles)
    let result = acl.check_resources_for_cycles();
    assert!(result.is_ok(), "Diamond structure should not be detected as a cycle");

    Ok(())
  }

  // ============================
  // Tests for check_for_cycles
  // ============================

  #[test]
  fn test_check_for_cycles_no_cycles() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add valid roles and resources
    acl.add_role("guest", None)?;
    acl.add_role("user", Some(&["guest"]))?;
    acl.add_role("admin", Some(&["user"]))?;

    acl.add_resource("index", None)?;
    acl.add_resource("blog", Some(&["index"]))?;
    acl.add_resource("admin-panel", Some(&["blog"]))?;

    // Should not detect any cycles in either graph
    let result = acl.check_for_cycles();
    assert!(result.is_ok(), "Should not detect cycles in valid ACL");

    Ok(())
  }

  #[test]
  fn test_check_for_cycles_empty_acl() -> Result<(), Box<dyn std::error::Error>> {
    let acl = Acl::new();

    // Empty ACL should not have cycles
    let result = acl.check_for_cycles();
    assert!(result.is_ok(), "Empty ACL should not have cycles");

    Ok(())
  }

  #[test]
  fn test_check_for_cycles_detects_role_cycle() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add valid resources
    acl.add_resource("blog", None)?;

    // Create a cycle in roles
    acl.add_role("role-a", None)?;
    acl.add_role("role-b", None)?;
    acl.add_role("role-a", Some(&["role-b"]))?;
    acl.add_role("role-b", Some(&["role-a"]))?;

    // Should detect cycle in roles
    let result = acl.check_for_cycles();
    assert!(result.is_err(), "Should detect cycle in roles");

    if let Err(msg) = result {
      assert!(msg.contains("roles"), "Error message should mention roles");
    }

    Ok(())
  }

  #[test]
  fn test_check_for_cycles_detects_resource_cycle() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Add valid roles
    acl.add_role("user", None)?;

    // Create a cycle in resources
    acl.add_resource("res-a", None)?;
    acl.add_resource("res-b", None)?;
    acl.add_resource("res-a", Some(&["res-b"]))?;
    acl.add_resource("res-b", Some(&["res-a"]))?;

    // Should detect cycle in resources
    let result = acl.check_for_cycles();
    assert!(result.is_err(), "Should detect cycle in resources");

    if let Err(msg) = result {
      assert!(msg.contains("resources"), "Error message should mention resources");
    }

    Ok(())
  }

  #[test]
  fn test_check_for_cycles_detects_both_cycles() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create a cycle in roles
    acl.add_role("role-a", None)?;
    acl.add_role("role-b", None)?;
    acl.add_role("role-a", Some(&["role-b"]))?;
    acl.add_role("role-b", Some(&["role-a"]))?;

    // Create a cycle in resources
    acl.add_resource("res-a", None)?;
    acl.add_resource("res-b", None)?;
    acl.add_resource("res-a", Some(&["res-b"]))?;
    acl.add_resource("res-b", Some(&["res-a"]))?;

    // Should detect cycle (roles are checked first)
    let result = acl.check_for_cycles();
    assert!(result.is_err(), "Should detect cycles");

    // Should fail on roles first (since check_for_cycles checks roles before resources)
    if let Err(msg) = result {
      assert!(msg.contains("roles"), "Error message should mention roles (checked first)");
    }

    Ok(())
  }

  #[test]
  fn test_check_for_cycles_with_complex_valid_structure() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = Acl::new();

    // Create complex valid role hierarchy
    acl.add_role("guest", None)?;
    acl.add_role("member", Some(&["guest"]))?;
    acl.add_role("moderator", Some(&["member"]))?;
    acl.add_role("admin", Some(&["moderator"]))?;
    acl.add_role("super-admin", Some(&["admin"]))?;

    // Create complex valid resource hierarchy with multiple inheritance
    acl.add_resource("base", None)?;
    acl.add_resource("read-only", Some(&["base"]))?;
    acl.add_resource("write-enabled", Some(&["base"]))?;
    acl.add_resource("full-access", Some(&["read-only", "write-enabled"]))?;

    // Should not detect any cycles
    let result = acl.check_for_cycles();
    assert!(result.is_ok(), "Should not detect cycles in complex valid structure");

    Ok(())
  }
}

use crate::prelude::{String, Vec, vec, format};
use walrs_digraph::{DigraphDFSShape, DirectedCycle, DirectedPathsDFS, DisymGraph};

use crate::simple::rule::{Rule};
use crate::simple::resource_role_rules::ResourceRoleRules;

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
/// Note: This implementation does not expose any `*remove*` methods as both 'allow' and 'deny',
/// rules can be set for any given role, resource, and/or privilege, and, additionally, any
/// conditional logic can be performed at declaration time.
///
/// Note: If you require `*remove*`/`*delete*` functionality please
/// open an issue ticket/pull-request for it.
///
/// # Usage
///
/// ACLs should be created using `AclBuilder` for a fluent, type-safe construction experience:
///
/// ```rust
/// use walrs_acl::simple::AclBuilder;
///
/// let acl = AclBuilder::new()
///     .add_role("guest", None)?
///     .add_role("user", Some(&["guest"]))?
///     .add_role("admin", Some(&["user"]))?
///     .add_resource("blog", None)?
///     .add_resource("admin_panel", None)?
///     .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
///     .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?
///     .allow(Some(&["admin"]), None, None)?
///     .build()?;
///
/// assert!(acl.is_allowed(Some("admin"), Some("blog"), Some("delete")));
/// # Ok::<(), String>(())
/// ```
#[derive(Debug)]
pub struct Acl {
  pub(crate) _roles: DisymGraph,
  pub(crate) _resources: DisymGraph,
  pub(crate) _rules: ResourceRoleRules,
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

  /// Creates an Acl instance from its constituent parts.
  /// This is primarily used by `AclBuilder`.
  pub(crate) fn from_parts(roles: DisymGraph, resources: DisymGraph, rules: ResourceRoleRules) -> Self {
    Acl {
      _roles: roles,
      _resources: resources,
      _rules: rules,
    }
  }

  /// Returns the number of roles in the Acl.
  pub fn role_count(&self) -> usize {
    self._roles.vert_count()
  }

  /// Returns the number of resources in the Acl.
  pub fn resource_count(&self) -> usize {
    self._resources.vert_count()
  }

  /// Returns a boolean indicating whether the Acl contains a given role or not.
  pub fn has_role(&self, role: &str) -> bool {
    self._roles.has_vertex(role.as_ref())
  }

  /// Returns a boolean indicating whether `role` inherits `inherits` (... extends it etc.).
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let guest = "guest";
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  ///
  /// // Add roles, and their relationships to the acl:
  /// let acl = AclBuilder::default()
  ///     .add_role(&guest, None)?
  ///     .add_role(&admin, Some(&[&guest]))?
  ///     .add_role(&super_admin, Some(&[&admin]))?
  ///     .build()?;
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
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let guest = "guest";
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  ///
  /// // Add roles, and their relationships
  /// let acl = AclBuilder::default()
  ///     .add_role(&guest, None)?
  ///     .add_role(&admin, Some(&[&guest]))?
  ///     .add_role(&super_admin, Some(&[&admin]))?
  ///     .build()?;
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

  /// Returns a `bool` indicating whether Acl contains given "resource" symbol or not.
  pub fn has_resource(&self, resource: &str) -> bool {
    self._resources.contains(resource)
  }

  /// Returns a `Result` containing a boolean indicating whether `resource` inherits
  /// `inherits` (... extends it etc.). Returns `Result::Err` if any of the given vertices
  /// do not exists in the `Acl`.
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let guest = "guest";
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  ///
  /// // Add resources, and their relationships
  /// let acl = AclBuilder::default()
  ///     .add_resource(&guest, None)?
  ///     .add_resource(&admin, Some(&[&guest]))?
  ///     .add_resource(&super_admin, Some(&[&admin]))?
  ///     .build()?;
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
      return DirectedPathsDFS::new(self._resources.graph(), v1).and_then(|dfs| dfs.has_path_to(v2));
    }
    Err(format!("{} is not in symbol graph", inherits))
  }

  /// Returns a boolean indicating whether `resource` inherits `inherits` (... extends it etc.).
  /// Note: This method panics if `resource`, and/or `inherits`, don't exist in the ACL;
  /// For non "panic" version use `#Acl.inherits_resource_safe`.
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let guest = "guest";
  /// let admin = "admin";
  /// let super_admin = "super_admin";
  ///
  /// // Add resources, and their relationships
  /// let acl = AclBuilder::default()
  ///     .add_resource(&guest, None)?
  ///     .add_resource(&admin, Some(&[&guest]))?
  ///     .add_resource(&super_admin, Some(&[&admin]))?
  ///     .build()?;
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

  /// Checks roles graph for directed cycles; If `Ok(())` then no cycles are found, else returns
  /// `Err(String)` with a message indicating the cycle(s) found.
  pub fn check_roles_for_cycles(&self) -> Result<(), String> {
    if let Some(cycles) = DirectedCycle::new(self._roles.graph()).cycle() {
      if cycles.is_empty() { return Ok(()); }
      let cycles_repr = self._roles.names(cycles).unwrap()
          .join(" <- ");
      return Err(format!("Acl contains cyclic edges in \"roles\" graph: {:?}", cycles_repr));
    }
    Ok(())
  }

  /// Checks resources graph for directed cycles; If `Ok(())` then no cycles are found, else returns
  /// `Err(String)` with a message indicating the cycle(s) found.
  pub fn check_resources_for_cycles(&self) -> Result<(), String> {
    if let Some(cycles) = DirectedCycle::new(self._resources.graph()).cycle() {
      if cycles.is_empty() { return Ok(()); }
      let cycles_repr = self._resources.names(cycles).unwrap()
          .join(" <- ");
      return Err(format!("Acl contains cycles in 'resources' graph: {:?}", cycles_repr));
    }
    Ok(())
  }

  /// Checks Acl for cycles; E.g., if Acl contains cycles, then it is not possible to determine
  /// whether a given role/resource/privilege rule combination is allowed or not so the method
  /// should be used before using the [acl] structure for validating "allowed" rules.
  pub fn check_for_cycles(&self) -> Result<(), String> {
    self.check_roles_for_cycles()?;
    self.check_resources_for_cycles()?;
    Ok(())
  }

  /// Returns a boolean indicating whether the given "role" is allowed access to
  /// the given "privilege" on the given "resource".  If any of the args are `None` the "all"
  /// variant is checked for that `None` value; E.g.,
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// // Roles
  /// let guest = "guest";
  /// let user = "user";
  /// let special = "special";
  /// let admin = "admin";
  ///
  /// // Resources
  /// let index = "index";
  /// let protected = "protected";
  ///
  /// // Privilege
  /// let read = "read";
  ///
  /// // Build ACL
  /// let acl = AclBuilder::default()
  ///   // Second 'arg' is role inheritance, which is optional
  ///   .add_roles(&[
  ///     (guest, None),
  ///     (user, Some(&[guest])),
  ///     (special, None),
  ///     (admin, Some(&[user, special]))
  ///   ])?
  ///   .add_resources(&[
  ///     (index, None),
  ///     (protected, None)
  ///   ])?
  ///   .build()?;
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
  /// let acl = AclBuilder::try_from(acl)?
  ///   .allow(Some(&[guest]), Some(&[index]), Some(&[read]))?
  ///   .build()?;
  ///
  /// // Perform check
  /// assert_eq!(acl.is_allowed(Some(guest), Some(index), Some(read)), true, "Has \"read\" privilege on \"index\"");
  ///
  /// // Add "all privileges" for "user", on "index" resource
  /// let acl = AclBuilder::try_from(acl)?
  ///   .allow(Some(&[user]), Some(&[index]), None)?
  ///   .build()?;
  ///
  /// // Checks
  /// assert!(acl.is_allowed(Some(user), Some(index), None));
  /// assert!(acl.is_allowed(Some(admin), Some(index), None)); // inherits access from "user" role
  ///
  /// // Check random resource and priv, on "admin"
  /// assert!(!acl.is_allowed(Some(admin), Some(protected), Some("GET")));
  ///
  /// // Add "all privileges" for "admin", on all resources
  /// let acl = AclBuilder::try_from(acl)?
  ///   .allow(Some(&[admin]), None, None)?
  ///   .build()?;
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
  /// let acl = AclBuilder::try_from(acl)?
  ///   .allow(Some(&[special]), Some(&[index]), Some(&["report"]))?
  ///   .build()?;
  ///
  /// // Checks
  /// assert!(acl.is_allowed(Some(special), Some(index), Some("report")), "Should have \"report\" privilege on \"index\"");
  /// # Ok::<(), String>(())
  /// ```
  pub fn is_allowed(
    &self,
    role: Option<&str>,
    resource: Option<&str>,
    privilege: Option<&str>,
  ) -> bool {
    // Get ALL inherited roles (including transitive parents) using DFS
    let _roles = role.and_then(|_role| {
      let role_idx = self._roles.index(_role)?;
      let dfs = DirectedPathsDFS::new(self._roles.graph(), role_idx).ok()?;

      let mut inherited = Vec::new();
      for i in 0..self._roles.vert_count() {
        if i != role_idx && dfs.marked(i).unwrap_or(false) {
          if let Some(name) = self._roles.name_as_ref(i) {
            inherited.push(name);
          }
        }
      }

      if inherited.is_empty() { None } else { Some(inherited) }
    });

    // Get ALL inherited resources (including transitive parents) using DFS
    let _resources = resource.and_then(|_resource| {
      let resource_idx = self._resources.index(_resource)?;
      let dfs = DirectedPathsDFS::new(self._resources.graph(), resource_idx).ok()?;

      let mut inherited = Vec::new();
      for i in 0..self._resources.vert_count() {
        if i != resource_idx && dfs.marked(i).unwrap_or(false) {
          if let Some(name) = self._resources.name_as_ref(i) {
            inherited.push(name);
          }
        }
      }

      if inherited.is_empty() { None } else { Some(inherited) }
    });

    // CRITICAL: Check for explicit Deny on the DIRECT role/resource combo FIRST
    // This ensures that deny rules on a role/resource override inherited allow rules
    // We only block if there's an EXPLICIT Deny entry in the by_privilege_id map
    let has_explicit_deny = if let Some(priv_id) = privilege {
      // Checking a specific privilege - look for explicit Deny in the map
      if resource.is_some() {
        let role_rules = self._rules.get_role_privilege_rules(resource).get_privilege_rules(role);
        role_rules.by_privilege_id.as_ref()
          .and_then(|map| map.get(priv_id))
          .map(|rule| rule == &Rule::Deny)
          .unwrap_or(false)
      } else {
        let role_rules = self._rules.for_all_resources.get_privilege_rules(role);
        role_rules.by_privilege_id.as_ref()
          .and_then(|map| map.get(priv_id))
          .map(|rule| rule == &Rule::Deny)
          .unwrap_or(false)
      }
    } else {
      // Checking all privileges (None) - this is handled by normal logic, don't block
      false
    };

    // If there's an explicit deny on the direct role/resource, deny immediately (don't check inheritance)
    if has_explicit_deny {
      return false;
    }

    // ...existing code...

    // Callback for returning `allow` check result, or checking if current parameter set has `allow` permission
    //  Helps dry up the code, below, a bit
    let rslt_or_check_direct = |rslt| {
      if rslt {
        rslt
      } else {
        self._matches_rule_no_dfs(role, resource, privilege, &Rule::Allow)
      }
    };

    // println!("Inherited roles and resources {:?}, {:?}", &_roles, &_resources);

    // If inherited `resources`, and `roles`, found, loop through them and check for `Allow` rule
    _resources
      .as_ref()
      .zip(_roles.as_ref())
      .map(|(_resources, _roles2)| {
        _resources.iter().rev().any(|_resource| {
          _roles2
            .iter()
            .rev()
            .any(|_role| self._matches_rule_no_dfs(Some(_role), Some(_resource), privilege, &Rule::Allow))
        })
      })
      // If no inherited roles/resources directly allowed check direct allow on incoming (role, resource, privilege)
      .map(rslt_or_check_direct)
      // If only `roles`, only `resources`, or neither of the two, check for `Allow` rule
      .or_else(|| {
        // If only `roles` check for allow on roles inheritance graph from shallowest node
        if _resources.is_none() && _roles.is_some() {
          _roles
            .map(|_rs| {
              _rs
                .iter()
                .rev()
                .any(|r| self._matches_rule_no_dfs(Some(r), resource, privilege, &Rule::Allow))
            })
            .map(rslt_or_check_direct)
        }
        // Else inherited resources is set, but not inherited roles,
        // check resources inheritance graph from shallowest node to deepest
        else if _resources.is_some() && _roles.is_none() {
          _resources
            .map(|_rs| {
              _rs
                .iter()
                .rev()
                .any(|r| self._matches_rule_no_dfs(role, Some(*r), privilege, &Rule::Allow))
            })
            .map(rslt_or_check_direct)
        }
        // Else check for direct allowance
        else {
          self._matches_rule_no_dfs(role, resource, privilege, &Rule::Allow).into()
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
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// // Define roles
  /// let guest = "guest";
  /// let user = "user";
  /// let admin = "admin";
  ///
  /// // Define resources
  /// let blog = "blog";
  /// let account = "account";
  /// let admin_panel = "admin-panel";
  ///
  /// // Define privileges
  /// let read = "read";
  /// let write = "write";
  /// let delete = "delete";
  ///
  /// // Build ACL with roles, resources, and permissions
  /// let acl = AclBuilder::default()
  ///   // Add roles with inheritance: admin -> user -> guest
  ///   .add_roles(&[
  ///     (guest, None),
  ///     (user, Some(&[guest])),
  ///     (admin, Some(&[user]))
  ///   ])?
  ///   .add_resource(blog, None)?
  ///   .add_resource(account, None)?
  ///   .add_resource(admin_panel, None)?
  ///   // Guest can read blog
  ///   .allow(Some(&[guest]), Some(&[blog]), Some(&[read]))?
  ///   // User can write to blog and account
  ///   .allow(Some(&[user]), Some(&[blog, account]), Some(&[write]))?
  ///   // Admin has delete privilege on admin-panel
  ///   .allow(Some(&[admin]), Some(&[admin_panel]), Some(&[delete]))?
  ///   .build()?;
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
  /// let acl = AclBuilder::default()
  ///   .add_roles(&[
  ///     (guest, None),
  ///     (user, Some(&[guest])),
  ///     (admin, Some(&[user]))
  ///   ])?
  ///   .add_resource(blog, None)?
  ///   .add_resource(account, None)?
  ///   .add_resource(admin_panel, None)?
  ///   .allow(Some(&[guest]), Some(&[blog]), Some(&[read]))?
  ///   .allow(Some(&[user]), Some(&[blog, account]), Some(&[write]))?
  ///   .allow(Some(&[admin]), Some(&[admin_panel]), Some(&[delete]))?
  ///   .allow(Some(&[user]), Some(&[account]), None)? // Give user all privileges on account
  ///   .build()?;
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

  /// Returns a boolean indicating whether the given rule matches or not -
  /// Does not check symbol graph for inheritance chains (flat "rules" check).
  fn _matches_rule_no_dfs(
    &self,
    role: Option<&str>,
    resource: Option<&str>,
    privilege: Option<&str>,
    rule: &Rule,
  ) -> bool {
    // First check the specific resource (if provided)
    if resource.is_some() {
      let specific_rule = self
        ._rules
        .get_role_privilege_rules(resource)
        .get_privilege_rules(role)
        .get_rule(privilege);

      // If we found an explicit match, return true
      if specific_rule == rule {
        return true;
      }

      // Also check for_all_resources for this role (global rules)
      let global_rule = self
        ._rules
        .for_all_resources
        .get_privilege_rules(role)
        .get_rule(privilege);

      return global_rule == rule;
    }

    // If no specific resource, just check for_all_resources
    self
      ._rules
      .for_all_resources
      .get_privilege_rules(role)
      .get_rule(privilege)
      == rule
  }
}

impl Default for Acl {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod test_acl {
  use crate::simple::acl::{Acl};
  use crate::simple::acl_builder::AclBuilder;
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
  fn test_has_resource() -> Result<(), String> {
    let index = "index";
    let users = "users";
    let non_existent_resource = "non-existent-resource";

    // Build ACL with resources and their relationships
    let acl = AclBuilder::new()
      .add_resource(users, Some([index].as_slice()))?
      .build()?;

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

    Ok(())
  }

  #[test]
  fn test_has_role() -> Result<(), String> {
    let admin = "admin";
    let super_admin = "super_admin";
    let non_existent_role = "non-existent-role";

    // Build ACL with roles and their relationships
    let acl = AclBuilder::new()
      .add_role(admin, Some([super_admin].as_slice()))?
      .build()?;

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

    Ok(())
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
  fn test_acl_allow() -> Result<(), String> {
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

    let build_acl_with_symbols = || -> Result<AclBuilder, String> {
      let mut builder = AclBuilder::new();
      builder.add_role(guest_role, None)?
        .add_role(user_role, Some(&[guest_role]))?
        .add_role(admin_role, Some(&[user_role]))?
        .add_resource(index_resource, None)?
        .add_resource(blog_resource, Some(&[index_resource]))?
        .add_resource(account_resource, None)?
        .add_resource(users_resource, None)?;
      Ok(builder)
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
      let mut builder = build_acl_with_symbols()?;

      // If we're testing for 'allow' set allow rule result to test
      if expected {
        builder.allow(roles, resources, privileges)?;
      }

      let acl = builder.build()?;

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

    Ok(())
  }

  #[test]
  fn test_acl_deny() -> Result<(), String> {
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

    let build_acl_with_symbols = || -> Result<AclBuilder, String> {
      let mut builder = AclBuilder::new();
      builder.add_role(guest_role, None)?
        .add_role(user_role, Some(&[guest_role]))?
        .add_role(admin_role, Some(&[user_role]))?
        .add_resource(index_resource, None)?
        .add_resource(blog_resource, Some(&[index_resource]))?
        .add_resource(account_resource, None)?
        .add_resource(users_resource, None)?;
      Ok(builder)
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
      let acl = build_acl_with_symbols()?
        .deny(roles, resources, privileges)?
        .build()?;

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

    Ok(())
  }

  #[test]
  fn test_acl_deny_comprehensive() -> Result<(), String> {
    // Define roles with inheritance
    let guest = "guest";
    let user = "user";
    let moderator = "moderator";
    let admin = "admin";

    // Define resources
    let blog = "blog";
    let account = "account";
    let admin_panel = "admin-panel";
    let secret = "secret";

    // Define privileges
    let read = "read";
    let write = "write";
    let delete = "delete";
    let publish = "publish";

    // Helper to build base ACL or build from existing ACL
    let build_base_acl = |prev_acl: Option<&Acl>| -> Result<AclBuilder, String> {
      match prev_acl {
        Some(acl) => AclBuilder::try_from(acl),
        None => {
          let mut builder = AclBuilder::new();
          builder.add_roles(&[
            (guest, None),
            (user, Some(&[guest])),
            (moderator, Some(&[user])),
            (admin, Some(&[moderator]))
          ])?
          .add_resource(blog, None)?
          .add_resource(account, None)?
          .add_resource(admin_panel, None)?
          .add_resource(secret, None)?;
          Ok(builder)
        }
      }
    };

    // Test 1: Deny specific privilege on specific resource for specific role
    let acl1 = build_base_acl(None)?
      .deny(Some(&[guest]), Some(&[admin_panel]), Some(&[read]))?
      .build()?;
    assert!(
      !acl1.is_allowed(Some(guest), Some(admin_panel), Some(read)),
      "Guest should be denied read access to admin-panel"
    );

    // Test 2: Deny all privileges on a resource (None for privileges)
    let acl2 = build_base_acl(Some(&acl1))?
      .deny(Some(&[user]), Some(&[secret]), None)?
      .build()?;
    assert!(
      !acl2.is_allowed(Some(user), Some(secret), Some(read)),
      "User should be denied all access to secret resource"
    );
    assert!(
      !acl2.is_allowed(Some(user), Some(secret), Some(write)),
      "User should be denied all access to secret resource"
    );
    assert!(
      !acl2.is_allowed(Some(user), Some(secret), None),
      "User should be denied all access to secret resource"
    );

    // Test 3: Deny multiple privileges at once
    let acl3 = build_base_acl(Some(&acl2))?
      .deny(Some(&[guest]), Some(&[blog]), Some(&[write, delete, publish]))?
      .build()?;
    assert!(
      !acl3.is_allowed(Some(guest), Some(blog), Some(write)),
      "Guest should be denied write on blog"
    );
    assert!(
      !acl3.is_allowed(Some(guest), Some(blog), Some(delete)),
      "Guest should be denied delete on blog"
    );
    assert!(
      !acl3.is_allowed(Some(guest), Some(blog), Some(publish)),
      "Guest should be denied publish on blog"
    );

    // Test 4: Deny across multiple roles
    let acl4 = build_base_acl(Some(&acl3))?
      .deny(Some(&[guest, user]), Some(&[admin_panel]), None)?
      .build()?;
    assert!(
      !acl4.is_allowed(Some(guest), Some(admin_panel), None),
      "Guest should be denied all access to admin-panel"
    );
    assert!(
      !acl4.is_allowed(Some(user), Some(admin_panel), Some(write)),
      "User should be denied all access to admin-panel"
    );

    // Test 5: Deny across multiple resources
    let acl5 = build_base_acl(Some(&acl4))?
      .deny(Some(&[moderator]), Some(&[secret, admin_panel]), Some(&[delete]))?
      .build()?;
    assert!(
      !acl5.is_allowed(Some(moderator), Some(secret), Some(delete)),
      "Moderator should be denied delete on secret"
    );
    assert!(
      !acl5.is_allowed(Some(moderator), Some(admin_panel), Some(delete)),
      "Moderator should be denied delete on admin-panel"
    );

    // Test 6: Allow and then deny for the same role (explicit deny takes precedence)
    let acl6 = build_base_acl(Some(&acl5))?
      .allow(Some(&[user]), Some(&[blog]), Some(&[write]))?
      .deny(Some(&[user]), Some(&[blog]), Some(&[write]))?
      .build()?;
    assert!(
      !acl6.is_allowed(Some(user), Some(blog), Some(write)),
      "User should be denied write access to blog (deny overrides allow)"
    );

    // Test 7: Deny all roles on a resource (None for roles)
    let acl7 = build_base_acl(Some(&acl6))?
      .deny(None, Some(&[secret]), Some(&[read]))?
      .build()?;
    assert!(
      !acl7.is_allowed(Some(guest), Some(secret), Some(read)),
      "All roles (including guest) should be denied read on secret"
    );
    assert!(
      !acl7.is_allowed(Some(admin), Some(secret), Some(read)),
      "All roles (including admin) should be denied read on secret"
    );

    // Test 8: Deny role on all resources (None for resources)
    let acl8 = build_base_acl(Some(&acl7))?
      .deny(Some(&[guest]), None, Some(&[delete]))?
      .build()?;
    assert!(
      !acl8.is_allowed(Some(guest), Some(blog), Some(delete)),
      "Guest should be denied delete on all resources (blog)"
    );
    assert!(
      !acl8.is_allowed(Some(guest), Some(account), Some(delete)),
      "Guest should be denied delete on all resources (account)"
    );

    // Test 9: Method chaining
    let acl9 = build_base_acl(Some(&acl8))?
      .deny(Some(&[user]), Some(&[account]), Some(&[delete]))?
      .deny(Some(&[user]), Some(&[blog]), Some(&[publish]))?
      .deny(Some(&[moderator]), Some(&[secret]), None)?
      .build()?;
    assert!(!acl9.is_allowed(Some(user), Some(account), Some(delete)));
    assert!(!acl9.is_allowed(Some(user), Some(blog), Some(publish)));
    assert!(!acl9.is_allowed(Some(moderator), Some(secret), Some(read)));

    // Test 10: Empty arrays behave like None (all)
    let acl10 = build_base_acl(Some(&acl9))?
      .deny(Some(&[]), Some(&[admin_panel]), Some(&[write]))?
      .build()?;
    // Empty roles array means "all roles"
    assert!(
      !acl10.is_allowed(Some(admin), Some(admin_panel), Some(write)),
      "Empty roles array should deny all roles"
    );

    Ok(())
  }

  #[test]
  #[should_panic(expected = "d is not in symbol graph")]
  fn test_inherits_role() {
    let acl = AclBuilder::new()
      .add_role("a", Some(["b", "c"].as_slice()))
      .unwrap()
      .build()
      .unwrap();
    assert!(acl.inherits_role("a", "b"));
    assert!(acl.inherits_role("a", "c"));
    assert!(acl.inherits_role("a", "d"));
  }

  #[test]
  #[should_panic(expected = "d is not in symbol graph")]
  fn test_inherits_resource() {
    let acl = AclBuilder::new()
      .add_resource("a", Some(["b", "c"].as_slice()))
      .unwrap()
      .build()
      .unwrap();
    assert!(acl.inherits_resource("a", "b"));
    assert!(acl.inherits_resource("a", "c"));
    assert!(acl.inherits_resource("a", "d"));
  }


  // ============================
  // Tests for check_resources_for_cycles
  // ============================

  #[test]
  fn test_check_resources_for_cycles_no_cycles() -> Result<(), Box<dyn std::error::Error>> {
    // Create a valid DAG structure
    let acl = AclBuilder::new()
      .add_resource("guest", None)?
      .add_resource("user", Some(&["guest"]))?
      .add_resource("admin", Some(&["user"]))?
      .build()?;

    // Should not detect any cycles (build already checked)
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
    let acl = AclBuilder::new()
      .add_resource("blog", None)?
      .build()?;

    // Single resource should not have cycles
    let result = acl.check_resources_for_cycles();
    assert!(result.is_ok(), "Single resource should not have cycles");

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_detects_simple_cycle() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple cycle: a -> b -> a
    // The build() method should detect this cycle
    let result = AclBuilder::new()
      .add_resource("a", None)?
      .add_resource("b", None)?
      .add_resource("a", Some(&["b"]))?
      .add_resource("b", Some(&["a"]))?
      .build();

    // Should detect the cycle during build
    assert!(result.is_err(), "Should detect simple cycle");

    if let Err(msg) = result {
      assert!(msg.contains("cycle") || msg.contains("Cycle"), "Error message should mention cycles");
    }

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_detects_self_cycle() -> Result<(), Box<dyn std::error::Error>> {

    // Create a self-referencing resource
    // The build() method should detect this cycle
    let result = AclBuilder::new()
      .add_resource("self-ref", None)?
      .add_resource("self-ref", Some(&["self-ref"]))?
      .build();

    // Should detect the self-cycle during build
    assert!(result.is_err(), "Should detect self-referencing cycle");

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_detects_complex_cycle() -> Result<(), Box<dyn std::error::Error>> {
    // Create a complex cycle: a -> b -> c -> d -> b
    // The build() method should detect this cycle
    let result = AclBuilder::new()
      .add_resource("a", None)?
      .add_resource("b", None)?
      .add_resource("c", None)?
      .add_resource("d", None)?
      .add_resource("a", Some(&["b"]))?
      .add_resource("b", Some(&["c"]))?
      .add_resource("c", Some(&["d"]))?
      .add_resource("d", Some(&["b"]))? // Creates cycle
      .build();

    // Should detect the cycle during build
    assert!(result.is_err(), "Should detect complex cycle");

    Ok(())
  }

  #[test]
  fn test_check_resources_for_cycles_with_diamond_structure() -> Result<(), Box<dyn std::error::Error>> {

    // Create a diamond structure (not a cycle)
    //       top
    //      /   \
    //   left   right
    //      \   /
    //     bottom
    let acl = AclBuilder::new()
      .add_resource("top", None)?
      .add_resource("left", Some(&["top"]))?
      .add_resource("right", Some(&["top"]))?
      .add_resource("bottom", Some(&["left", "right"]))?
      .build()?;

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

    // Add valid roles and resources - build will check for cycles
    let acl = AclBuilder::new()
      .add_role("guest", None)?
      .add_role("user", Some(&["guest"]))?
      .add_role("admin", Some(&["user"]))?
      .add_resource("index", None)?
      .add_resource("blog", Some(&["index"]))?
      .add_resource("admin-panel", Some(&["blog"]))?
      .build()?;

    // Should not detect any cycles in either graph (already validated by build)
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
    // Create a cycle in roles - should fail at build time
    let result = AclBuilder::new()
      .add_resource("blog", None)?
      .add_role("role-a", None)?
      .add_role("role-b", None)?
      .add_role("role-a", Some(&["role-b"]))?
      .add_role("role-b", Some(&["role-a"]))?
      .build();

    // Should detect cycle in roles during build
    assert!(result.is_err(), "Should detect cycle in roles");

    if let Err(msg) = result {
      assert!(msg.contains("role") || msg.contains("Role") || msg.contains("Cycle") || msg.contains("cycle"),
        "Error message should mention roles or cycles");
    }

    Ok(())
  }

  #[test]
  fn test_check_for_cycles_detects_resource_cycle() -> Result<(), Box<dyn std::error::Error>> {
    // Create a cycle in resources - should fail at build time
    let result = AclBuilder::new()
      .add_role("user", None)?
      .add_resource("res-a", None)?
      .add_resource("res-b", None)?
      .add_resource("res-a", Some(&["res-b"]))?
      .add_resource("res-b", Some(&["res-a"]))?
      .build();

    // Should detect cycle in resources during build
    assert!(result.is_err(), "Should detect cycle in resources");

    if let Err(msg) = result {
      assert!(msg.contains("resource") || msg.contains("Resource") || msg.contains("Cycle") || msg.contains("cycle"),
        "Error message should mention resources or cycles");
    }

    Ok(())
  }

  #[test]
  fn test_check_for_cycles_detects_both_cycles() -> Result<(), Box<dyn std::error::Error>> {
    // Create a cycle in both roles and resources - should fail at build time
    let result = AclBuilder::new()
      .add_role("role-a", None)?
      .add_role("role-b", None)?
      .add_role("role-a", Some(&["role-b"]))?
      .add_role("role-b", Some(&["role-a"]))?
      .add_resource("res-a", None)?
      .add_resource("res-b", None)?
      .add_resource("res-a", Some(&["res-b"]))?
      .add_resource("res-b", Some(&["res-a"]))?
      .build();

    // Should detect cycle during build (roles are checked first)
    assert!(result.is_err(), "Should detect cycles");

    if let Err(msg) = result {
      assert!(msg.contains("role") || msg.contains("Role") || msg.contains("Cycle") || msg.contains("cycle"),
        "Error message should mention roles or cycles (roles checked first)");
    }

    Ok(())
  }

  #[test]
  fn test_check_for_cycles_with_complex_valid_structure() -> Result<(), Box<dyn std::error::Error>> {
    // Create complex valid role and resource hierarchy - should succeed
    let acl = AclBuilder::new()
      .add_role("guest", None)?
      .add_role("member", Some(&["guest"]))?
      .add_role("moderator", Some(&["member"]))?
      .add_role("admin", Some(&["moderator"]))?
      .add_role("super-admin", Some(&["admin"]))?
      .add_resource("base", None)?
      .add_resource("read-only", Some(&["base"]))?
      .add_resource("write-enabled", Some(&["base"]))?
      .add_resource("full-access", Some(&["read-only", "write-enabled"]))?
      .build()?;

    // Should not detect any cycles (already validated by build)
    let result = acl.check_for_cycles();
    assert!(result.is_ok(), "Should not detect cycles in complex valid structure");

    Ok(())
  }
}

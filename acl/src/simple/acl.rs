use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;

use serde_json;
use walrs_graph::digraph::{
  DigraphDFSShape,
  DigraphDipathsDFS,
  DisymGraph
};

use crate::simple::acl_data::AclData;
use crate::simple::rule::{Rule};
use crate::simple::privilege_rules::PrivilegeRules;
use crate::simple::resource_role_rules::ResourceRoleRules;
use crate::simple::role_privilege_rules::RolePrivilegeRules;

// Note: Rules structure:
// Resources contain roles, roles contain privileges,
// privileges contain allow/deny rules, and/or, assertion functions,
// Privilege, Role, and Resource Ids  are string slices - See relevant imports
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
  /// acl .add_role(developer, Some(&[tester]))
  ///     .add_role(admin, Some(&[developer]))
  ///     .add_role(super_admin, Some(&[admin]));
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
  /// ```
  pub fn add_role(&mut self, role: &str, parents: Option<&[&str]>) -> &mut Self {
    if let Some(parents) = parents {
      if let Err(err) = self._roles.add_edge(role, parents) {
        panic!("{}", err);
      }
    }
    self._roles.add_vertex(role);
    self
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
  /// ]);
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
  /// ```
  pub fn add_roles(&mut self, roles: &[(&str, Option<&[&str]>)]) -> &mut Self {
    for &(role, parents) in roles {
      self.add_role(role, parents);
    }
    self
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
  /// acl.add_role(&guest, None)
  ///     .add_role(&admin, Some(&[&guest]))
  ///     .add_role(&super_admin, Some(&[&admin]));
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
  /// ```
  pub fn inherits_role_safe(&self, role: &str, inherits: &str) -> Result<bool, String> {
    if let Some((v1, v2)) = self._roles.index(role).zip(self._roles.index(inherits)) {
      return DigraphDipathsDFS::new(self._roles.graph(), v1).and_then(|dfs| dfs.marked(v2));
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
  /// acl.add_role(&guest, None)
  ///     .add_role(&admin, Some(&[&guest]))
  ///     .add_role(&super_admin, Some(&[&admin]));
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
  /// acl.add_resource(&term, None)
  ///     .add_resource(&post, Some(&[&term]))
  ///     .add_resource(&post_categories, Some(&[&term]));  ///
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
  /// ```
  pub fn add_resource(&mut self, resource: &str, parents: Option<&[&str]>) -> &mut Self {
    if let Some(parents) = parents {
      if let Err(err) = self._resources.add_edge(resource, parents) {
        panic!("{}", err);
      }
    }
    self._resources.add_vertex(resource);
    self
  }

  /// Returns a `bool` indicating whether Acl contains given resource symbol or not.
  pub fn has_resource(&self, resource: &str) -> bool {
    self._resources.contains(resource)
  }

  /// Returns a `Result` containing a boolean indicating whether `resource` inherits `inherits` (... extends it etc.).
  /// Returns `Result::Err` if any of the given vertices do not exists in the `Acl`.
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
  /// acl.add_resource(&guest, None)
  ///     .add_resource(&admin, Some(&[&guest]))
  ///     .add_resource(&super_admin, Some(&[&admin]));
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
  /// ```
  ///
  /// @todo Remove '*_safe' suffix.
  pub fn inherits_resource_safe(&self, resource: &str, inherits: &str) -> Result<bool, String> {
    if let Some((v1, v2)) = self
      ._resources
      .index(resource)
      .zip(self._resources.index(inherits))
    {
      return DigraphDipathsDFS::new(self._resources.graph(), v1).and_then(|dfs| dfs.marked(v2));
    }
    Err(format!("{} is not in symbol graph", inherits))
  }

  /// Returns a boolean indicating whether `resource` inherits `inherits` (... extends it etc.).
  /// Note: This method panics if `resource`, and/or `inherits`, don't exists in the ACL; For safe version use
  ///  `#Acl.inherits_resource_safe`.
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
  /// acl.add_resource(&guest, None)
  ///     .add_resource(&admin, Some(&[&guest]))
  ///     .add_resource(&super_admin, Some(&[&admin]));
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
  /// ```
  pub fn inherits_resource(&self, resource: &str, inherits: &str) -> bool {
    match self.inherits_resource_safe(resource, inherits) {
      Ok(is_inherited) => is_inherited,
      Err(err) => panic!("{}", err),
    }
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
  ///   .add_role(guest_role, None) // Inherits from none.
  ///   .add_role(user_role, Some(&[guest_role])) // 'user' role inherits rules applied to 'guest' role
  ///   .add_role(admin_role, Some(&[user_role])) // ...
  ///
  ///   // Add Resources
  ///   // ----
  ///   .add_resource(index_resource, None) // 'index' resource has inherits from none.
  ///   .add_resource(blog_resource, Some(&[index_resource])) // 'blog' resource inherits rules applied to 'index' resource
  ///   .add_resource(account_resource, None)
  ///   .add_resource(users_resource, None)
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
  ///
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
  ///  for a match.
  ///
  /// ```rust
  ///
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
      roles.iter().for_each(|(role, parents)| {
        // Convert `parents` to `Option<&[&str]>`
        let parents = parents
          .as_deref()
          .map(|xs| -> Vec<&str> { xs.iter().map(|x: &String| x.as_str()).collect() });

        // Add role(s);  If parent roles aren't in the acl, they get added via `acl.add_role`
        acl.add_role(role, parents.as_deref());
      });
    }

    // Add `resources` to `acl`
    if let Some(resources) = data.resources.as_ref() {
      // Loop through resource entries
      resources.iter().for_each(|(resource, parents)| {
        // Convert `parents` to `Option<&[&str]>`
        let parents = parents
          .as_deref()
          .map(|xs| -> Vec<&str> { xs.iter().map(|x: &String| x.as_str()).collect() });

        // Add resource(s);  If parent resources aren't in the acl, they get added via `acl.add_resource`
        acl.add_resource(resource, parents.as_deref());
      });
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

    // @todo Test non existent roles and resources for no inheritance, and allow rules against such

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
  fn test_has_resource() {
    let mut acl = Acl::new();
    let index = "index";
    let users = "users";
    let non_existent_resource = "non-existent-resource";

    // Add resources, and their relationships to the acl:
    acl.add_resource(users, Some([index].as_slice()));

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
    acl.add_role(admin, Some([super_admin].as_slice()));

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
      acl.add_role(guest_role, None);
      acl.add_role(user_role, Some(&[guest_role]));
      acl.add_role(admin_role, Some(&[user_role]));

      // Add Resources
      acl.add_resource(index_resource, None);
      acl.add_resource(blog_resource, Some(&[index_resource]));
      acl.add_resource(account_resource, None);
      acl.add_resource(users_resource, None);
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
      acl.add_role(guest_role, None);
      acl.add_role(user_role, Some(&[guest_role]));
      acl.add_role(admin_role, Some(&[user_role]));

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
  #[should_panic(expected = "d is not in symbol graph")]
  fn test_inherits_role() {
    let mut acl = Acl::new();
    acl.add_role("a", Some(["b", "c"].as_slice()));
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
}

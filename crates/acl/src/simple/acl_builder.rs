// The serde data model for rules uses deeply nested Vec/Option/tuple types by
// design. Silence clippy's `type_complexity` here — refactoring them into
// named aliases buys little readability at significant signature noise cost.
#![allow(clippy::type_complexity)]
use crate::prelude::{String, ToString, Vec, format, vec};
use core::convert::TryFrom;

#[cfg(feature = "std")]
use std::fs::File;

use crate::simple::{Acl, AclData, ResourceRoleRules, Rule};
use walrs_digraph::DisymGraph;

// Convenience method.
fn _is_empty(list: &Option<&[&str]>) -> bool {
  list.is_none_or(|xs| xs.is_empty())
}

/// Builder for constructing `Acl` instances with a fluent interface.
///
/// # Example
///
/// ```rust
/// use walrs_acl::simple::{Acl, AclBuilder};
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
/// assert!(acl.is_allowed(Some("admin"), Some("blog"), Some("read")));
/// # Ok::<(), String>(())
/// ```
#[derive(Debug)]
pub struct AclBuilder {
  _roles: DisymGraph,
  _resources: DisymGraph,
  _rules: ResourceRoleRules,
}

impl AclBuilder {
  /// Creates a new `AclBuilder` instance.
  pub fn new() -> Self {
    AclBuilder {
      _roles: DisymGraph::new(),
      _resources: DisymGraph::new(),
      _rules: ResourceRoleRules::new(),
    }
  }

  /// Adds a role to the ACL being built.
  ///
  /// # Arguments
  ///
  /// * `role` - The role identifier.
  /// * `parents` - Optional slice of parent role identifiers that this role inherits from.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let mut builder = AclBuilder::new()
  ///     .add_role("guest", None)?
  ///     .add_role("user", Some(&["guest"]))?
  ///     .add_role("admin", Some(&["user"]))?;
  /// # Ok::<(), String>(())
  /// ```
  pub fn add_role(&mut self, role: &str, parents: Option<&[&str]>) -> Result<&mut Self, String> {
    self.add_roles(&[(role, parents)])
  }

  /// Adds multiple roles to the ACL being built.
  ///
  /// # Arguments
  ///
  /// * `roles` - Slice of tuples containing role identifiers and their optional parent roles.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let mut builder = AclBuilder::new()
  ///   .add_roles(&[
  ///     ("guest", None),
  ///     ("user", Some(&["guest"])),
  ///     ("admin", Some(&["user"])),
  /// ])?;
  /// # Ok::<(), String>(())
  /// ```
  pub fn add_roles(&mut self, roles: &[(&str, Option<&[&str]>)]) -> Result<&mut Self, String> {
    for &(role, parents) in roles {
      if let Some(parents) = parents {
        self._roles.add_edge(role, parents)?;
      }
      self._roles.add_vertex(role);
    }
    Ok(self)
  }

  /// Adds a resource to the ACL being built.
  ///
  /// # Arguments
  ///
  /// * `resource` - The resource identifier.
  /// * `parents` - Optional slice of parent resource identifiers that this resource inherits from.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let mut builder = AclBuilder::new()
  ///     .add_resource("blog", None)?
  ///     .add_resource("blog_post", Some(&["blog"]))?
  ///     .add_resource("blog_comment", Some(&["blog"]))?;
  /// # Ok::<(), String>(())
  /// ```
  pub fn add_resource(
    &mut self,
    resource: &str,
    parents: Option<&[&str]>,
  ) -> Result<&mut Self, String> {
    self.add_resources(&[(resource, parents)])
  }

  /// Adds multiple resources to the ACL being built.
  ///
  /// # Arguments
  ///
  /// * `resources` - Slice of tuples containing resource identifiers and their optional parent resources.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let mut builder = AclBuilder::new()
  ///   .add_resources(&[
  ///     ("blog", None),
  ///     ("blog_post", Some(&["blog"])),
  ///     ("blog_comment", Some(&["blog"])),
  ///   ])?;
  /// # Ok::<(), String>(())
  /// ```
  pub fn add_resources(
    &mut self,
    resources: &[(&str, Option<&[&str]>)],
  ) -> Result<&mut Self, String> {
    for &(resource, parents) in resources {
      if let Some(parents) = parents {
        self._resources.add_edge(resource, parents)?;
      }
      self._resources.add_vertex(resource);
    }
    Ok(self)
  }

  /// Adds an "allow" rule for the specified roles, resources, and privileges.
  ///
  /// # Arguments
  ///
  /// * `roles` - Optional slice of role identifiers (None means all roles).
  /// * `resources` - Optional slice of resource identifiers (None means all resources).
  /// * `privileges` - Optional slice of privilege identifiers (None means all privileges).
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let mut builder = AclBuilder::new()
  ///     .add_role("user", None)?
  ///     .add_resource("blog", None)?
  ///     .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?;
  /// # Ok::<(), String>(())
  /// ```
  pub fn allow(
    &mut self,
    roles: Option<&[&str]>,
    resources: Option<&[&str]>,
    privileges: Option<&[&str]>,
  ) -> Result<&mut Self, String> {
    self._add_rule(Rule::Allow, roles, resources, privileges);
    Ok(self)
  }

  /// Adds a "deny" rule for the specified roles, resources, and privileges.
  ///
  /// # Arguments
  ///
  /// * `roles` - Optional slice of role identifiers (None means all roles).
  /// * `resources` - Optional slice of resource identifiers (None means all resources).
  /// * `privileges` - Optional slice of privilege identifiers (None means all privileges).
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let acl = AclBuilder::new()
  ///     .add_role("user", None)?
  ///     .add_resource("admin_panel", None)?
  ///     .deny(Some(&["user"]), Some(&["admin_panel"]), None)?;
  /// # Ok::<(), String>(())
  /// ```
  pub fn deny(
    &mut self,
    roles: Option<&[&str]>,
    resources: Option<&[&str]>,
    privileges: Option<&[&str]>,
  ) -> Result<&mut Self, String> {
    self._add_rule(Rule::Deny, roles, resources, privileges);
    Ok(self)
  }

  /// Adds a conditional "allow" rule keyed by `assertion_key`. The caller
  /// supplies an [`AssertionResolver`](crate::simple::AssertionResolver) at
  /// check time (via [`Acl::is_allowed_with`]) to decide whether the key
  /// resolves to `true`.
  ///
  /// When checked via plain [`Acl::is_allowed`] (no resolver), the rule is
  /// treated conservatively — `AllowIf` does NOT allow. Pair with explicit
  /// `Allow` if you need an "always-on" fallback.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let acl = AclBuilder::new()
  ///   .add_role("editor", None)?
  ///   .add_resource("post", None)?
  ///   .allow_if(Some(&["editor"]), Some(&["post"]), Some(&["edit"]), "is_owner")?
  ///   .build()?;
  ///
  /// let resolver = |k: &str| k == "is_owner";
  /// assert!(acl.is_allowed_with(Some("editor"), Some("post"), Some("edit"), &resolver));
  /// # Ok::<(), String>(())
  /// ```
  pub fn allow_if(
    &mut self,
    roles: Option<&[&str]>,
    resources: Option<&[&str]>,
    privileges: Option<&[&str]>,
    assertion_key: &str,
  ) -> Result<&mut Self, String> {
    self._add_rule(
      Rule::AllowIf(assertion_key.to_string()),
      roles,
      resources,
      privileges,
    );
    Ok(self)
  }

  /// Adds a conditional "deny" rule keyed by `assertion_key`. Mirrors
  /// [`allow_if`](AclBuilder::allow_if) semantics. Without a resolver, plain
  /// [`Acl::is_allowed`] treats this as "not-blocking" (no deny fires).
  pub fn deny_if(
    &mut self,
    roles: Option<&[&str]>,
    resources: Option<&[&str]>,
    privileges: Option<&[&str]>,
    assertion_key: &str,
  ) -> Result<&mut Self, String> {
    self._add_rule(
      Rule::DenyIf(assertion_key.to_string()),
      roles,
      resources,
      privileges,
    );
    Ok(self)
  }

  /// Builds and returns the final `Acl` instance.
  ///
  /// This method clones the builder's internal state and performs validation checks on the
  /// constructed ACL (checking for cycles in roles and resources).
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_acl::simple::AclBuilder;
  ///
  /// let acl = AclBuilder::new()
  ///     .add_role("guest", None)?
  ///     .add_resource("blog", None)?
  ///     .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
  ///     .build()?;
  ///
  /// assert!(acl.is_allowed(Some("guest"), Some("blog"), Some("read")));
  /// # Ok::<(), String>(())
  /// ```
  pub fn build(&mut self) -> Result<Acl, String> {
    let acl = Acl::from_parts(
      self._roles.clone(),
      self._resources.clone(),
      self._rules.clone(),
    );

    // Validate the ACL structure
    acl.check_for_cycles()?;

    Ok(acl)
  }

  pub(crate) fn from_parts(
    roles: DisymGraph,
    resources: DisymGraph,
    rules: ResourceRoleRules,
  ) -> Self {
    Self {
      _roles: roles,
      _resources: resources,
      _rules: rules,
    }
  }

  /// Internal method to add a rule to the builder.
  fn _add_rule(
    &mut self,
    rule_type: Rule,
    roles: Option<&[&str]>,
    resources: Option<&[&str]>,
    privileges: Option<&[&str]>,
  ) {
    #[cfg(not(feature = "std"))]
    use alloc::collections::BTreeMap as HashMap;
    #[cfg(feature = "std")]
    use std::collections::HashMap;

    // Special case: if all parameters are empty (None or empty list), reset the entire rules structure
    if _is_empty(&roles) && _is_empty(&resources) && _is_empty(&privileges) {
      self._rules = ResourceRoleRules::new();
      self
        ._rules
        .for_all_resources
        .for_all_roles
        .for_all_privileges = rule_type;
      return;
    }

    // Filter out non-existent roles, and return `vec![None]` if filtered list is empty.
    let _roles: Vec<Option<String>> = self._get_only_keys_in_graph(&self._roles, roles);

    // Filter out non-existent resources, and return `vec![None]` if filtered list is empty.
    // allows using for loops as a `while` loop
    let _resources: Vec<Option<String>> = self._get_only_keys_in_graph(&self._resources, resources);

    // Determine whether the rule we're adding is allowing or denying — used to
    // select the opposing family we need to clear.
    let incoming_is_allowing = rule_type.is_allowing_family();

    // Apply the rule to each resource and role combination
    // ----
    for resource in _resources.iter() {
      for role in _roles.iter() {
        // If `resource` is None, consider it the "received resources are empty" signal,
        // and clear the 'by_resource_id.by_role_id' map for given role
        if resource.is_none() {
          // clear the by_resource_id entries for those roles across all resources
          if let Some(_role) = role.as_deref() {
            for (_, resource_rules) in self._rules.by_resource_id.iter_mut() {
              if let Some(by_role_map) = resource_rules.by_role_id.as_mut() {
                by_role_map.remove(_role);
              }
            }
          }
          // Else if both received both resources and roles "are empty" signal,
          // then clear the resources to role specific maps
          else {
            self._rules.by_resource_id.clear();
            self._rules.for_all_resources.by_role_id = None;
          }
        }
        // If only `role` is None, consider it the "received roles are empty" signal and
        // clear the 'by_role_id' map for this visiting resource
        else if role.is_none()
          && let Some(resource_id) = resource
          && let Some(res_rules) = self._rules.by_resource_id.get_mut(resource_id)
        {
          res_rules.by_role_id = None;
        }

        // Get role rules for resource (will either be "for all roles" or specific role based on Some/None args passed in)
        let role_rules = self._get_role_rules_mut(resource.as_deref(), role.as_deref());

        // Clear opposing-family rules before setting new rule.
        //
        // "Opposing family" = if we're adding Allow/AllowIf, the opposing family
        // is {Deny, DenyIf(_)} and vice versa. Using the family predicates keeps
        // this logic correct across the four Rule variants.
        // ----
        let is_opposing = |existing: &Rule| -> bool {
          if incoming_is_allowing {
            existing.is_denying_family()
          } else {
            existing.is_allowing_family()
          }
        };

        if let Some(privilege_list) = privileges {
          // Clear opposing rule for each specific privilege we're about to set
          if let Some(p_map) = role_rules.by_privilege_id.as_mut() {
            for privilege in privilege_list {
              // Remove the privilege entry if it has a rule in the opposing family
              if let Some(existing_rule) = p_map.get(*privilege)
                && is_opposing(existing_rule)
              {
                p_map.remove(*privilege);
              }
            }
          }
        } else {
          // Setting rule for "all privileges" - clear all opposing rules.
          // (for_all_privileges will be overwritten below anyway.)

          // Clear any specific privilege rules that have an opposing-family rule
          if let Some(p_map) = role_rules.by_privilege_id.as_mut() {
            p_map.retain(|_, rule| !is_opposing(rule));

            // If map is now empty after clearing, set to None for cleanliness
            if p_map.is_empty() {
              role_rules.by_privilege_id = None;
            }
          }
        }

        // Apply privilege rules
        // ----
        if let Some(privilege_list) = privileges {
          // Set rule for each specific privilege
          let p_map = role_rules.by_privilege_id.get_or_insert_with(HashMap::new);
          for privilege in privilege_list {
            p_map.insert(privilege.to_string(), rule_type.clone());
          }
        } else {
          // Set rule for "all privileges" and clear any existing per-privilege rules
          role_rules.for_all_privileges = rule_type.clone();
          role_rules.by_privilege_id = None;
        }
      }
    }
  }

  /// Helper to get keys that exist in a graph, or return vec![None] if input is None or empty.
  fn _get_only_keys_in_graph(
    &self,
    graph: &DisymGraph,
    keys_to_filter: Option<&[&str]>,
  ) -> Vec<Option<String>> {
    keys_to_filter.map_or(vec![None], |keys| {
      keys
        .iter()
        .filter(|key| graph.contains(key))
        .map(|key| Some(key.to_string()))
        .collect()
    })
  }

  /// Helper to get mutable reference to role rules.
  fn _get_role_rules_mut(
    &mut self,
    resource: Option<&str>,
    role: Option<&str>,
  ) -> &mut crate::simple::PrivilegeRules {
    #[cfg(not(feature = "std"))]
    use alloc::collections::BTreeMap as HashMap;
    #[cfg(feature = "std")]
    use std::collections::HashMap;

    use crate::simple::RolePrivilegeRules;

    // Get or create resource rules
    let resource_rules = match resource {
      Some(res_id) => self
        ._rules
        .by_resource_id
        .entry(res_id.to_string())
        .or_insert_with(|| RolePrivilegeRules::new(true)),
      None => &mut self._rules.for_all_resources,
    };

    // Get or create role rules
    match role {
      Some(role_id) => resource_rules
        .by_role_id
        .get_or_insert_with(HashMap::new)
        .entry(role_id.to_string())
        .or_insert_with(|| crate::simple::PrivilegeRules::new(true)),
      None => &mut resource_rules.for_all_roles,
    }
  }
}

impl Default for AclBuilder {
  fn default() -> Self {
    Self::new()
  }
}

/// Attempts conversion of an `Acl` instance into an `AclBuilder`.
///
/// This allows you to take an existing `Acl` and convert it back into a builder,
/// which can be useful for modifying an existing ACL by adding new roles, resources,
/// or rules before building it again.
///
/// # Example
///
/// ```rust
/// use std::convert::TryFrom;
/// use walrs_acl::simple::{Acl, AclBuilder};
///
/// // Create an ACL
/// let acl = AclBuilder::new()
///     .add_role("guest", None)?
///     .add_resource("blog", None)?
///     .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
///     .build()?;
///
/// // Convert it back to a builder, modify it, and rebuild it
/// let modified_acl = AclBuilder::try_from(acl)?
///     .add_role("admin", Some(&["guest"]))?
///     .allow(Some(&["admin"]), None, None)?
///     .build()?;
///
/// assert!(modified_acl.is_allowed(Some("admin"), Some("blog"), Some("write")));
/// # Ok::<(), String>(())
/// ```
impl TryFrom<Acl> for AclBuilder {
  type Error = String;

  fn try_from(acl: Acl) -> Result<Self, Self::Error> {
    // Create a new builder with the ACL's roles, resources, and rules
    let builder = AclBuilder::from_parts(acl._roles, acl._resources, acl._rules);

    Ok(builder)
  }
}

/// Attempts conversion of an `Acl` reference to an `AclBuilder`.
///
/// Enables building ACLs based on existing ones without losing ownership.
///
/// # Example
///
/// ```rust
/// use walrs_acl::simple::{AclBuilder, Acl};
/// use std::convert::TryFrom;
///
/// let acl = AclBuilder::new()
///     .add_role("guest", None)?
///     .add_resource("blog", None)?
///     .allow(Some(&["guest"]), Some(&["blog"]), Some(&["read"]))?
///     .build()?;
///
/// // Build upon the existing ACL
/// let modified_acl = AclBuilder::try_from(&acl)?
///     .add_role("admin", Some(&["guest"]))?
///     .allow(Some(&["admin"]), Some(&["blog"]), Some(&["write"]))?
///     .build()?;
///
/// // Original ACL is still available
/// assert!(acl.is_allowed(Some("guest"), Some("blog"), Some("read")));
/// assert!(modified_acl.is_allowed(Some("admin"), Some("blog"), Some("write")));
/// # Ok::<(), String>(())
/// ```
impl TryFrom<&Acl> for AclBuilder {
  type Error = String;

  fn try_from(acl: &Acl) -> Result<Self, Self::Error> {
    // Clone the internal graphs and rules to create a new builder
    let builder = AclBuilder::from_parts(
      acl._roles.clone(),
      acl._resources.clone(),
      acl._rules.clone(),
    );

    Ok(builder)
  }
}

/// Attempts conversion of an `AclData` reference into an `AclBuilder` -
///
/// Effectively enables loading ACL configuration from JSON or other serialized format
/// (into an `AclData`) and then parsing it into an `AclBuilder`.
///
/// # Example
///
/// ```rust
/// use walrs_acl::simple::{AclBuilder, AclData};
/// use std::convert::TryFrom;
///
/// let acl_data = AclData {
///     roles: Some(vec![
///         ("guest".to_string(), None),
///         ("user".to_string(), Some(vec!["guest".to_string()])),
///     ]),
///     resources: Some(vec![
///         ("blog".to_string(), None),
///     ]),
///     allow: Some(vec![
///         ("blog".to_string(), Some(vec![
///             ("guest".to_string(), Some(vec!["read".to_string()])),
///         ])),
///     ]),
///     deny: None,
///     allow_if: None,
///     deny_if: None,
/// };
///
/// let acl = AclBuilder::try_from(&acl_data)?
///     .add_role("admin", Some(&["user"]))?
///     .allow(Some(&["admin"]), None, None)?
///     .build()?;
///
/// assert!(acl.is_allowed(Some("admin"), Some("blog"), Some("write")));
/// # Ok::<(), String>(())
/// ```
impl<'a> TryFrom<&'a AclData> for AclBuilder {
  type Error = String;

  fn try_from(data: &'a AclData) -> Result<Self, Self::Error> {
    use walrs_digraph::DisymGraph;

    let mut builder = AclBuilder::new();

    // Add `roles` to builder using DisymGraph conversion
    if let Some(roles) = data.roles.as_ref() {
      builder._roles =
        DisymGraph::try_from(roles).map_err(|e| format!("Failed to create roles graph: {}", e))?;
    }

    // Add `resources` to builder using DisymGraph conversion
    if let Some(resources) = data.resources.as_ref() {
      builder._resources = DisymGraph::try_from(resources)
        .map_err(|e| format!("Failed to create resources graph: {}", e))?;
    }

    // Helper function to process rules (works for both allow and deny)
    let process_rules = |builder: &mut AclBuilder,
                         rules: &Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>,
                         is_allow: bool|
     -> Result<(), String> {
      for (resource, roles_and_privileges_assoc_list) in rules.iter() {
        // Handle "*" as "all resources" (None)
        let resource_slice: Option<&[&str]> = if resource == "*" {
          None
        } else {
          Some(&[resource.as_str()])
        };

        // If `(roles, privileges)` associative list exists, loop through it
        if let Some(rs_and_ps_list) = roles_and_privileges_assoc_list {
          for (role, privileges) in rs_and_ps_list.iter() {
            // Handle "*" as "all roles" (None)
            let role_slice: Option<&[&str]> = if role == "*" {
              None
            } else {
              Some(&[role.as_str()])
            };

            let ps: Option<Vec<&str>> = privileges
              .as_deref()
              .map(|ps| ps.iter().map(|p| &**p).collect());

            // Apply rule based on type
            if is_allow {
              builder.allow(role_slice, resource_slice, ps.as_deref())?;
            } else {
              builder.deny(role_slice, resource_slice, ps.as_deref())?;
            }
          }
        } else {
          // Apply rule for all roles and privileges on the given resource
          if is_allow {
            builder.allow(None, resource_slice, None)?;
          } else {
            builder.deny(None, resource_slice, None)?;
          }
        }
      }
      Ok(())
    };

    // Helper: process conditional rules (allow_if / deny_if). The innermost
    // list is `(privilege_name, assertion_key)` tuples instead of privilege
    // strings; we have to split by assertion_key because one call to
    // `allow_if` / `deny_if` on the builder can only accept one assertion key.
    #[cfg(not(feature = "std"))]
    use alloc::collections::BTreeMap as KeyMap;
    #[cfg(feature = "std")]
    use std::collections::HashMap as KeyMap;
    let process_conditional_rules = |builder: &mut AclBuilder,
                                     rules: &Vec<(
      String,
      Option<Vec<(String, Option<Vec<(String, String)>>)>>,
    )>,
                                     is_allow: bool|
     -> Result<(), String> {
      for (resource, roles_and_privileges_assoc_list) in rules.iter() {
        let resource_slice: Option<&[&str]> = if resource == "*" {
          None
        } else {
          Some(&[resource.as_str()])
        };

        if let Some(rs_and_ps_list) = roles_and_privileges_assoc_list {
          for (role, privileges_with_keys) in rs_and_ps_list.iter() {
            let role_slice: Option<&[&str]> = if role == "*" {
              None
            } else {
              Some(&[role.as_str()])
            };

            match privileges_with_keys.as_deref() {
              Some(pairs) => {
                // Group privileges by assertion key so we can issue one
                // builder call per key with all applicable privileges.
                let mut by_key: KeyMap<&str, Vec<&str>> = KeyMap::new();
                for (priv_name, key) in pairs.iter() {
                  by_key.entry(key.as_str()).or_default().push(priv_name.as_str());
                }
                for (key, ps) in by_key.into_iter() {
                  if is_allow {
                    builder.allow_if(role_slice, resource_slice, Some(ps.as_slice()), key)?;
                  } else {
                    builder.deny_if(role_slice, resource_slice, Some(ps.as_slice()), key)?;
                  }
                }
              }
              None => {
                // If no pairs are given for this role, skip — a conditional
                // rule without any assertion key is meaningless.
              }
            }
          }
        }
        // Note: we intentionally do not treat `None` at the outer level as
        // "apply to all roles" for conditional rules — there is no assertion
        // key to bind to in that shape.
      }
      Ok(())
    };

    // Add `allow` rules to builder, if any
    if let Some(allow) = data.allow.as_ref() {
      process_rules(&mut builder, allow, true)?;
    }

    // Add `deny` rules to builder, if any
    if let Some(deny) = data.deny.as_ref() {
      process_rules(&mut builder, deny, false)?;
    }

    // Add conditional `allow_if` rules to builder, if any
    if let Some(allow_if) = data.allow_if.as_ref() {
      process_conditional_rules(&mut builder, allow_if, true)?;
    }

    // Add conditional `deny_if` rules to builder, if any
    if let Some(deny_if) = data.deny_if.as_ref() {
      process_conditional_rules(&mut builder, deny_if, false)?;
    }

    // Return the builder (without calling .build())
    Ok(builder)
  }
}

/// Attempts conversion of an `AclData` reference into an `AclBuilder`.
///
/// # Example
///
/// ```rust
/// use walrs_acl::simple::{AclBuilder, AclData};
/// use std::convert::TryFrom;
/// use std::fs::File;
///
/// let file_path = "./test-fixtures/example-acl.json";
/// let mut f = File::open(&file_path)?;
/// let acl_data = AclData::try_from(&mut f)?;
///
/// let acl = AclBuilder::try_from(acl_data)?
///     .add_role("extra_role", None)?
///     .build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
impl TryFrom<AclData> for AclBuilder {
  type Error = String;

  fn try_from(data: AclData) -> Result<Self, Self::Error> {
    AclBuilder::try_from(&data)
  }
}

// TODO finalize implementation (still in progress).
impl TryFrom<&AclBuilder> for AclData {
  type Error = String;

  fn try_from(builder: &AclBuilder) -> Result<Self, Self::Error> {
    #[cfg(not(feature = "std"))]
    use alloc::collections::BTreeMap as HashMap;
    #[cfg(feature = "std")]
    use std::collections::HashMap;

    use walrs_digraph::DisymGraphData;

    // Extract roles with their parents using DisymGraph conversion
    let roles = if builder._roles.vert_count() > 0 {
      Some(
        DisymGraphData::try_from(&builder._roles)
          .map_err(|e| format!("Failed to extract roles: {}", e))?,
      )
    } else {
      None
    };

    // Extract resources with their parents using DisymGraph conversion
    let resources = if builder._resources.vert_count() > 0 {
      Some(
        DisymGraphData::try_from(&builder._resources)
          .map_err(|e| format!("Failed to extract resources: {}", e))?,
      )
    } else {
      None
    };

    // Helper to extract unconditional rules (Allow / Deny) from RolePrivilegeRules.
    let extract_rules = |role_priv_rules: &crate::simple::RolePrivilegeRules,
                         match_rule: &crate::simple::Rule|
     -> Option<Vec<(String, Option<Vec<String>>)>> {
      let mut role_rules = HashMap::new();

      // Check "for all roles" rules
      let by_priv_is_empty = role_priv_rules
        .for_all_roles
        .by_privilege_id
        .as_ref()
        .is_none_or(|m| m.is_empty());

      if !by_priv_is_empty {
        if let Some(ref by_priv) = role_priv_rules.for_all_roles.by_privilege_id {
          let matching_privileges: Vec<String> = by_priv
            .iter()
            .filter(|(_, rule)| *rule == match_rule)
            .map(|(k, _)| k.to_string())
            .collect();
          if !matching_privileges.is_empty() {
            role_rules.insert("*".to_string(), Some(matching_privileges));
          }
        }
      } else if &role_priv_rules.for_all_roles.for_all_privileges == match_rule {
        // Only insert for Allow rules (Deny is the default, so we don't capture it unless explicit)
        if match_rule == &crate::simple::Rule::Allow {
          role_rules.insert("*".to_string(), None);
        }
      }

      // Check per-role rules
      if let Some(ref by_role) = role_priv_rules.by_role_id {
        for (role, priv_rules) in by_role.iter() {
          let role_by_priv_is_empty = priv_rules
            .by_privilege_id
            .as_ref()
            .is_none_or(|m| m.is_empty());

          if !role_by_priv_is_empty {
            if let Some(ref by_priv) = priv_rules.by_privilege_id {
              let matching_privileges: Vec<String> = by_priv
                .iter()
                .filter(|(_, rule)| *rule == match_rule)
                .map(|(k, _)| k.to_string())
                .collect();
              if !matching_privileges.is_empty() {
                role_rules.insert(role.clone(), Some(matching_privileges));
              }
            }
          } else if &priv_rules.for_all_privileges == match_rule {
            role_rules.insert(role.clone(), None);
          }
        }
      }

      if role_rules.is_empty() {
        None
      } else {
        Some(role_rules.into_iter().collect())
      }
    };

    // Helper to extract conditional rules (AllowIf / DenyIf). Matches on the
    // variant; the `(privilege, assertion_key)` pairs are the inner items.
    // `want_allow_if = true` => AllowIf, false => DenyIf.
    let extract_conditional_rules =
      |role_priv_rules: &crate::simple::RolePrivilegeRules, want_allow_if: bool|
       -> Option<Vec<(String, Option<Vec<(String, String)>>)>> {
        let mut role_rules: HashMap<String, Option<Vec<(String, String)>>> = HashMap::new();

        let variant_matches = |rule: &crate::simple::Rule| -> Option<String> {
          match (rule, want_allow_if) {
            (crate::simple::Rule::AllowIf(k), true) => Some(k.clone()),
            (crate::simple::Rule::DenyIf(k), false) => Some(k.clone()),
            _ => None,
          }
        };

        // Check "for all roles" per-privilege rules
        if let Some(ref by_priv) = role_priv_rules.for_all_roles.by_privilege_id {
          let matches: Vec<(String, String)> = by_priv
            .iter()
            .filter_map(|(pname, rule)| variant_matches(rule).map(|k| (pname.to_string(), k)))
            .collect();
          if !matches.is_empty() {
            role_rules.insert("*".to_string(), Some(matches));
          }
        }
        // Per-role "for all privileges" conditional rules: also check those.
        if let Some(k) = variant_matches(&role_priv_rules.for_all_roles.for_all_privileges) {
          // No privilege name makes sense for "all privileges"; use empty
          // privilege-name slot paired with key. We skip this — the data
          // format requires a privilege name, so these are not representable.
          // Log via a no-op; user who cares will supply per-privilege rules.
          let _ = k;
        }

        // Check per-role rules
        if let Some(ref by_role) = role_priv_rules.by_role_id {
          for (role, priv_rules) in by_role.iter() {
            if let Some(ref by_priv) = priv_rules.by_privilege_id {
              let matches: Vec<(String, String)> = by_priv
                .iter()
                .filter_map(|(pname, rule)| variant_matches(rule).map(|k| (pname.to_string(), k)))
                .collect();
              if !matches.is_empty() {
                role_rules.insert(role.clone(), Some(matches));
              }
            }
            // Skipping per-role "for_all_privileges" conditional rules for the
            // same reason as above — not representable without a privilege id.
          }
        }

        if role_rules.is_empty() {
          None
        } else {
          Some(role_rules.into_iter().collect())
        }
      };

    // Extract allow rules
    let mut allow_map: HashMap<String, Option<Vec<(String, Option<Vec<String>>)>>> = HashMap::new();

    let for_all_allow =
      extract_rules(&builder._rules.for_all_resources, &crate::simple::Rule::Allow);
    if for_all_allow.is_some() {
      allow_map.insert("*".to_string(), for_all_allow);
    }

    for (resource, role_priv_rules) in builder._rules.by_resource_id.iter() {
      let resource_allow = extract_rules(role_priv_rules, &crate::simple::Rule::Allow);
      if resource_allow.is_some() {
        allow_map.insert(resource.clone(), resource_allow);
      }
    }

    let allow = if allow_map.is_empty() {
      None
    } else {
      Some(allow_map.into_iter().collect())
    };

    // Extract deny rules
    let mut deny_map: HashMap<String, Option<Vec<(String, Option<Vec<String>>)>>> = HashMap::new();

    let for_all_deny =
      extract_rules(&builder._rules.for_all_resources, &crate::simple::Rule::Deny);
    if for_all_deny.is_some() {
      deny_map.insert("*".to_string(), for_all_deny);
    }

    for (resource, role_priv_rules) in builder._rules.by_resource_id.iter() {
      let resource_deny = extract_rules(role_priv_rules, &crate::simple::Rule::Deny);
      if resource_deny.is_some() {
        deny_map.insert(resource.clone(), resource_deny);
      }
    }

    let deny = if deny_map.is_empty() {
      None
    } else {
      Some(deny_map.into_iter().collect())
    };

    // Extract allow_if rules
    let mut allow_if_map: HashMap<String, Option<Vec<(String, Option<Vec<(String, String)>>)>>> =
      HashMap::new();

    let for_all_allow_if = extract_conditional_rules(&builder._rules.for_all_resources, true);
    if for_all_allow_if.is_some() {
      allow_if_map.insert("*".to_string(), for_all_allow_if);
    }

    for (resource, role_priv_rules) in builder._rules.by_resource_id.iter() {
      let r_allow_if = extract_conditional_rules(role_priv_rules, true);
      if r_allow_if.is_some() {
        allow_if_map.insert(resource.clone(), r_allow_if);
      }
    }

    let allow_if = if allow_if_map.is_empty() {
      None
    } else {
      Some(allow_if_map.into_iter().collect())
    };

    // Extract deny_if rules
    let mut deny_if_map: HashMap<String, Option<Vec<(String, Option<Vec<(String, String)>>)>>> =
      HashMap::new();

    let for_all_deny_if = extract_conditional_rules(&builder._rules.for_all_resources, false);
    if for_all_deny_if.is_some() {
      deny_if_map.insert("*".to_string(), for_all_deny_if);
    }

    for (resource, role_priv_rules) in builder._rules.by_resource_id.iter() {
      let r_deny_if = extract_conditional_rules(role_priv_rules, false);
      if r_deny_if.is_some() {
        deny_if_map.insert(resource.clone(), r_deny_if);
      }
    }

    let deny_if = if deny_if_map.is_empty() {
      None
    } else {
      Some(deny_if_map.into_iter().collect())
    };

    Ok(AclData {
      roles,
      resources,
      allow,
      deny,
      allow_if,
      deny_if,
    })
  }
}

/// Attempts conversion of a mutable file reference into an `AclBuilder`.
///
/// # Example
///
/// ```rust
/// use walrs_acl::simple::AclBuilder;
/// use std::convert::TryFrom;
/// use std::fs::File;
///
/// let file_path = "./test-fixtures/example-acl.json";
/// let mut f = File::open(&file_path)?;
///
/// let acl = AclBuilder::try_from(&mut f)?
///     .add_role("extra_role", None)?
///     .allow(Some(&["extra_role"]), None, None)?
///     .build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[cfg(feature = "std")]
impl TryFrom<&mut File> for AclBuilder {
  type Error = serde_json::Error;

  fn try_from(file: &mut File) -> Result<Self, Self::Error> {
    AclData::try_from(file).and_then(|data| {
      AclBuilder::try_from(&data)
        .map_err(|e| serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    })
  }
}

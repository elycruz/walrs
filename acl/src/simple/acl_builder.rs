use core::convert::TryFrom;
use crate::prelude::{String, Vec, vec, ToString, format};

#[cfg(feature = "std")]
use std::fs::File;

use walrs_digraph::DisymGraph;
use crate::simple::{Acl, AclData, ResourceRoleRules, Rule};

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
        if let Some(parents) = parents {
            self._roles.add_edge(role, parents)?;
        }
        self._roles.add_vertex(role);
        Ok(self)
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
    pub fn add_resource(&mut self, resource: &str, parents: Option<&[&str]>) -> Result<&mut Self, String> {
        if let Some(parents) = parents {
            self._resources.add_edge(resource, parents)?;
        }
        self._resources.add_vertex(resource);
        Ok(self)
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
    pub fn add_resources(&mut self, resources: &[(&str, Option<&[&str]>)]) -> Result<&mut Self, String> {
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
        let acl = Acl::from_parts(self._roles.clone(), self._resources.clone(), self._rules.clone());

        // Validate the ACL structure
        acl.check_for_cycles()?;

        Ok(acl)
    }

    pub(crate) fn from_parts(roles: DisymGraph, resources: DisymGraph, rules: ResourceRoleRules) -> Self {
        Self { _roles: roles, _resources: resources, _rules: rules }
    }

    /// Internal method to add a rule to the builder.
    fn _add_rule(
        &mut self,
        rule_type: Rule,
        roles: Option<&[&str]>,
        resources: Option<&[&str]>,
        privileges: Option<&[&str]>,
    ) {
        #[cfg(feature = "std")]
        use std::collections::HashMap;
        #[cfg(not(feature = "std"))]
        use alloc::collections::BTreeMap as HashMap;

        // TODO: Consolidate/clean-up this implementation.

        // Apply overwrite/clearing logic
        // ----
        // Special case: if all parameters are None, reset the entire rules structure
        if roles.is_none() && resources.is_none() && privileges.is_none() {
            self._rules = ResourceRoleRules::new();
            self._rules.for_all_resources.for_all_roles.for_all_privileges = rule_type;
            return;
        }

        // Filter out non-existent roles, and return `vec![None]` if result is empty list, or `None`.
        let _roles: Vec<Option<String>> = self._get_only_keys_in_graph(&self._roles, roles);

        // Filter out non-existent resources, and return `vec![None]` if result is empty list, or `None`
        let _resources: Vec<Option<String>> = self._get_only_keys_in_graph(&self._resources, resources);

        for resource in _resources.iter() {
            // If resources is None (for all resources), we need to handle role clearing
            if resources.is_none() {
                for role in _roles.iter() {
                    if let Some(role_id) = role {
                        // When setting a rule for "all resources" on a specific role,
                        // clear the by_resource_id entries for this role across all resources
                        for (_, resource_rules) in self._rules.by_resource_id.iter_mut() {
                            if let Some(by_role_map) = resource_rules.by_role_id.as_mut() {
                                by_role_map.remove(role_id);
                            }
                        }
                    }
                }
            } else {
                // If resources is Some (specific resources), and we're setting a rule on specific roles,
                // we need to clear the "for all resources" rule for those roles to avoid conflicts
                for role in _roles.iter() {
                    if let Some(role_id) = role {
                        // Clear the "for all resources" rule for this specific role
                        if let Some(for_all_by_role) = self._rules.for_all_resources.by_role_id.as_mut() {
                            for_all_by_role.remove(role_id);
                        }
                    }
                }
            }

            for role in _roles.iter() {
                // Get role rules for resource
                let role_rules = self._get_role_rules_mut(resource.as_deref(), role.as_deref());

                // If 'privileges' is `None`, set 'rule' for "all privileges"
                // and clear any existing per-privilege rules
                if privileges.is_none() {
                    role_rules.for_all_privileges = rule_type;
                    // Clear out any per-privilege rules to avoid conflicts
                    role_rules.by_privilege_id = None;
                    continue;
                }
                // Else loop through privileges, and insert 'rule' for each privilege
                privileges.unwrap().iter().for_each(|privilege| {
                    // Get privilege map for role and insert rule
                    let p_map = role_rules.by_privilege_id.get_or_insert(HashMap::new());

                    // Insert rule
                    p_map.insert(privilege.to_string(), rule_type);
                });
            }

            // If roles is None (e.g., for all roles rule), clear the 'by_role_id' map for this resource
            if roles.is_none() && resource.is_some() {
                let resource_rules = self._rules.by_resource_id.get_mut(resource.as_ref().unwrap());
                if let Some(res_rules) = resource_rules {
                    res_rules.by_role_id = None;
                }
            }
        }
        // If 'resources', and 'roles', are `None` clear the `by_resource_id`, and
        // `by_role_id`, maps.
        if resources.is_none() && roles.is_none() {
            self._rules.by_resource_id.clear();
            self._rules.for_all_resources.by_role_id = None;
        }
    }

    /// Helper to get keys that exist in a graph, or return vec![None] if input is None or empty.
    fn _get_only_keys_in_graph(
        &self,
        graph: &DisymGraph,
        keys_to_filter: Option<&[&str]>,
    ) -> Vec<Option<String>> {
        keys_to_filter.map_or(vec![None], |keys| {
            keys.iter()
                .filter(|key| graph.contains(*key))
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
        #[cfg(feature = "std")]
        use std::collections::HashMap;
        #[cfg(not(feature = "std"))]
        use alloc::collections::BTreeMap as HashMap;

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
            acl._rules.clone()
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
            builder._roles = DisymGraph::try_from(roles)
                .map_err(|e| format!("Failed to create roles graph: {}", e))?;
        }

        // Add `resources` to builder using DisymGraph conversion
        if let Some(resources) = data.resources.as_ref() {
            builder._resources = DisymGraph::try_from(resources)
                .map_err(|e| format!("Failed to create resources graph: {}", e))?;
        }

        // Add `allow` rules to builder, if any
        if let Some(allow) = data.allow.as_ref() {
            // For entry in allow rules
            for (resource, roles_and_privileges_assoc_list) in allow.iter() {
                // Handle "*" as "all resources" (None)
                let resource_slice: Option<&[&str]> = if resource == "*" {
                    None
                } else {
                    Some(&[resource.as_str()])
                };

                // If `(roles, privileges)` associative list loop through it`
                if let Some(rs_and_ps_list) = roles_and_privileges_assoc_list {
                    // For each entry in `role -> privilege` list
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
                        // Apply `allow` rule
                        builder.allow(
                            role_slice,
                            resource_slice,
                            ps.as_deref(),
                        )?;
                    }
                }
                // Else add allow rule for all `roles`, on all `privileges`, for given `resource`
                else {
                    builder.allow(None, resource_slice, None)?;
                }
            }
        }

        // Add `deny` rules to builder, if any
        if let Some(deny) = data.deny.as_ref() {
            for (resource, roles_and_privileges_assoc_list) in deny.iter() {
                // Handle "*" as "all resources" (None)
                let resource_slice: Option<&[&str]> = if resource == "*" {
                    None
                } else {
                    Some(&[resource.as_str()])
                };

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
                        builder.deny(
                            role_slice,
                            resource_slice,
                            ps.as_deref(),
                        )?;
                    }
                } else {
                    builder.deny(None, resource_slice, None)?;
                }
            }
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
        #[cfg(feature = "std")]
        use std::collections::HashMap;
        #[cfg(not(feature = "std"))]
        use alloc::collections::BTreeMap as HashMap;

        use walrs_digraph::DisymGraphData;

        // Extract roles with their parents using DisymGraph conversion
        let roles = if builder._roles.vert_count() > 0 {
            Some(DisymGraphData::try_from(&builder._roles)
                .map_err(|e| format!("Failed to extract roles: {}", e))?)
        } else {
            None
        };

        // Extract resources with their parents using DisymGraph conversion
        let resources = if builder._resources.vert_count() > 0 {
            Some(DisymGraphData::try_from(&builder._resources)
                .map_err(|e| format!("Failed to extract resources: {}", e))?)
        } else {
            None
        };

        // Helper to extract rules from RolePrivilegeRules
        let extract_rules = |role_priv_rules: &crate::simple::RolePrivilegeRules| -> Option<Vec<(String, Option<Vec<String>>)>> {
            let mut role_rules = HashMap::new();

            // Check "for all roles" rules
            if let Some(ref by_priv) = role_priv_rules.for_all_roles.by_privilege_id {
                if !by_priv.is_empty() {
                    let privileges: Vec<String> = by_priv.keys().map(|k| k.to_string()).collect();
                    if !privileges.is_empty() {
                        role_rules.insert("*".to_string(), Some(privileges));
                    }
                }
            } else if role_priv_rules.for_all_roles.for_all_privileges == crate::simple::Rule::Allow {
                role_rules.insert("*".to_string(), None);
            }

            // Check per-role rules
            if let Some(ref by_role) = role_priv_rules.by_role_id {
                for (role, priv_rules) in by_role.iter() {
                    if let Some(ref by_priv) = priv_rules.by_privilege_id {
                        if !by_priv.is_empty() {
                            let privileges: Vec<String> = by_priv.keys().map(|k| k.to_string()).collect();
                            role_rules.insert(role.clone(), Some(privileges));
                        }
                    } else if priv_rules.for_all_privileges == crate::simple::Rule::Allow {
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

        // Extract allow rules
        let mut allow_map: HashMap<String, Option<Vec<(String, Option<Vec<String>>)>>> = HashMap::new();

        // Check "for all resources" allow rules
        let for_all_allow = extract_rules(&builder._rules.for_all_resources);
        if for_all_allow.is_some() {
            allow_map.insert("*".to_string(), for_all_allow);
        }

        // Check per-resource allow rules
        for (resource, role_priv_rules) in builder._rules.by_resource_id.iter() {
            let resource_allow = extract_rules(role_priv_rules);
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
        // Note: The current structure primarily tracks Allow rules explicitly.
        // Deny rules would need similar extraction logic based on Rule::Deny values.
        let mut deny_map: HashMap<String, Option<Vec<(String, Option<Vec<String>>)>>> = HashMap::new();

        // Helper to extract deny rules from RolePrivilegeRules
        let extract_deny_rules = |role_priv_rules: &crate::simple::RolePrivilegeRules| -> Option<Vec<(String, Option<Vec<String>>)>> {
            let mut role_rules = HashMap::new();

            // Check "for all roles" deny rules
            if let Some(ref by_priv) = role_priv_rules.for_all_roles.by_privilege_id {
                let deny_privileges: Vec<String> = by_priv.iter()
                    .filter(|(_, rule)| **rule == crate::simple::Rule::Deny)
                    .map(|(k, _)| k.to_string())
                    .collect();
                if !deny_privileges.is_empty() {
                    role_rules.insert("*".to_string(), Some(deny_privileges));
                }
            } else if role_priv_rules.for_all_roles.for_all_privileges == crate::simple::Rule::Deny {
                // Only include explicit denies, not default denies
                // Skip this as it's the default state
            }

            // Check per-role deny rules
            if let Some(ref by_role) = role_priv_rules.by_role_id {
                for (role, priv_rules) in by_role.iter() {
                    if let Some(ref by_priv) = priv_rules.by_privilege_id {
                        let deny_privileges: Vec<String> = by_priv.iter()
                            .filter(|(_, rule)| **rule == crate::simple::Rule::Deny)
                            .map(|(k, _)| k.to_string())
                            .collect();
                        if !deny_privileges.is_empty() {
                            role_rules.insert(role.clone(), Some(deny_privileges));
                        }
                    } else if priv_rules.for_all_privileges == crate::simple::Rule::Deny {
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

        // Check "for all resources" deny rules
        let for_all_deny = extract_deny_rules(&builder._rules.for_all_resources);
        if for_all_deny.is_some() {
            deny_map.insert("*".to_string(), for_all_deny);
        }

        // Check per-resource deny rules
        for (resource, role_priv_rules) in builder._rules.by_resource_id.iter() {
            let resource_deny = extract_deny_rules(role_priv_rules);
            if resource_deny.is_some() {
                deny_map.insert(resource.clone(), resource_deny);
            }
        }

        let deny = if deny_map.is_empty() {
            None
        } else {
            Some(deny_map.into_iter().collect())
        };

        Ok(AclData {
            roles,
            resources,
            allow,
            deny,
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
            AclBuilder::try_from(&data).map_err(|e| {
                serde_json::Error::io(
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e)
                )
            })
        })
    }
}

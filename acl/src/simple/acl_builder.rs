use std::convert::TryFrom;
use walrs_graph::digraph::DisymGraph;
use crate::simple::{Acl, ResourceRoleRules, Rule};

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
    /// let builder = AclBuilder::new()
    ///     .add_role("guest", None)?
    ///     .add_role("user", Some(&["guest"]))?
    ///     .add_role("admin", Some(&["user"]))?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn add_role(mut self, role: &str, parents: Option<&[&str]>) -> Result<Self, String> {
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
    /// let builder = AclBuilder::new()
    ///     .add_roles(&[
    ///         ("guest", None),
    ///         ("user", Some(&["guest"])),
    ///         ("admin", Some(&["user"])),
    ///     ])?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn add_roles(mut self, roles: &[(&str, Option<&[&str]>)]) -> Result<Self, String> {
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
    /// let builder = AclBuilder::new()
    ///     .add_resource("blog", None)?
    ///     .add_resource("blog_post", Some(&["blog"]))?
    ///     .add_resource("blog_comment", Some(&["blog"]))?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn add_resource(mut self, resource: &str, parents: Option<&[&str]>) -> Result<Self, String> {
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
    /// let builder = AclBuilder::new()
    ///     .add_resources(&[
    ///         ("blog", None),
    ///         ("blog_post", Some(&["blog"])),
    ///         ("blog_comment", Some(&["blog"])),
    ///     ])?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn add_resources(mut self, resources: &[(&str, Option<&[&str]>)]) -> Result<Self, String> {
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
    /// let builder = AclBuilder::new()
    ///     .add_role("user", None)?
    ///     .add_resource("blog", None)?
    ///     .allow(Some(&["user"]), Some(&["blog"]), Some(&["read", "write"]))?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn allow(
        mut self,
        roles: Option<&[&str]>,
        resources: Option<&[&str]>,
        privileges: Option<&[&str]>,
    ) -> Result<Self, String> {
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
    /// let builder = AclBuilder::new()
    ///     .add_role("user", None)?
    ///     .add_resource("admin_panel", None)?
    ///     .deny(Some(&["user"]), Some(&["admin_panel"]), None)?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn deny(
        mut self,
        roles: Option<&[&str]>,
        resources: Option<&[&str]>,
        privileges: Option<&[&str]>,
    ) -> Result<Self, String> {
        self._add_rule(Rule::Deny, roles, resources, privileges);
        Ok(self)
    }

    /// Builds and returns the final `Acl` instance.
    ///
    /// This method consumes the builder and performs validation checks on the
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
    pub fn build(self) -> Result<Acl, String> {
        let acl = Acl::from_parts(self._roles, self._resources, self._rules);

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
        use std::collections::HashMap;

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

/// Converts an `Acl` instance into an `AclBuilder`.
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
/// // Convert it back to a builder
/// let builder = AclBuilder::try_from(acl)?;
///
/// // Modify and rebuild
/// let modified_acl = builder
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

/// Implements conversion from a reference to an `Acl` to an `AclBuilder`.
/// 
/// This allows building a new ACL based on an existing one without taking ownership.
/// The internal graphs and rules are cloned from the source ACL.
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
/// let builder = AclBuilder::try_from(&acl)?;
/// let modified_acl = builder
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

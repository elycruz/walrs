//! WebAssembly bindings for walrs_acl
//!
//! This module provides JavaScript-compatible wrappers around the ACL structure from
//! the walrs_acl crate.

use wasm_bindgen::prelude::*;
use crate::simple::{Acl, AclBuilder, AclData};
use crate::prelude::{String, Vec, format};

/// JavaScript-compatible wrapper for Acl
#[wasm_bindgen]
pub struct JsAcl {
    inner: Acl,
}

#[wasm_bindgen]
impl JsAcl {
    /// Creates a new empty ACL
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Acl::new(),
        }
    }

    /// Creates an ACL from a JSON string
    ///
    /// # Example JSON format
    /// ```json
    /// {
    ///   "roles": [["guest", null], ["user", ["guest"]]],
    ///   "resources": [["blog", null]],
    ///   "allow": [["blog", [["guest", ["read"]]]]]
    /// }
    /// ```
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<JsAcl, JsValue> {
        let acl_data: AclData = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;

        let acl = AclBuilder::try_from(&acl_data)
            .map_err(|e| JsValue::from_str(&e))?
            .build()
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(JsAcl { inner: acl })
    }

    /// Converts the ACL to a JSON string
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        // Note: Direct serialization of Acl is not implemented
        // This would require implementing Serialize for Acl
        Err(JsValue::from_str("Direct ACL serialization not yet implemented. Use AclBuilder instead."))
    }

    /// Checks if a role is allowed to perform an action on a resource
    ///
    /// # Arguments
    /// * `role` - The role name (e.g., "user", "admin"). Pass null for "all roles"
    /// * `resource` - The resource name (e.g., "blog", "admin_panel"). Pass null for "all resources"
    /// * `privilege` - The privilege/action name (e.g., "read", "write"). Pass null for "all privileges"
    ///
    /// # Returns
    /// `true` if the permission is allowed, `false` otherwise
    #[wasm_bindgen(js_name = isAllowed)]
    pub fn is_allowed(&self, role: Option<String>, resource: Option<String>, privilege: Option<String>) -> bool {
        self.inner.is_allowed(
            role.as_deref(),
            resource.as_deref(),
            privilege.as_deref(),
        )
    }

    /// Checks if any of the roles is allowed any of the privileges on any of the resources
    ///
    /// # Arguments
    /// * `roles` - Array of role names (null means "all roles")
    /// * `resources` - Array of resource names (null means "all resources")
    /// * `privileges` - Array of privilege names (null means "all privileges")
    #[wasm_bindgen(js_name = isAllowedAny)]
    pub fn is_allowed_any(
        &self,
        roles: Option<Vec<String>>,
        resources: Option<Vec<String>>,
        privileges: Option<Vec<String>>
    ) -> bool {
        let role_refs: Option<Vec<&str>> = roles.as_ref().map(|r| {
            r.iter().map(|s| s.as_str()).collect()
        });
        let resource_refs: Option<Vec<&str>> = resources.as_ref().map(|r| {
            r.iter().map(|s| s.as_str()).collect()
        });
        let privilege_refs: Option<Vec<&str>> = privileges.as_ref().map(|p| {
            p.iter().map(|s| s.as_str()).collect()
        });

        self.inner.is_allowed_any(
            role_refs.as_deref(),
            resource_refs.as_deref(),
            privilege_refs.as_deref(),
        )
    }

    /// Checks if the ACL contains a specific role
    #[wasm_bindgen(js_name = hasRole)]
    pub fn has_role(&self, role: &str) -> bool {
        self.inner.has_role(role)
    }

    /// Checks if the ACL contains a specific resource
    #[wasm_bindgen(js_name = hasResource)]
    pub fn has_resource(&self, resource: &str) -> bool {
        self.inner.has_resource(resource)
    }

    /// Checks if one role inherits from another
    #[wasm_bindgen(js_name = inheritsRole)]
    pub fn inherits_role(&self, role: &str, inherits: &str) -> bool {
        self.inner.inherits_role(role, inherits)
    }

    /// Checks if one resource inherits from another
    #[wasm_bindgen(js_name = inheritsResource)]
    pub fn inherits_resource(&self, resource: &str, inherits: &str) -> bool {
        self.inner.inherits_resource(resource, inherits)
    }
}

/// JavaScript-compatible wrapper for AclBuilder
#[wasm_bindgen]
pub struct JsAclBuilder {
    inner: AclBuilder,
}

#[wasm_bindgen]
impl JsAclBuilder {
    /// Creates a new ACL builder
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: AclBuilder::new(),
        }
    }

    /// Creates an ACL builder from a JSON string
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<JsAclBuilder, JsValue> {
        let acl_data: AclData = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;

        let builder = AclBuilder::try_from(&acl_data)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(JsAclBuilder { inner: builder })
    }

    /// Adds a role to the ACL
    ///
    /// # Arguments
    /// * `role` - The role name
    /// * `parents` - Optional array of parent role names this role inherits from
    #[wasm_bindgen(js_name = addRole)]
    pub fn add_role(&mut self, role: &str, parents: Option<Vec<String>>) {
        let parent_refs: Option<Vec<&str>> = parents.as_ref().map(|p| {
            p.iter().map(|s| s.as_str()).collect()
        });

        self.inner.add_role(role, parent_refs.as_deref())
            .unwrap_or_else(|e| panic!("{}", e));
    }

    /// Adds multiple roles to the ACL
    ///
    /// # Arguments
    /// * `roles` - Array of tuples [roleName, parentRoles[]] where parentRoles can be null
    ///
    /// # Example
    /// ```javascript
    /// builder.addRoles([
    ///     ["guest", null],
    ///     ["user", ["guest"]],
    ///     ["admin", ["user"]]
    /// ])
    /// ```
    #[wasm_bindgen(js_name = addRoles)]
    pub fn add_roles(&mut self, roles: Vec<JsValue>) {
        use serde_wasm_bindgen::from_value;

        // Parse the array of [string, string[] | null] tuples
        let roles_data: Vec<(String, Option<Vec<String>>)> = from_value(JsValue::from(roles))
            .unwrap_or_else(|e| panic!("Failed to parse 'roles' array: {:?}", e));

        for (role, parents) in roles_data {
            let parent_refs: Option<Vec<&str>> = parents.as_ref().map(|p| {
                p.iter().map(|s| s.as_str()).collect()
            });

            self.inner.add_role(&role, parent_refs.as_deref())
                .unwrap_or_else(|e| panic!("{}", e));
        }
    }

    /// Adds a resource to the ACL
    ///
    /// # Arguments
    /// * `resource` - The resource name
    /// * `parents` - Optional array of parent resource names this resource inherits from
    #[wasm_bindgen(js_name = addResource)]
    pub fn add_resource(&mut self, resource: &str, parents: Option<Vec<String>>) {
        let parent_refs: Option<Vec<&str>> = parents.as_ref().map(|p| {
            p.iter().map(|s| s.as_str()).collect()
        });

        self.inner.add_resource(resource, parent_refs.as_deref())
            .unwrap_or_else(|e| panic!("{}", e));
    }

    /// Adds multiple resources to the ACL
    ///
    /// # Arguments
    /// * `resources` - Array of tuples [resourceName, parentResources[]] where parentResources can be null
    ///
    /// # Example
    /// ```javascript
    /// builder.addResources([
    ///     ["index", null],
    ///     ["blog", ["index"]],
    ///     ["admin_panel", null]
    /// ])
    /// ```
    #[wasm_bindgen(js_name = addResources)]
    pub fn add_resources(&mut self, resources: Vec<JsValue>) {
        use serde_wasm_bindgen::from_value;

        // Parse the array of [string, string[] | null] tuples
        let resources_data: Vec<(String, Option<Vec<String>>)> = from_value(JsValue::from(resources))
            .unwrap_or_else(|e| panic!("Failed to parse 'resources' array: {:?}", e));

        for (resource, parents) in resources_data {
            let parent_refs: Option<Vec<&str>> = parents.as_ref().map(|p| {
                p.iter().map(|s| s.as_str()).collect()
            });

            self.inner.add_resource(&resource, parent_refs.as_deref())
                .unwrap_or_else(|e| panic!("{}", e));
        }
    }

    /// Adds an "allow" rule
    ///
    /// # Arguments
    /// * `roles` - Array of role names (null means "all roles")
    /// * `resources` - Array of resource names (null means "all resources")
    /// * `privileges` - Array of privilege names (null means "all privileges")
    #[wasm_bindgen(js_name = allow)]
    pub fn allow(
        &mut self,
        roles: Option<Vec<String>>,
        resources: Option<Vec<String>>,
        privileges: Option<Vec<String>>,
    ) {
        let role_refs: Option<Vec<&str>> = roles.as_ref().map(|r| {
            r.iter().map(|s| s.as_str()).collect()
        });
        let resource_refs: Option<Vec<&str>> = resources.as_ref().map(|r| {
            r.iter().map(|s| s.as_str()).collect()
        });
        let privilege_refs: Option<Vec<&str>> = privileges.as_ref().map(|p| {
            p.iter().map(|s| s.as_str()).collect()
        });

        self.inner.allow(
            role_refs.as_deref(),
            resource_refs.as_deref(),
            privilege_refs.as_deref(),
        ).unwrap_or_else(|e| panic!("{}", e));
    }

    /// Adds a "deny" rule
    ///
    /// # Arguments
    /// * `roles` - Array of role names (null means "all roles")
    /// * `resources` - Array of resource names (null means "all resources")
    /// * `privileges` - Array of privilege names (null means "all privileges")
    #[wasm_bindgen(js_name = deny)]
    pub fn deny(
        &mut self,
        roles: Option<Vec<String>>,
        resources: Option<Vec<String>>,
        privileges: Option<Vec<String>>,
    ) {
        let role_refs: Option<Vec<&str>> = roles.as_ref().map(|r| {
            r.iter().map(|s| s.as_str()).collect()
        });
        let resource_refs: Option<Vec<&str>> = resources.as_ref().map(|r| {
            r.iter().map(|s| s.as_str()).collect()
        });
        let privilege_refs: Option<Vec<&str>> = privileges.as_ref().map(|p| {
            p.iter().map(|s| s.as_str()).collect()
        });

        self.inner.deny(
            role_refs.as_deref(),
            resource_refs.as_deref(),
            privilege_refs.as_deref(),
        ).unwrap_or_else(|e| panic!("{}", e));
    }

    /// Builds the final ACL
    #[wasm_bindgen(js_name = build)]
    pub fn build(&mut self) -> Result<JsAcl, JsValue> {
        let acl = self.inner.build()
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(JsAcl { inner: acl })
    }
}

/// Convenience function to create an ACL from JSON
#[wasm_bindgen(js_name = createAclFromJson)]
pub fn create_acl_from_json(json: &str) -> Result<JsAcl, JsValue> {
    JsAcl::from_json(json)
}

/// Convenience function to check a permission directly
#[wasm_bindgen(js_name = checkPermission)]
pub fn check_permission(
    acl_json: &str,
    role: Option<String>,
    resource: Option<String>,
    privilege: Option<String>,
) -> Result<bool, JsValue> {
    let acl = JsAcl::from_json(acl_json)?;
    Ok(acl.is_allowed(role, resource, privilege))
}

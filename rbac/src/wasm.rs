//! WebAssembly bindings for walrs_rbac
//!
//! This module provides JavaScript-compatible wrappers around the RBAC structure from
//! the walrs_rbac crate.

use crate::prelude::{String, Vec, format};
use crate::rbac::Rbac;
use crate::rbac_builder::RbacBuilder;
use crate::rbac_data::RbacData;
use crate::role::Role;
use wasm_bindgen::prelude::*;

/// JavaScript-compatible wrapper for Rbac
#[wasm_bindgen]
pub struct JsRbac {
  inner: Rbac,
}

#[wasm_bindgen]
/// JavaScript-compatible implementation of the Rbac type.
/// Wraps the Rust `Rbac` struct for use in JS/WASM environments.
impl JsRbac {
  /// Creates a new empty RBAC
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self { inner: Rbac::new() }
  }

  /// Creates an RBAC from a JSON string
  ///
  /// # Example JSON format
  /// ```json
  /// {
  ///   "roles": [
  ///     ["guest", ["read.public"], null],
  ///     ["admin", ["admin.panel"], ["guest"]]
  ///   ]
  /// }
  /// ```
  #[wasm_bindgen(js_name = fromJson)]
  pub fn from_json(json: &str) -> Result<JsRbac, JsValue> {
    let rbac_data: RbacData = serde_json::from_str(json)
      .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;

    let rbac = RbacBuilder::try_from(&rbac_data)
      .map_err(|e| JsValue::from_str(&format!("{}", e)))?
      .build()
      .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

    Ok(JsRbac { inner: rbac })
  }

  /// Checks if a role is granted a specific permission
  ///
  /// # Arguments
  /// * `role` - The role name (e.g., "admin", "user")
  /// * `permission` - The permission to check (e.g., "edit.article")
  ///
  /// # Returns
  /// `true` if the permission is granted (directly or via inheritance), `false` otherwise
  #[wasm_bindgen(js_name = isGranted)]
  pub fn is_granted(&self, role: &str, permission: &str) -> bool {
    self.inner.is_granted(role, permission)
  }

  /// Checks if the RBAC contains a specific role
  #[wasm_bindgen(js_name = hasRole)]
  pub fn has_role(&self, role: &str) -> bool {
    self.inner.has_role(role)
  }

  /// Returns the number of roles in the RBAC
  #[wasm_bindgen(js_name = roleCount)]
  pub fn role_count(&self) -> usize {
    self.inner.role_count()
  }
}

/// JavaScript-compatible wrapper for RbacBuilder
#[wasm_bindgen]
pub struct JsRbacBuilder {
  inner: RbacBuilder,
}

#[wasm_bindgen]
/// JavaScript-compatible implementation of the RbacBuilder type.
/// Wraps the Rust `RbacBuilder` struct for use in JS/WASM environments.
impl JsRbacBuilder {
  /// Creates a new RBAC builder
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self {
      inner: RbacBuilder::new(),
    }
  }

  /// Creates an RBAC builder from a JSON string
  #[wasm_bindgen(js_name = fromJson)]
  pub fn from_json(json: &str) -> Result<JsRbacBuilder, JsValue> {
    let rbac_data: RbacData = serde_json::from_str(json)
      .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;

    let builder =
      RbacBuilder::try_from(&rbac_data).map_err(|e| JsValue::from_str(&format!("{}", e)))?;

    Ok(JsRbacBuilder { inner: builder })
  }

  /// Adds a role with permissions and optional children
  ///
  /// # Arguments
  /// * `name` - The role name
  /// * `permissions` - Array of permission strings
  /// * `children` - Optional array of child role names
  #[wasm_bindgen(js_name = addRole)]
  pub fn add_role(
    mut self,
    name: &str,
    permissions: Vec<String>,
    children: Option<Vec<String>>,
  ) -> Result<JsRbacBuilder, JsValue> {
    let perm_refs: Vec<&str> = permissions.iter().map(|s| s.as_str()).collect();
    let child_refs: Option<Vec<&str>> = children
      .as_ref()
      .map(|c| c.iter().map(|s| s.as_str()).collect());

    self
      .inner
      .add_role(name, &perm_refs, child_refs.as_deref())
      .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

    Ok(self)
  }

  /// Adds a permission to a role
  ///
  /// # Arguments
  /// * `role_name` - The role name
  /// * `permission` - The permission to add
  #[wasm_bindgen(js_name = addPermission)]
  pub fn add_permission(
    mut self,
    role_name: &str,
    permission: &str,
  ) -> Result<JsRbacBuilder, JsValue> {
    self
      .inner
      .add_permission(role_name, permission)
      .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

    Ok(self)
  }

  /// Adds a child role to a parent role
  ///
  /// # Arguments
  /// * `parent_name` - The parent role name
  /// * `child_name` - The child role name
  #[wasm_bindgen(js_name = addChild)]
  pub fn add_child(
    mut self,
    parent_name: &str,
    child_name: &str,
  ) -> Result<JsRbacBuilder, JsValue> {
    self
      .inner
      .add_child(parent_name, child_name)
      .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

    Ok(self)
  }

  /// Builds the final RBAC
  #[wasm_bindgen(js_name = build)]
  pub fn build(self) -> Result<JsRbac, JsValue> {
    let rbac = self
      .inner
      .build()
      .map_err(|e| JsValue::from_str(&format!("{}", e)))?;

    Ok(JsRbac { inner: rbac })
  }
}

/// Convenience function to create an RBAC from JSON
#[wasm_bindgen(js_name = createRbacFromJson)]
pub fn create_rbac_from_json(json: &str) -> Result<JsRbac, JsValue> {
  JsRbac::from_json(json)
}

/// Convenience function to check a permission directly
#[wasm_bindgen(js_name = checkPermission)]
pub fn check_permission(rbac_json: &str, role: &str, permission: &str) -> Result<bool, JsValue> {
  let rbac = JsRbac::from_json(rbac_json)?;
  Ok(rbac.is_granted(role, permission))
}

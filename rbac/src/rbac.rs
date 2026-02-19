#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(feature = "std")]
use std::collections::HashMap;

use crate::error::{RbacError, Result};
use crate::prelude::{String, ToString};
use crate::role::Role;
use serde_derive::{Deserialize, Serialize};

/// Role-Based Access Control (RBAC) permissions management.
///
/// Provides a structure for managing roles and permissions, where roles
/// can have child roles and permissions are inherited through the role
/// hierarchy. Inspired by the
/// [laminas-permissions-rbac](https://github.com/laminas/laminas-permissions-rbac)
/// PHP library.
///
/// In RBAC, permissions are attached to roles, and users are assigned roles.
/// Parent roles inherit permissions from their children, allowing for
/// hierarchical permission structures.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::{Rbac, RbacBuilder};
///
/// let rbac = RbacBuilder::new()
///   .add_role("editor", &["edit.article", "publish.article"], None)?
///   .add_role("admin", &["admin.panel"], Some(&["editor"]))?
///   .build()?;
///
/// // Admin inherits editor permissions
/// assert!(rbac.is_granted("admin", "edit.article"));
/// assert!(rbac.is_granted("admin", "admin.panel"));
///
/// // Editor has its own permissions
/// assert!(rbac.is_granted("editor", "edit.article"));
/// assert!(!rbac.is_granted("editor", "admin.panel"));
/// # Ok::<(), walrs_rbac::RbacError>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rbac {
  pub(crate) roles: HashMap<String, Role>,
}

impl Rbac {
  /// Creates a new, empty RBAC instance.
  pub fn new() -> Self {
    Rbac {
      roles: HashMap::new(),
    }
  }

  /// Creates an RBAC instance from a pre-built roles map.
  pub(crate) fn from_roles(roles: HashMap<String, Role>) -> Self {
    Rbac { roles }
  }

  /// Adds a role to the RBAC. Returns `&mut Self` for chaining.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::{Rbac, Role};
  ///
  /// let mut rbac = Rbac::new();
  /// let mut role = Role::new("admin");
  /// role.add_permission("manage.users");
  /// rbac.add_role(role);
  ///
  /// assert!(rbac.has_role("admin"));
  /// ```
  pub fn add_role(&mut self, role: Role) -> &mut Self {
    self.roles.insert(role.name().to_string(), role);
    self
  }

  /// Checks if a role exists in the RBAC.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::{Rbac, Role};
  ///
  /// let mut rbac = Rbac::new();
  /// rbac.add_role(Role::new("admin"));
  ///
  /// assert!(rbac.has_role("admin"));
  /// assert!(!rbac.has_role("guest"));
  /// ```
  pub fn has_role(&self, role_name: &str) -> bool {
    self.roles.contains_key(role_name)
  }

  /// Returns a reference to a role by name, if it exists.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::{Rbac, Role};
  ///
  /// let mut rbac = Rbac::new();
  /// let mut role = Role::new("editor");
  /// role.add_permission("edit.article");
  /// rbac.add_role(role);
  ///
  /// let editor = rbac.get_role("editor").unwrap();
  /// assert!(editor.has_permission("edit.article"));
  /// ```
  pub fn get_role(&self, role_name: &str) -> Option<&Role> {
    self.roles.get(role_name)
  }

  /// Returns the number of roles in the RBAC.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::{Rbac, Role};
  ///
  /// let mut rbac = Rbac::new();
  /// rbac.add_role(Role::new("admin"))
  ///     .add_role(Role::new("user"));
  ///
  /// assert_eq!(rbac.role_count(), 2);
  /// ```
  pub fn role_count(&self) -> usize {
    self.roles.len()
  }

  /// Checks if a given role is granted a specific permission.
  ///
  /// This checks both the role's direct permissions and any permissions
  /// inherited from child roles (recursively).
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::{Rbac, RbacBuilder};
  ///
  /// let rbac = RbacBuilder::new()
  ///   .add_role("viewer", &["read.article"], None)?
  ///   .add_role("editor", &["edit.article"], Some(&["viewer"]))?
  ///   .add_role("admin", &["admin.panel"], Some(&["editor"]))?
  ///   .build()?;
  ///
  /// assert!(rbac.is_granted("admin", "read.article")); // inherited from viewer via editor
  /// assert!(rbac.is_granted("admin", "edit.article")); // inherited from editor
  /// assert!(rbac.is_granted("admin", "admin.panel"));  // direct permission
  /// assert!(!rbac.is_granted("viewer", "edit.article")); // viewer can't edit
  /// # Ok::<(), walrs_rbac::RbacError>(())
  /// ```
  pub fn is_granted(&self, role_name: &str, permission: &str) -> bool {
    self
      .roles
      .get(role_name)
      .is_some_and(|role| role.has_permission_recursive(permission))
  }

  /// Checks if a role is granted a permission, returning an error if the role
  /// does not exist.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::{Rbac, Role};
  ///
  /// let mut rbac = Rbac::new();
  /// let mut role = Role::new("admin");
  /// role.add_permission("manage");
  /// rbac.add_role(role);
  ///
  /// assert!(rbac.is_granted_safe("admin", "manage").unwrap());
  /// assert!(rbac.is_granted_safe("nonexistent", "manage").is_err());
  /// ```
  pub fn is_granted_safe(&self, role_name: &str, permission: &str) -> Result<bool> {
    self
      .roles
      .get(role_name)
      .map(|role| role.has_permission_recursive(permission))
      .ok_or_else(|| RbacError::RoleNotFound(role_name.to_string()))
  }

  /// Returns an iterator over all role names in the RBAC.
  pub fn role_names(&self) -> impl Iterator<Item = &String> {
    self.roles.keys()
  }
}

impl Default for Rbac {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_rbac() {
    let rbac = Rbac::new();
    assert_eq!(rbac.role_count(), 0);
  }

  #[test]
  fn test_default_rbac() {
    let rbac = Rbac::default();
    assert_eq!(rbac.role_count(), 0);
  }

  #[test]
  fn test_add_role() {
    let mut rbac = Rbac::new();
    rbac.add_role(Role::new("admin"));
    assert!(rbac.has_role("admin"));
    assert_eq!(rbac.role_count(), 1);
  }

  #[test]
  fn test_add_role_chaining() {
    let mut rbac = Rbac::new();
    rbac
      .add_role(Role::new("admin"))
      .add_role(Role::new("user"))
      .add_role(Role::new("guest"));
    assert_eq!(rbac.role_count(), 3);
  }

  #[test]
  fn test_has_role() {
    let mut rbac = Rbac::new();
    rbac.add_role(Role::new("admin"));
    assert!(rbac.has_role("admin"));
    assert!(!rbac.has_role("guest"));
  }

  #[test]
  fn test_get_role() {
    let mut rbac = Rbac::new();
    let mut role = Role::new("editor");
    role.add_permission("edit.article");
    rbac.add_role(role);

    let editor = rbac.get_role("editor").unwrap();
    assert_eq!(editor.name(), "editor");
    assert!(editor.has_permission("edit.article"));
    assert!(rbac.get_role("nonexistent").is_none());
  }

  #[test]
  fn test_is_granted_direct() {
    let mut rbac = Rbac::new();
    let mut role = Role::new("user");
    role.add_permission("read.article");
    rbac.add_role(role);

    assert!(rbac.is_granted("user", "read.article"));
    assert!(!rbac.is_granted("user", "delete.article"));
  }

  #[test]
  fn test_is_granted_inherited() {
    let mut child = Role::new("editor");
    child.add_permission("edit.article");

    let mut parent = Role::new("admin");
    parent.add_permission("admin.panel");
    parent.add_child(child);

    let mut rbac = Rbac::new();
    rbac.add_role(parent);

    assert!(rbac.is_granted("admin", "admin.panel"));
    assert!(rbac.is_granted("admin", "edit.article"));
    assert!(!rbac.is_granted("admin", "nonexistent"));
  }

  #[test]
  fn test_is_granted_nonexistent_role() {
    let rbac = Rbac::new();
    assert!(!rbac.is_granted("nonexistent", "anything"));
  }

  #[test]
  fn test_is_granted_safe() {
    let mut rbac = Rbac::new();
    let mut role = Role::new("admin");
    role.add_permission("manage");
    rbac.add_role(role);

    assert!(rbac.is_granted_safe("admin", "manage").unwrap());
    assert!(!rbac.is_granted_safe("admin", "other").unwrap());
    assert!(rbac.is_granted_safe("nonexistent", "manage").is_err());
  }

  #[test]
  fn test_is_granted_safe_role_not_found_error() {
    let rbac = Rbac::new();
    let result = rbac.is_granted_safe("nonexistent", "manage");
    assert!(matches!(result, Err(RbacError::RoleNotFound(name)) if name == "nonexistent"));
  }

  #[test]
  fn test_is_granted_deep_inheritance() {
    let mut level3 = Role::new("reader");
    level3.add_permission("read");

    let mut level2 = Role::new("writer");
    level2.add_permission("write");
    level2.add_child(level3);

    let mut level1 = Role::new("admin");
    level1.add_permission("admin");
    level1.add_child(level2);

    let mut rbac = Rbac::new();
    rbac.add_role(level1);

    assert!(rbac.is_granted("admin", "admin"));
    assert!(rbac.is_granted("admin", "write"));
    assert!(rbac.is_granted("admin", "read"));
  }

  #[test]
  fn test_role_names() {
    let mut rbac = Rbac::new();
    rbac
      .add_role(Role::new("admin"))
      .add_role(Role::new("user"));

    let names: Vec<&String> = rbac.role_names().collect();
    assert_eq!(names.len(), 2);
  }

  #[test]
  fn test_replace_existing_role() {
    let mut rbac = Rbac::new();
    let mut role1 = Role::new("admin");
    role1.add_permission("perm1");
    rbac.add_role(role1);

    let mut role2 = Role::new("admin");
    role2.add_permission("perm2");
    rbac.add_role(role2);

    assert_eq!(rbac.role_count(), 1);
    assert!(!rbac.is_granted("admin", "perm1"));
    assert!(rbac.is_granted("admin", "perm2"));
  }
}

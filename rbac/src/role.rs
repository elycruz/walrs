#[cfg(feature = "std")]
use std::collections::HashSet;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeSet as HashSet;

use crate::prelude::{String, Vec, ToString};
use serde_derive::{Deserialize, Serialize};

/// A role in the RBAC system.
///
/// Each role has a name, a set of permissions, and optional child roles.
/// Child roles inherit permissions from their parent role when checking
/// if a permission is granted.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::Role;
///
/// let mut role = Role::new("admin");
/// role.add_permission("edit.article")
///     .add_permission("delete.article");
///
/// assert!(role.has_permission("edit.article"));
/// assert!(!role.has_permission("publish.article"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Role {
  name: String,
  permissions: HashSet<String>,
  children: Vec<Role>,
}

impl Role {
  /// Creates a new role with the given name.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::Role;
  ///
  /// let role = Role::new("user");
  /// assert_eq!(role.name(), "user");
  /// assert_eq!(role.permission_count(), 0);
  /// assert_eq!(role.child_count(), 0);
  /// ```
  pub fn new(name: &str) -> Self {
    Role {
      name: name.to_string(),
      permissions: HashSet::new(),
      children: Vec::new(),
    }
  }

  /// Returns the name of this role.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Adds a permission to this role. Returns `&mut Self` for chaining.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::Role;
  ///
  /// let mut role = Role::new("editor");
  /// role.add_permission("edit.article")
  ///     .add_permission("publish.article");
  ///
  /// assert!(role.has_permission("edit.article"));
  /// assert!(role.has_permission("publish.article"));
  /// ```
  pub fn add_permission(&mut self, permission: &str) -> &mut Self {
    self.permissions.insert(permission.to_string());
    self
  }

  /// Adds multiple permissions to this role. Returns `&mut Self` for chaining.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::Role;
  ///
  /// let mut role = Role::new("editor");
  /// role.add_permissions(&["edit.article", "publish.article", "delete.article"]);
  ///
  /// assert_eq!(role.permission_count(), 3);
  /// ```
  pub fn add_permissions(&mut self, permissions: &[&str]) -> &mut Self {
    for p in permissions {
      self.permissions.insert(p.to_string());
    }
    self
  }

  /// Checks if this role has a specific permission (does not check children).
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::Role;
  ///
  /// let mut role = Role::new("user");
  /// role.add_permission("read.article");
  ///
  /// assert!(role.has_permission("read.article"));
  /// assert!(!role.has_permission("delete.article"));
  /// ```
  pub fn has_permission(&self, permission: &str) -> bool {
    self.permissions.contains(permission)
  }

  /// Checks if this role or any of its children (recursively) has the given permission.
  ///
  /// In the Laminas RBAC model, parent roles inherit permissions from their
  /// children. This method traverses the role hierarchy.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::Role;
  ///
  /// let mut child = Role::new("editor");
  /// child.add_permission("edit.article");
  ///
  /// let mut parent = Role::new("admin");
  /// parent.add_child(child);
  ///
  /// // Admin inherits "edit.article" from editor child
  /// assert!(parent.has_permission_recursive("edit.article"));
  /// assert!(!parent.has_permission_recursive("delete.article"));
  /// ```
  pub fn has_permission_recursive(&self, permission: &str) -> bool {
    if self.permissions.contains(permission) {
      return true;
    }
    self.children.iter().any(|child| child.has_permission_recursive(permission))
  }

  /// Adds a child role. Returns `&mut Self` for chaining.
  ///
  /// Child role permissions are inherited by this (parent) role when
  /// using `has_permission_recursive`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::Role;
  ///
  /// let mut child = Role::new("editor");
  /// child.add_permission("edit.article");
  ///
  /// let mut parent = Role::new("admin");
  /// parent.add_child(child);
  ///
  /// assert_eq!(parent.child_count(), 1);
  /// ```
  pub fn add_child(&mut self, child: Role) -> &mut Self {
    self.children.push(child);
    self
  }

  /// Returns the number of direct permissions on this role.
  pub fn permission_count(&self) -> usize {
    self.permissions.len()
  }

  /// Returns the number of direct child roles.
  pub fn child_count(&self) -> usize {
    self.children.len()
  }

  /// Returns a reference to the direct children of this role.
  pub fn children(&self) -> &[Role] {
    &self.children
  }

  /// Returns an iterator over the permissions of this role.
  pub fn permissions(&self) -> impl Iterator<Item = &String> {
    self.permissions.iter()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_role() {
    let role = Role::new("guest");
    assert_eq!(role.name(), "guest");
    assert_eq!(role.permission_count(), 0);
    assert_eq!(role.child_count(), 0);
  }

  #[test]
  fn test_add_permission() {
    let mut role = Role::new("user");
    role.add_permission("read");
    assert!(role.has_permission("read"));
    assert!(!role.has_permission("write"));
  }

  #[test]
  fn test_add_permissions() {
    let mut role = Role::new("user");
    role.add_permissions(&["read", "write", "delete"]);
    assert_eq!(role.permission_count(), 3);
    assert!(role.has_permission("read"));
    assert!(role.has_permission("write"));
    assert!(role.has_permission("delete"));
  }

  #[test]
  fn test_add_permission_chaining() {
    let mut role = Role::new("editor");
    role.add_permission("edit")
      .add_permission("publish")
      .add_permission("delete");
    assert_eq!(role.permission_count(), 3);
  }

  #[test]
  fn test_duplicate_permission() {
    let mut role = Role::new("user");
    role.add_permission("read").add_permission("read");
    assert_eq!(role.permission_count(), 1);
  }

  #[test]
  fn test_add_child() {
    let child = Role::new("editor");
    let mut parent = Role::new("admin");
    parent.add_child(child);
    assert_eq!(parent.child_count(), 1);
    assert_eq!(parent.children()[0].name(), "editor");
  }

  #[test]
  fn test_add_child_chaining() {
    let mut parent = Role::new("admin");
    parent.add_child(Role::new("editor"))
      .add_child(Role::new("moderator"));
    assert_eq!(parent.child_count(), 2);
  }

  #[test]
  fn test_has_permission_recursive() {
    let mut grandchild = Role::new("viewer");
    grandchild.add_permission("read.article");

    let mut child = Role::new("editor");
    child.add_permission("edit.article");
    child.add_child(grandchild);

    let mut parent = Role::new("admin");
    parent.add_permission("admin.panel");
    parent.add_child(child);

    // Direct permission
    assert!(parent.has_permission_recursive("admin.panel"));
    // Inherited from child
    assert!(parent.has_permission_recursive("edit.article"));
    // Inherited from grandchild
    assert!(parent.has_permission_recursive("read.article"));
    // Non-existent
    assert!(!parent.has_permission_recursive("delete.user"));
  }

  #[test]
  fn test_has_permission_recursive_no_children() {
    let mut role = Role::new("guest");
    role.add_permission("read");
    assert!(role.has_permission_recursive("read"));
    assert!(!role.has_permission_recursive("write"));
  }

  #[test]
  fn test_permissions_iterator() {
    let mut role = Role::new("user");
    role.add_permissions(&["a", "b", "c"]);
    let perms: Vec<&String> = role.permissions().collect();
    assert_eq!(perms.len(), 3);
  }

  #[test]
  fn test_role_clone() {
    let mut original = Role::new("admin");
    original.add_permission("all");
    let cloned = original.clone();
    assert_eq!(original, cloned);
  }

  #[test]
  fn test_role_equality() {
    let mut r1 = Role::new("user");
    r1.add_permission("read");
    let mut r2 = Role::new("user");
    r2.add_permission("read");
    assert_eq!(r1, r2);

    let r3 = Role::new("admin");
    assert_ne!(r1, r3);
  }
}

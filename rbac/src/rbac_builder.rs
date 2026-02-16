#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

use core::convert::TryFrom;

#[cfg(feature = "std")]
use std::fs::File;

use crate::prelude::{String, Vec, ToString, format};
use crate::rbac::Rbac;
use crate::role::Role;
use crate::rbac_data::RbacData;
use crate::error::{RbacError, Result};

/// Builder for constructing `Rbac` instances with a fluent interface.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::RbacBuilder;
///
/// let rbac = RbacBuilder::new()
///   .add_role("guest", &["read.public"], None)?
///   .add_role("user", &["write.post"], Some(&["guest"]))?
///   .add_role("admin", &["admin.panel"], Some(&["user"]))?
///   .build()?;
///
/// assert!(rbac.is_granted("admin", "read.public")); // inherited via user <- guest
/// assert!(rbac.is_granted("admin", "write.post"));  // inherited from user
/// assert!(rbac.is_granted("admin", "admin.panel")); // direct
/// assert!(!rbac.is_granted("guest", "admin.panel"));
/// # Ok::<(), walrs_rbac::RbacError>(())
/// ```
#[derive(Debug)]
pub struct RbacBuilder {
  roles: HashMap<String, (Vec<String>, Vec<String>)>, // name -> (permissions, children_names)
}

impl RbacBuilder {
  /// Creates a new `RbacBuilder` instance.
  pub fn new() -> Self {
    RbacBuilder {
      roles: HashMap::new(),
    }
  }

  /// Adds a role with the given permissions and optional child role names.
  ///
  /// In the Laminas RBAC model, child roles' permissions are inherited
  /// by the parent. So "admin" having children ["editor"] means admin
  /// inherits editor's permissions.
  ///
  /// # Arguments
  ///
  /// * `name` - The role name.
  /// * `permissions` - Slice of permission strings to assign to this role.
  /// * `children` - Optional slice of child role names whose permissions this role inherits.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacBuilder;
  ///
  /// let rbac = RbacBuilder::new()
  ///   .add_role("viewer", &["read.article"], None)?
  ///   .add_role("editor", &["edit.article"], Some(&["viewer"]))?
  ///   .build()?;
  ///
  /// assert!(rbac.is_granted("editor", "edit.article")); // direct
  /// assert!(rbac.is_granted("editor", "read.article")); // inherited from viewer child
  /// # Ok::<(), walrs_rbac::RbacError>(())
  /// ```
  pub fn add_role(
    &mut self,
    name: &str,
    permissions: &[&str],
    children: Option<&[&str]>,
  ) -> Result<&mut Self> {
    let perms: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    let child_names: Vec<String> = children
      .map(|c| c.iter().map(|n| n.to_string()).collect())
      .unwrap_or_default();

    self.roles.insert(name.to_string(), (perms, child_names));
    Ok(self)
  }

  /// Adds multiple roles at once.
  ///
  /// Each tuple is `(name, permissions, optional_children)`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacBuilder;
  ///
  /// let rbac = RbacBuilder::new()
  ///   .add_roles(&[
  ///     ("guest", &["read.public"], None),
  ///     ("user", &["write.post"], Some(&["guest"])),
  ///     ("admin", &["admin.panel"], Some(&["user"])),
  ///   ])?
  ///   .build()?;
  ///
  /// assert!(rbac.is_granted("admin", "read.public"));
  /// # Ok::<(), walrs_rbac::RbacError>(())
  /// ```
  #[allow(clippy::type_complexity)]
  pub fn add_roles(
    &mut self,
    roles: &[(&str, &[&str], Option<&[&str]>)],
  ) -> Result<&mut Self> {
    for &(name, permissions, children) in roles {
      self.add_role(name, permissions, children)?;
    }
    Ok(self)
  }

  /// Adds a permission to an existing role.
  ///
  /// If the role does not yet exist, it will be created.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacBuilder;
  ///
  /// let rbac = RbacBuilder::new()
  ///   .add_role("user", &["read"], None)?
  ///   .add_permission("user", "write")?
  ///   .build()?;
  ///
  /// assert!(rbac.is_granted("user", "read"));
  /// assert!(rbac.is_granted("user", "write"));
  /// # Ok::<(), walrs_rbac::RbacError>(())
  /// ```
  pub fn add_permission(&mut self, role_name: &str, permission: &str) -> Result<&mut Self> {
    let entry = self.roles
      .entry(role_name.to_string())
      .or_insert_with(|| (Vec::new(), Vec::new()));
    entry.0.push(permission.to_string());
    Ok(self)
  }

  /// Adds a child role to a parent role.
  ///
  /// The parent role inherits permissions from the child.
  /// If the parent role does not yet exist, it will be created.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacBuilder;
  ///
  /// let rbac = RbacBuilder::new()
  ///   .add_role("editor", &["edit.article"], None)?
  ///   .add_role("admin", &[], None)?
  ///   .add_child("admin", "editor")?
  ///   .build()?;
  ///
  /// assert!(rbac.is_granted("admin", "edit.article"));
  /// # Ok::<(), walrs_rbac::RbacError>(())
  /// ```
  pub fn add_child(&mut self, parent_name: &str, child_name: &str) -> Result<&mut Self> {
    let entry = self.roles
      .entry(parent_name.to_string())
      .or_insert_with(|| (Vec::new(), Vec::new()));
    entry.1.push(child_name.to_string());
    Ok(self)
  }

  /// Builds and returns the final `Rbac` instance.
  ///
  /// This resolves the role hierarchy, embedding child roles into their
  /// parents. Returns an error if a referenced child role does not exist
  /// or if a cycle is detected.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_rbac::RbacBuilder;
  ///
  /// let rbac = RbacBuilder::new()
  ///   .add_role("guest", &["read"], None)?
  ///   .build()?;
  ///
  /// assert!(rbac.is_granted("guest", "read"));
  /// # Ok::<(), walrs_rbac::RbacError>(())
  /// ```
  pub fn build(&self) -> Result<Rbac> {
    // Validate all child references exist
    for (name, (_, children)) in &self.roles {
      for child_name in children {
        if !self.roles.contains_key(child_name) {
          return Err(RbacError::InvalidConfiguration(
            format!("Role '{}' references child '{}' which does not exist", name, child_name)
          ));
        }
      }
    }

    // Check for cycles via DFS
    self.check_for_cycles()?;

    // Build roles with resolved children
    let mut built_roles: HashMap<String, Role> = HashMap::new();

    // Use topological-like approach: build leaf roles first
    let mut resolved: HashMap<String, Role> = HashMap::new();
    let mut visit_stack: Vec<String> = Vec::new();

    for name in self.roles.keys() {
      self.resolve_role(name, &mut resolved, &mut visit_stack)?;
    }

    for (name, role) in resolved {
      built_roles.insert(name, role);
    }

    Ok(Rbac::from_roles(built_roles))
  }

  /// Recursively resolves a role and its children.
  fn resolve_role(
    &self,
    name: &str,
    resolved: &mut HashMap<String, Role>,
    _visit_stack: &mut Vec<String>,
  ) -> Result<Role> {
    if let Some(role) = resolved.get(name) {
      return Ok(role.clone());
    }

    let (permissions, children_names) = self.roles.get(name)
      .ok_or_else(|| RbacError::RoleNotFound(name.to_string()))?;

    let mut role = Role::new(name);
    for perm in permissions {
      role.add_permission(perm);
    }

    for child_name in children_names {
      let child = self.resolve_role(child_name, resolved, &mut Vec::new())?;
      role.add_child(child);
    }

    resolved.insert(name.to_string(), role.clone());
    Ok(role)
  }

  /// Checks for cycles in the role hierarchy using DFS.
  fn check_for_cycles(&self) -> Result<()> {
    #[cfg(feature = "std")]
    use std::collections::HashSet;
    #[cfg(not(feature = "std"))]
    use alloc::collections::BTreeSet as HashSet;

    fn dfs(
      name: &str,
      roles: &HashMap<String, (Vec<String>, Vec<String>)>,
      visited: &mut HashSet<String>,
      path: &mut Vec<String>,
    ) -> Result<()> {
      if path.contains(&name.to_string()) {
        return Err(RbacError::CycleDetected(
          format!("Cycle detected at role '{}'", name)
        ));
      }
      if visited.contains(name) {
        return Ok(());
      }

      path.push(name.to_string());

      if let Some((_, children)) = roles.get(name) {
        for child in children {
          dfs(child, roles, visited, path)?;
        }
      }

      path.pop();
      visited.insert(name.to_string());
      Ok(())
    }

    let mut visited = HashSet::new();
    let mut path = Vec::new();

    for name in self.roles.keys() {
      dfs(name, &self.roles, &mut visited, &mut path)?;
    }

    Ok(())
  }
}

impl Default for RbacBuilder {
  fn default() -> Self {
    Self::new()
  }
}

/// Converts an `RbacData` reference into an `RbacBuilder`.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::{RbacBuilder, RbacData};
/// use std::convert::TryFrom;
///
/// let data = RbacData {
///   roles: vec![
///     ("guest".to_string(), vec!["read.public".to_string()], None),
///     ("admin".to_string(), vec!["admin.panel".to_string()], Some(vec!["guest".to_string()])),
///   ],
/// };
///
/// let rbac = RbacBuilder::try_from(&data)?.build()?;
/// assert!(rbac.is_granted("admin", "read.public"));
/// assert!(!rbac.is_granted("guest", "admin.panel"));
/// # Ok::<(), walrs_rbac::RbacError>(())
/// ```
impl<'a> TryFrom<&'a RbacData> for RbacBuilder {
  type Error = RbacError;

  fn try_from(data: &'a RbacData) -> Result<Self> {
    let mut builder = RbacBuilder::new();

    for (name, permissions, children) in &data.roles {
      let perm_refs: Vec<&str> = permissions.iter().map(|s| s.as_str()).collect();
      let child_refs: Option<Vec<&str>> = children
        .as_ref()
        .map(|c| c.iter().map(|s| s.as_str()).collect());
      builder.add_role(name, &perm_refs, child_refs.as_deref())?;
    }

    Ok(builder)
  }
}

/// Converts an `RbacData` into an `RbacBuilder`.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::{RbacBuilder, RbacData};
/// use std::convert::TryFrom;
///
/// let data = RbacData {
///   roles: vec![
///     ("user".to_string(), vec!["read".to_string()], None),
///   ],
/// };
///
/// let rbac = RbacBuilder::try_from(data)?.build()?;
/// assert!(rbac.is_granted("user", "read"));
/// # Ok::<(), walrs_rbac::RbacError>(())
/// ```
impl TryFrom<RbacData> for RbacBuilder {
  type Error = RbacError;

  fn try_from(data: RbacData) -> Result<Self> {
    RbacBuilder::try_from(&data)
  }
}

/// Converts an `Rbac` instance back into an `RbacBuilder`.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::RbacBuilder;
/// use std::convert::TryFrom;
///
/// let rbac = RbacBuilder::new()
///   .add_role("user", &["read"], None)?
///   .build()?;
///
/// let modified = RbacBuilder::try_from(&rbac)?
///   .add_role("admin", &["admin.panel"], Some(&["user"]))?
///   .build()?;
///
/// assert!(modified.is_granted("admin", "read"));
/// # Ok::<(), walrs_rbac::RbacError>(())
/// ```
impl TryFrom<&Rbac> for RbacBuilder {
  type Error = RbacError;

  fn try_from(rbac: &Rbac) -> Result<Self> {
    let mut builder = RbacBuilder::new();

    fn extract_role(
      role: &Role,
      builder: &mut RbacBuilder,
    ) -> Result<()> {
      let perms: Vec<&str> = role.permissions().map(|s| s.as_str()).collect();
      let children_names: Vec<&str> = role.children().iter().map(|c| c.name()).collect();
      let children: Option<&[&str]> = if children_names.is_empty() {
        None
      } else {
        Some(&children_names)
      };
      builder.add_role(role.name(), &perms, children)?;

      for child in role.children() {
        extract_role(child, builder)?;
      }

      Ok(())
    }

    for role in rbac.roles.values() {
      extract_role(role, &mut builder)?;
    }

    Ok(builder)
  }
}

/// Converts a mutable file reference into an `RbacBuilder` by reading JSON.
///
/// # Example
///
/// ```rust
/// use walrs_rbac::RbacBuilder;
/// use std::convert::TryFrom;
/// use std::fs::File;
///
/// let file_path = "./test-fixtures/example-rbac.json";
/// let mut f = File::open(&file_path)?;
/// let rbac = RbacBuilder::try_from(&mut f)?.build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[cfg(feature = "std")]
impl TryFrom<&mut File> for RbacBuilder {
  type Error = RbacError;

  fn try_from(file: &mut File) -> Result<Self> {
    let data = RbacData::try_from(file)?;
    RbacBuilder::try_from(&data)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_builder() {
    let builder = RbacBuilder::new();
    let rbac = builder.build().unwrap();
    assert_eq!(rbac.role_count(), 0);
  }

  #[test]
  fn test_default_builder() {
    let builder = RbacBuilder::default();
    let rbac = builder.build().unwrap();
    assert_eq!(rbac.role_count(), 0);
  }

  #[test]
  fn test_add_role() {
    let rbac = RbacBuilder::new()
      .add_role("admin", &["manage.users"], None).unwrap()
      .build().unwrap();

    assert!(rbac.has_role("admin"));
    assert!(rbac.is_granted("admin", "manage.users"));
  }

  #[test]
  fn test_add_role_with_children() {
    let rbac = RbacBuilder::new()
      .add_role("guest", &["read"], None).unwrap()
      .add_role("user", &["write"], Some(&["guest"])).unwrap()
      .build().unwrap();

    assert!(rbac.is_granted("user", "write"));
    assert!(rbac.is_granted("user", "read")); // inherited
    assert!(!rbac.is_granted("guest", "write"));
  }

  #[test]
  fn test_add_roles() {
    let rbac = RbacBuilder::new()
      .add_roles(&[
        ("guest", &["read"], None),
        ("user", &["write"], Some(&["guest"])),
        ("admin", &["admin"], Some(&["user"])),
      ]).unwrap()
      .build().unwrap();

    assert!(rbac.is_granted("admin", "read"));
    assert!(rbac.is_granted("admin", "write"));
    assert!(rbac.is_granted("admin", "admin"));
    assert!(!rbac.is_granted("guest", "write"));
  }

  #[test]
  fn test_add_permission() {
    let rbac = RbacBuilder::new()
      .add_role("user", &["read"], None).unwrap()
      .add_permission("user", "write").unwrap()
      .build().unwrap();

    assert!(rbac.is_granted("user", "read"));
    assert!(rbac.is_granted("user", "write"));
  }

  #[test]
  fn test_add_permission_creates_role() {
    let rbac = RbacBuilder::new()
      .add_permission("user", "read").unwrap()
      .build().unwrap();

    assert!(rbac.has_role("user"));
    assert!(rbac.is_granted("user", "read"));
  }

  #[test]
  fn test_add_child() {
    let rbac = RbacBuilder::new()
      .add_role("editor", &["edit"], None).unwrap()
      .add_role("admin", &["admin"], None).unwrap()
      .add_child("admin", "editor").unwrap()
      .build().unwrap();

    assert!(rbac.is_granted("admin", "admin"));
    assert!(rbac.is_granted("admin", "edit"));
  }

  #[test]
  fn test_add_child_creates_parent() {
    let rbac = RbacBuilder::new()
      .add_role("editor", &["edit"], None).unwrap()
      .add_child("admin", "editor").unwrap()
      .build().unwrap();

    assert!(rbac.has_role("admin"));
    assert!(rbac.is_granted("admin", "edit"));
  }

  #[test]
  fn test_missing_child_role_error() {
    let result = RbacBuilder::new()
      .add_role("admin", &["manage"], Some(&["nonexistent"])).unwrap()
      .build();

    assert!(result.is_err());
    match result {
      Err(RbacError::InvalidConfiguration(msg)) => {
        assert!(msg.contains("nonexistent"));
      }
      _ => panic!("Expected InvalidConfiguration error"),
    }
  }

  #[test]
  fn test_cycle_detection() {
    let result = RbacBuilder::new()
      .add_role("a", &[], Some(&["b"])).unwrap()
      .add_role("b", &[], Some(&["a"])).unwrap()
      .build();

    assert!(result.is_err());
  }

  #[test]
  fn test_self_cycle_detection() {
    let result = RbacBuilder::new()
      .add_role("a", &[], Some(&["a"])).unwrap()
      .build();

    assert!(result.is_err());
  }

  #[test]
  fn test_three_node_cycle_detection() {
    let result = RbacBuilder::new()
      .add_role("a", &[], Some(&["b"])).unwrap()
      .add_role("b", &[], Some(&["c"])).unwrap()
      .add_role("c", &[], Some(&["a"])).unwrap()
      .build();

    assert!(matches!(result, Err(RbacError::CycleDetected(_))));
  }

  #[test]
  fn test_deep_hierarchy() {
    let rbac = RbacBuilder::new()
      .add_role("level4", &["perm4"], None).unwrap()
      .add_role("level3", &["perm3"], Some(&["level4"])).unwrap()
      .add_role("level2", &["perm2"], Some(&["level3"])).unwrap()
      .add_role("level1", &["perm1"], Some(&["level2"])).unwrap()
      .build().unwrap();

    assert!(rbac.is_granted("level1", "perm1"));
    assert!(rbac.is_granted("level1", "perm2"));
    assert!(rbac.is_granted("level1", "perm3"));
    assert!(rbac.is_granted("level1", "perm4"));
    assert!(!rbac.is_granted("level4", "perm1"));
  }

  #[test]
  fn test_multiple_children() {
    let rbac = RbacBuilder::new()
      .add_role("reader", &["read"], None).unwrap()
      .add_role("writer", &["write"], None).unwrap()
      .add_role("admin", &["admin"], Some(&["reader", "writer"])).unwrap()
      .build().unwrap();

    assert!(rbac.is_granted("admin", "read"));
    assert!(rbac.is_granted("admin", "write"));
    assert!(rbac.is_granted("admin", "admin"));
  }

  #[test]
  fn test_diamond_hierarchy() {
    let rbac = RbacBuilder::new()
      .add_role("base", &["base.perm"], None).unwrap()
      .add_role("left", &["left.perm"], Some(&["base"])).unwrap()
      .add_role("right", &["right.perm"], Some(&["base"])).unwrap()
      .add_role("top", &["top.perm"], Some(&["left", "right"])).unwrap()
      .build().unwrap();

    assert!(rbac.is_granted("top", "top.perm"));
    assert!(rbac.is_granted("top", "left.perm"));
    assert!(rbac.is_granted("top", "right.perm"));
    assert!(rbac.is_granted("top", "base.perm"));
  }

  #[test]
  fn test_empty_permissions() {
    let rbac = RbacBuilder::new()
      .add_role("empty", &[], None).unwrap()
      .build().unwrap();

    assert!(rbac.has_role("empty"));
    assert!(!rbac.is_granted("empty", "anything"));
  }

  #[test]
  fn test_try_from_rbac_ref() {
    let original = RbacBuilder::new()
      .add_role("user", &["read"], None).unwrap()
      .build().unwrap();

    let modified = RbacBuilder::try_from(&original).unwrap()
      .add_role("admin", &["admin"], Some(&["user"])).unwrap()
      .build().unwrap();

    assert!(modified.is_granted("admin", "read"));
    assert!(modified.is_granted("admin", "admin"));
    // Original is still available
    assert!(original.is_granted("user", "read"));
  }

  #[test]
  fn test_try_from_rbac_data_ref() {
    let data = RbacData {
      roles: vec![
        ("guest".to_string(), vec!["read".to_string()], None),
        ("admin".to_string(), vec!["admin".to_string()], Some(vec!["guest".to_string()])),
      ],
    };

    let rbac = RbacBuilder::try_from(&data).unwrap().build().unwrap();
    assert!(rbac.is_granted("admin", "read"));
    assert!(rbac.is_granted("admin", "admin"));
  }

  #[test]
  fn test_try_from_rbac_data() {
    let data = RbacData {
      roles: vec![
        ("user".to_string(), vec!["read".to_string()], None),
      ],
    };

    let rbac = RbacBuilder::try_from(data).unwrap().build().unwrap();
    assert!(rbac.is_granted("user", "read"));
  }
}

use std::collections::HashMap;

pub trait NavigationItem {
  fn add(&mut self, item: NavItem) -> isize;
  fn remove<F>(&mut self, pred: F) -> Option<NavItem>
  where
    F: Fn(&NavItem) -> bool;
  fn find<F>(&self, pred: F) -> Option<&NavItem>
  where
    F: Fn(&NavItem) -> bool;
  fn find_all<F>(&self, pred: F) -> Vec<&NavItem>
  where
    F: Fn(&NavItem) -> bool;
  fn find_by_id(&self, id: &str) -> Option<&NavItem>;
  fn has_children(&self) -> bool;
  fn size(&mut self) -> isize;
}

/// Represents a navigation item (page) in the navigation tree
/// This is a Rust implementation inspired by Laminas Navigation component
#[derive(Clone, Builder, Debug, PartialEq)]
#[builder(setter(into, strip_option), default)]
pub struct NavItem {
  /// Whether this page is currently active
  pub active: bool,

  /// HTML attributes for the page (e.g., data-* attributes)
  pub attributes: Option<HashMap<String, String>>,

  /// CSS class(es) for styling this page
  pub class: Option<String>,

  /// Whether to render only children (hide parent)
  pub children_only: bool,

  /// Fragment identifier (anchor) for the URI
  pub fragment: Option<String>,

  /// Unique identifier for this page
  pub id: Option<String>,

  /// Child navigation items
  pub items: Option<Vec<NavItem>>,

  /// Display label for the page
  pub label: Option<String>,

  /// Order/position in the navigation
  pub order: Option<i32>,

  /// ACL privilege for access control
  pub privilege: Option<String>,

  /// ACL resource for access control
  pub resource: Option<String>,

  /// Forward link relations (rel attribute)
  pub rel: Option<HashMap<String, String>>,

  /// Reverse link relations (rev attribute)
  pub rev: Option<HashMap<String, String>>,

  /// Target attribute (e.g., _blank, _self)
  pub target: Option<String>,

  /// Title attribute (tooltip)
  pub title: Option<String>,

  /// URI/URL for the page
  pub uri: Option<String>,

  /// Whether this page is visible
  pub visible: bool,

  /// Custom properties for extensibility
  pub custom_properties: Option<HashMap<String, serde_json::Value>>,

  // Internal state management
  #[builder(default)]
  _stored_size: isize,
  #[builder(default)]
  _reevaluate_order: bool,
  #[builder(default)]
  _reevaluate_size: bool,
}

impl NavItem {
  pub fn new() -> Self {
    NavItem {
      active: false,
      attributes: None,
      class: None,
      children_only: false,
      fragment: None,
      id: None,
      items: None,
      label: None,
      order: None,
      privilege: None,
      resource: None,
      rel: None,
      rev: None,
      target: None,
      title: None,
      uri: None,
      visible: true,
      custom_properties: None,

      _stored_size: 1,
      _reevaluate_order: false,
      _reevaluate_size: false,
    }
  }

  /// Checks if this nav item has any child items
  pub fn has_children(&self) -> bool {
    self.items.as_ref().map_or(false, |items| !items.is_empty())
  }

  /// Checks if this nav item is active
  pub fn is_active(&self) -> bool {
    self.active
  }

  /// Sets the active state of this nav item
  pub fn set_active(&mut self, active: bool) {
    self.active = active;
  }

  /// Gets the URI for this nav item, including fragment if present
  pub fn get_href(&self) -> Option<String> {
    self.uri.as_ref().map(|uri| {
      if let Some(ref fragment) = self.fragment {
        format!("{}#{}", uri, fragment)
      } else {
        uri.clone()
      }
    })
  }

  /// Recursively sorts children by order
  fn sort_children(&mut self) {
    if let Some(ref mut items) = self.items {
      items.sort_by_key(|item| item.order.unwrap_or(0));
      for item in items {
        item.sort_children();
      }
    }
    self._reevaluate_order = false;
  }

  /// Internal helper for removing with borrowed predicate
  fn remove_internal<F>(&mut self, pred: &F) -> Option<NavItem>
  where
    F: Fn(&NavItem) -> bool,
  {
    self._reevaluate_size = true;

    if let Some(ref mut items) = self.items {
      // Find the position of the item to remove
      if let Some(pos) = items.iter().position(|item| pred(item)) {
        return Some(items.remove(pos));
      }

      // Recursively search in children
      for item in items.iter_mut() {
        if let Some(removed) = item.remove_internal(pred) {
          return Some(removed);
        }
      }
    }

    None
  }

  /// Internal helper for finding with borrowed predicate
  fn find_internal<F>(&self, pred: &F) -> Option<&NavItem>
  where
    F: Fn(&NavItem) -> bool,
  {
    // Check current level
    if let Some(ref items) = self.items {
      for item in items {
        if pred(item) {
          return Some(item);
        }
      }

      // Recursively search in children
      for item in items {
        if let Some(found) = item.find_internal(pred) {
          return Some(found);
        }
      }
    }

    None
  }

  /// Internal helper for finding all with borrowed predicate
  fn find_all_internal<'a, F>(&'a self, pred: &F, results: &mut Vec<&'a NavItem>)
  where
    F: Fn(&NavItem) -> bool,
  {
    if let Some(ref items) = self.items {
      for item in items {
        if pred(item) {
          results.push(item);
        }
        // Recursively collect from children
        item.find_all_internal(pred, results);
      }
    }
  }
}

impl Default for NavItem {
  fn default() -> Self {
    NavItem::new()
  }
}

impl NavigationItem for NavItem {
  /// Adds a child navigation item
  fn add(&mut self, item: NavItem) -> isize {
    self._reevaluate_size = true;
    self._reevaluate_order = true;

    if self.items.is_none() {
      self.items = Some(vec![item]);
    } else {
      self.items.as_mut().unwrap().push(item);
    }

    self.size()
  }

  /// Removes a navigation item matching the predicate
  fn remove<F>(&mut self, pred: F) -> Option<NavItem>
  where
    F: Fn(&NavItem) -> bool,
  {
    self.remove_internal(&pred)
  }

  /// Finds the first navigation item matching the predicate
  fn find<F>(&self, pred: F) -> Option<&NavItem>
  where
    F: Fn(&NavItem) -> bool,
  {
    self.find_internal(&pred)
  }

  /// Finds all navigation items matching the predicate
  fn find_all<F>(&self, pred: F) -> Vec<&NavItem>
  where
    F: Fn(&NavItem) -> bool,
  {
    let mut results = Vec::new();
    self.find_all_internal(&pred, &mut results);
    results
  }

  /// Finds a navigation item by its ID
  fn find_by_id(&self, id: &str) -> Option<&NavItem> {
    self.find(|item| item.id.as_deref() == Some(id))
  }

  /// Checks if this item has children
  fn has_children(&self) -> bool {
    self.has_children()
  }

  /// Gets the total number of navigation items in the tree
  fn size(&mut self) -> isize {
    if !self._reevaluate_size {
      return self._stored_size;
    }

    let mut size = 1;
    if let Some(items) = self.items.as_deref_mut() {
      for item in items {
        size += item.size();
      }
    }
    self._stored_size = size;
    self._reevaluate_size = false;
    size
  }
}

/// A container for managing navigation trees
/// This is the main entry point for creating and managing navigation structures
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Container {
  /// Root navigation items
  pub items: Vec<NavItem>,
}

impl Container {
  /// Creates a new empty navigation container
  pub fn new() -> Self {
    Container { items: Vec::new() }
  }

  /// Adds a navigation item to the root level
  pub fn add(&mut self, item: NavItem) {
    self.items.push(item);
  }

  /// Removes a navigation item matching the predicate
  pub fn remove(&mut self, pred: impl Fn(&NavItem) -> bool) -> Option<NavItem> {
    // Try to find and remove at root level
    if let Some(pos) = self.items.iter().position(|item| pred(item)) {
      return Some(self.items.remove(pos));
    }

    // Recursively search in children
    for item in self.items.iter_mut() {
      if let Some(removed) = item.remove_internal(&pred) {
        return Some(removed);
      }
    }

    None
  }

  /// Finds the first item matching the predicate
  pub fn find(&self, pred: impl Fn(&NavItem) -> bool) -> Option<&NavItem> {
    for item in &self.items {
      if pred(item) {
        return Some(item);
      }
      if let Some(found) = item.find_internal(&pred) {
        return Some(found);
      }
    }
    None
  }

  /// Finds all items matching the predicate
  pub fn find_all(&self, pred: impl Fn(&NavItem) -> bool) -> Vec<&NavItem> {
    let mut results = Vec::new();
    for item in &self.items {
      if pred(item) {
        results.push(item);
      }
      item.find_all_internal(&pred, &mut results);
    }
    results
  }

  /// Finds an item by its ID
  pub fn find_by_id(&self, id: &str) -> Option<&NavItem> {
    self.find(|item| item.id.as_deref() == Some(id))
  }

  /// Gets the total number of items in the container
  pub fn size(&self) -> usize {
    let mut size = 0;
    for item in &self.items {
      let mut item_mut = item.clone();
      size += item_mut.size() as usize;
    }
    size
  }

  /// Checks if the container is empty
  pub fn is_empty(&self) -> bool {
    self.items.is_empty()
  }

  /// Sorts all items in the container by order
  pub fn sort(&mut self) {
    self.items.sort_by_key(|item| item.order.unwrap_or(0));
    for item in &mut self.items {
      item.sort_children();
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_nav_item_default() {
    let nav = NavItem::default();
    assert_eq!(nav.active, false);
    assert_eq!(nav.visible, true);
    assert_eq!(nav.order, None);
    assert!(nav.items.is_none());
  }

  #[test]
  fn test_nav_item_add_children() {
    let mut nav = NavItem::default();
    assert_eq!(nav.size(), 1);

    let child1 = NavItem {
      label: Some("Child 1".to_string()),
      ..Default::default()
    };
    let child2 = NavItem {
      label: Some("Child 2".to_string()),
      ..Default::default()
    };

    nav.add(child1);
    nav.add(child2);
    assert_eq!(nav.size(), 3);
    assert!(nav.has_children());
  }

  #[test]
  fn test_nav_item_nested_children() {
    let mut root = NavItem::default();

    let mut child1 = NavItem {
      label: Some("Child 1".to_string()),
      ..Default::default()
    };
    child1.add(NavItem {
      label: Some("Grandchild 1".to_string()),
      ..Default::default()
    });
    child1.add(NavItem {
      label: Some("Grandchild 2".to_string()),
      ..Default::default()
    });

    root.add(child1);
    assert_eq!(root.size(), 4); // root + child1 + 2 grandchildren
  }

  #[test]
  fn test_nav_item_find() {
    let mut root = NavItem::default();

    let child1 = NavItem {
      id: Some("page1".to_string()),
      label: Some("Page 1".to_string()),
      ..Default::default()
    };

    let child2 = NavItem {
      id: Some("page2".to_string()),
      label: Some("Page 2".to_string()),
      ..Default::default()
    };

    root.add(child1);
    root.add(child2);

    let found = root.find(|item| item.label.as_deref() == Some("Page 2"));
    assert!(found.is_some());
    assert_eq!(found.unwrap().id.as_deref(), Some("page2"));
  }

  #[test]
  fn test_nav_item_find_by_id() {
    let mut root = NavItem::default();

    let child = NavItem {
      id: Some("test-page".to_string()),
      label: Some("Test Page".to_string()),
      ..Default::default()
    };

    root.add(child);

    let found = root.find_by_id("test-page");
    assert!(found.is_some());
    assert_eq!(found.unwrap().label.as_deref(), Some("Test Page"));
  }

  #[test]
  fn test_nav_item_find_all() {
    let mut root = NavItem::default();

    root.add(NavItem {
      label: Some("Home".to_string()),
      active: true,
      ..Default::default()
    });

    root.add(NavItem {
      label: Some("About".to_string()),
      active: false,
      ..Default::default()
    });

    root.add(NavItem {
      label: Some("Contact".to_string()),
      active: true,
      ..Default::default()
    });

    let active_items = root.find_all(|item| item.active);
    assert_eq!(active_items.len(), 2);
  }

  #[test]
  fn test_nav_item_remove() {
    let mut root = NavItem::default();

    root.add(NavItem {
      id: Some("page1".to_string()),
      label: Some("Page 1".to_string()),
      ..Default::default()
    });

    root.add(NavItem {
      id: Some("page2".to_string()),
      label: Some("Page 2".to_string()),
      ..Default::default()
    });

    let removed = root.remove(|item| item.id.as_deref() == Some("page1"));
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().label.as_deref(), Some("Page 1"));

    let found = root.find_by_id("page1");
    assert!(found.is_none());
  }

  #[test]
  fn test_nav_item_get_href() {
    let item = NavItem {
      uri: Some("/path/to/page".to_string()),
      fragment: Some("section".to_string()),
      ..Default::default()
    };

    assert_eq!(item.get_href(), Some("/path/to/page#section".to_string()));

    let item_no_fragment = NavItem {
      uri: Some("/path/to/page".to_string()),
      ..Default::default()
    };

    assert_eq!(item_no_fragment.get_href(), Some("/path/to/page".to_string()));
  }

  #[test]
  fn test_nav_item_active_state() {
    let mut item = NavItem::default();
    assert!(!item.is_active());

    item.set_active(true);
    assert!(item.is_active());
  }

  #[test]
  fn test_container_basic() {
    let mut container = Container::new();
    assert!(container.is_empty());
    assert_eq!(container.size(), 0);

    container.add(NavItem {
      label: Some("Home".to_string()),
      ..Default::default()
    });

    assert!(!container.is_empty());
    assert_eq!(container.size(), 1);
  }

  #[test]
  fn test_container_find() {
    let mut container = Container::new();

    container.add(NavItem {
      id: Some("home".to_string()),
      label: Some("Home".to_string()),
      ..Default::default()
    });

    container.add(NavItem {
      id: Some("about".to_string()),
      label: Some("About".to_string()),
      ..Default::default()
    });

    let found = container.find_by_id("about");
    assert!(found.is_some());
    assert_eq!(found.unwrap().label.as_deref(), Some("About"));
  }

  #[test]
  fn test_container_remove() {
    let mut container = Container::new();

    container.add(NavItem {
      id: Some("page1".to_string()),
      ..Default::default()
    });

    container.add(NavItem {
      id: Some("page2".to_string()),
      ..Default::default()
    });

    let removed = container.remove(|item| item.id.as_deref() == Some("page1"));
    assert!(removed.is_some());
    assert_eq!(container.size(), 1);
  }

  #[test]
  fn test_container_sort() {
    let mut container = Container::new();

    container.add(NavItem {
      label: Some("Third".to_string()),
      order: Some(3),
      ..Default::default()
    });

    container.add(NavItem {
      label: Some("First".to_string()),
      order: Some(1),
      ..Default::default()
    });

    container.add(NavItem {
      label: Some("Second".to_string()),
      order: Some(2),
      ..Default::default()
    });

    container.sort();

    assert_eq!(container.items[0].label.as_deref(), Some("First"));
    assert_eq!(container.items[1].label.as_deref(), Some("Second"));
    assert_eq!(container.items[2].label.as_deref(), Some("Third"));
  }

  #[test]
  fn test_nav_item_builder() {
    let item = NavItemBuilder::default()
      .label("Home")
      .uri("/home")
      .active(true)
      .visible(true)
      .order(1)
      .build()
      .unwrap();

    assert_eq!(item.label.as_deref(), Some("Home"));
    assert_eq!(item.uri.as_deref(), Some("/home"));
    assert!(item.active);
    assert!(item.visible);
    assert_eq!(item.order, Some(1));
  }
}

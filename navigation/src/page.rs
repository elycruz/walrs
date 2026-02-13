use crate::error::{NavigationError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single navigation page/item in the navigation tree.
///
/// A `Page` is a node in a hierarchical navigation structure that can contain:
/// - Basic properties like label, URI, and title
/// - Access control settings (resource, privilege)
/// - Visibility and active state flags
/// - Child pages forming a tree structure
/// - Custom attributes as key-value pairs
///
/// # Examples
///
/// ```
/// use walrs_navigation::Page;
///
/// let page = Page::builder()
///     .label("Home")
///     .uri("/")
///     .build();
///
/// assert_eq!(page.label.as_deref(), Some("Home"));
/// assert_eq!(page.uri.as_deref(), Some("/"));
/// assert!(page.visible);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Page {
    /// Display label for the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// URI/URL for the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,

    /// Page title (for HTML title tag, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Fragment identifier (e.g., "#section")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment: Option<String>,

    /// Route name for routing systems
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,

    /// ACL resource identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,

    /// ACL privilege identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privilege: Option<String>,

    /// Whether the page is currently active
    #[serde(default)]
    pub active: bool,

    /// Whether the page is visible
    #[serde(default = "default_true")]
    pub visible: bool,

    /// CSS class for the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,

    /// ID attribute for the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Target attribute (e.g., "_blank")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// Custom attributes as key-value pairs
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attributes: HashMap<String, String>,

    /// Child pages
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pages: Vec<Page>,

    /// Display order (lower values appear first)
    #[serde(default)]
    pub order: i32,
}

fn default_true() -> bool {
    true
}

impl Page {
    /// Creates a new `PageBuilder` for constructing a page.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let page = Page::builder()
    ///     .label("About")
    ///     .uri("/about")
    ///     .build();
    /// ```
    pub fn builder() -> PageBuilder {
        PageBuilder::default()
    }

    /// Creates a new empty page.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let page = Page::new();
    /// assert!(page.label.is_none());
    /// assert!(page.visible);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut page = Page::builder().label("Home").build();
    /// page.set_attribute("data-id", "home");
    /// assert_eq!(page.attributes.get("data-id"), Some(&"home".to_string()));
    /// ```
    pub fn set_attribute<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Removes a custom attribute, returning its value if it was present.
    pub fn remove_attribute(&mut self, key: &str) -> Option<String> {
        self.attributes.remove(key)
    }

    /// Returns the full URI including fragment if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let page = Page::builder()
    ///     .uri("/about")
    ///     .fragment("team")
    ///     .build();
    /// assert_eq!(page.href(), Some("/about#team".to_string()));
    /// ```
    pub fn href(&self) -> Option<String> {
        self.uri.as_ref().map(|uri| {
            if let Some(fragment) = &self.fragment {
                format!("{}#{}", uri, fragment)
            } else {
                uri.clone()
            }
        })
    }

    /// Checks whether this page or any descendant is active.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut parent = Page::builder().label("Products").build();
    /// let child = Page::builder().label("Books").active(true).build();
    /// parent.add_page(child);
    ///
    /// assert!(parent.is_active_branch());
    /// ```
    pub fn is_active_branch(&self) -> bool {
        if self.active {
            return true;
        }
        self.pages.iter().any(|p| p.is_active_branch())
    }

    /// Checks whether this page has a specific child page (by reference equality
    /// using URI comparison), optionally searching recursively.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut parent = Page::builder().label("Products").build();
    /// parent.add_page(Page::builder().label("Books").uri("/books").build());
    ///
    /// assert!(parent.has_page(|p| p.uri.as_deref() == Some("/books"), false));
    /// assert!(!parent.has_page(|p| p.uri.as_deref() == Some("/nope"), false));
    /// ```
    pub fn has_page<F>(&self, predicate: F, recursive: bool) -> bool
    where
        F: Fn(&Page) -> bool + Copy,
    {
        for page in &self.pages {
            if predicate(page) {
                return true;
            }
            if recursive && page.has_page(predicate, true) {
                return true;
            }
        }
        false
    }

    /// Finds all pages matching a predicate, searching recursively.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut root = Page::builder().label("Root").build();
    /// root.add_page(Page::builder().label("A").visible(true).build());
    /// root.add_page(Page::builder().label("B").visible(false).build());
    /// root.add_page(Page::builder().label("C").visible(true).build());
    ///
    /// let visible = root.find_all_pages(|p| p.visible);
    /// assert_eq!(visible.len(), 3); // Root + A + C
    /// ```
    pub fn find_all_pages<F>(&self, predicate: F) -> Vec<&Page>
    where
        F: Fn(&Page) -> bool + Copy,
    {
        let mut result = Vec::new();
        self.collect_pages(&predicate, &mut result);
        result
    }

    fn collect_pages<'a, F>(&'a self, predicate: &F, result: &mut Vec<&'a Page>)
    where
        F: Fn(&Page) -> bool,
    {
        if predicate(self) {
            result.push(self);
        }
        for page in &self.pages {
            page.collect_pages(predicate, result);
        }
    }

    /// Adds a child page.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut parent = Page::builder().label("Products").build();
    /// let child = Page::builder().label("Books").build();
    ///
    /// parent.add_page(child);
    /// assert_eq!(parent.pages.len(), 1);
    /// ```
    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
        self.sort_pages();
    }

    /// Removes a child page at the given index.
    ///
    /// # Errors
    ///
    /// Returns `NavigationError::InvalidIndex` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut parent = Page::builder().label("Products").build();
    /// let child = Page::builder().label("Books").build();
    /// parent.add_page(child);
    ///
    /// let removed = parent.remove_page(0).unwrap();
    /// assert_eq!(removed.label.as_deref(), Some("Books"));
    /// assert_eq!(parent.pages.len(), 0);
    /// ```
    pub fn remove_page(&mut self, index: usize) -> Result<Page> {
        if index >= self.pages.len() {
            return Err(NavigationError::InvalidIndex(index));
        }
        Ok(self.pages.remove(index))
    }

    /// Finds a page by predicate, searching recursively.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut root = Page::builder().label("Root").build();
    /// let child = Page::builder().label("Child").uri("/child").build();
    /// root.add_page(child);
    ///
    /// let found = root.find_page(|p| p.uri.as_deref() == Some("/child"));
    /// assert!(found.is_some());
    /// assert_eq!(found.unwrap().label.as_deref(), Some("Child"));
    /// ```
    pub fn find_page<F>(&self, predicate: F) -> Option<&Page>
    where
        F: Fn(&Page) -> bool + Copy,
    {
        if predicate(self) {
            return Some(self);
        }

        for page in &self.pages {
            if let Some(found) = page.find_page(predicate) {
                return Some(found);
            }
        }

        None
    }

    /// Finds a mutable page by predicate, searching recursively.
    pub fn find_page_mut<F>(&mut self, predicate: F) -> Option<&mut Page>
    where
        F: Fn(&Page) -> bool + Copy,
    {
        if predicate(self) {
            return Some(self);
        }

        for page in &mut self.pages {
            if let Some(found) = page.find_page_mut(predicate) {
                return Some(found);
            }
        }

        None
    }

    /// Returns the total number of pages (including this page and all descendants).
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut root = Page::builder().label("Root").build();
    /// assert_eq!(root.count(), 1);
    ///
    /// root.add_page(Page::builder().label("Child 1").build());
    /// root.add_page(Page::builder().label("Child 2").build());
    /// assert_eq!(root.count(), 3);
    /// ```
    pub fn count(&self) -> usize {
        1 + self.pages.iter().map(|p| p.count()).sum::<usize>()
    }

    /// Returns whether this page has any child pages.
    pub fn has_pages(&self) -> bool {
        !self.pages.is_empty()
    }

    /// Sorts child pages by their order value.
    fn sort_pages(&mut self) {
        self.pages.sort_by_key(|p| p.order);
    }

    /// Adds multiple child pages at once.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut parent = Page::builder().label("Root").build();
    /// parent.add_pages(vec![
    ///     Page::builder().label("A").build(),
    ///     Page::builder().label("B").build(),
    /// ]);
    /// assert_eq!(parent.pages.len(), 2);
    /// ```
    pub fn add_pages(&mut self, pages: Vec<Page>) {
        for page in pages {
            self.pages.push(page);
        }
        self.sort_pages();
    }

    /// Removes all child pages.
    pub fn remove_pages(&mut self) {
        self.pages.clear();
    }

    /// Performs a depth-first traversal of the page tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut root = Page::builder().label("Root").build();
    /// root.add_page(Page::builder().label("Child 1").build());
    /// root.add_page(Page::builder().label("Child 2").build());
    ///
    /// let mut labels = Vec::new();
    /// root.traverse(&mut |page| {
    ///     if let Some(label) = page.label.as_deref() {
    ///         labels.push(label.to_string());
    ///     }
    /// });
    ///
    /// assert_eq!(labels, vec!["Root", "Child 1", "Child 2"]);
    /// ```
    pub fn traverse<F>(&self, f: &mut F)
    where
        F: FnMut(&Page),
    {
        f(self);
        for page in &self.pages {
            page.traverse(f);
        }
    }

    /// Performs a depth-first traversal of the page tree with depth information.
    ///
    /// The callback receives the page and its depth (0-based).
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut root = Page::builder().label("Root").build();
    /// let mut child = Page::builder().label("Child").build();
    /// child.add_page(Page::builder().label("Grandchild").build());
    /// root.add_page(child);
    ///
    /// let mut items = Vec::new();
    /// root.traverse_with_depth(0, &mut |page, depth| {
    ///     items.push((page.label.as_deref().unwrap_or("").to_string(), depth));
    /// });
    ///
    /// assert_eq!(items, vec![
    ///     ("Root".to_string(), 0),
    ///     ("Child".to_string(), 1),
    ///     ("Grandchild".to_string(), 2),
    /// ]);
    /// ```
    pub fn traverse_with_depth<F>(&self, depth: usize, f: &mut F)
    where
        F: FnMut(&Page, usize),
    {
        f(self, depth);
        for page in &self.pages {
            page.traverse_with_depth(depth + 1, f);
        }
    }

    /// Returns only visible child pages.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut root = Page::builder().label("Root").build();
    /// root.add_page(Page::builder().label("Visible").visible(true).build());
    /// root.add_page(Page::builder().label("Hidden").visible(false).build());
    ///
    /// assert_eq!(root.visible_pages().len(), 1);
    /// assert_eq!(root.visible_pages()[0].label.as_deref(), Some("Visible"));
    /// ```
    pub fn visible_pages(&self) -> Vec<&Page> {
        self.pages.iter().filter(|p| p.visible).collect()
    }
}

impl Default for Page {
    fn default() -> Self {
        Self {
            label: None,
            uri: None,
            title: None,
            fragment: None,
            route: None,
            resource: None,
            privilege: None,
            active: false,
            visible: true,
            class: None,
            id: None,
            target: None,
            attributes: HashMap::new(),
            pages: Vec::new(),
            order: 0,
        }
    }
}

/// Builder for constructing `Page` instances.
#[derive(Debug)]
pub struct PageBuilder {
    label: Option<String>,
    uri: Option<String>,
    title: Option<String>,
    fragment: Option<String>,
    route: Option<String>,
    resource: Option<String>,
    privilege: Option<String>,
    active: bool,
    visible: bool,
    class: Option<String>,
    id: Option<String>,
    target: Option<String>,
    attributes: HashMap<String, String>,
    pages: Vec<Page>,
    order: i32,
}

impl Default for PageBuilder {
    fn default() -> Self {
        Self {
            label: None,
            uri: None,
            title: None,
            fragment: None,
            route: None,
            resource: None,
            privilege: None,
            active: false,
            visible: true,
            class: None,
            id: None,
            target: None,
            attributes: HashMap::new(),
            pages: Vec::new(),
            order: 0,
        }
    }
}

impl PageBuilder {
    /// Sets the label.
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the URI.
    pub fn uri<S: Into<String>>(mut self, uri: S) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Sets the title.
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the fragment.
    pub fn fragment<S: Into<String>>(mut self, fragment: S) -> Self {
        self.fragment = Some(fragment.into());
        self
    }

    /// Sets the route.
    pub fn route<S: Into<String>>(mut self, route: S) -> Self {
        self.route = Some(route.into());
        self
    }

    /// Sets the resource.
    pub fn resource<S: Into<String>>(mut self, resource: S) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Sets the privilege.
    pub fn privilege<S: Into<String>>(mut self, privilege: S) -> Self {
        self.privilege = Some(privilege.into());
        self
    }

    /// Sets the active state.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Sets the visible state.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets the CSS class.
    pub fn class<S: Into<String>>(mut self, class: S) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Sets the ID.
    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the target.
    pub fn target<S: Into<String>>(mut self, target: S) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Adds a custom attribute.
    pub fn attribute<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Adds a child page.
    pub fn page(mut self, page: Page) -> Self {
        self.pages.push(page);
        self
    }

    /// Sets the order.
    pub fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Builds the `Page`.
    pub fn build(mut self) -> Page {
        // Sort child pages by order
        self.pages.sort_by_key(|p| p.order);

        Page {
            label: self.label,
            uri: self.uri,
            title: self.title,
            fragment: self.fragment,
            route: self.route,
            resource: self.resource,
            privilege: self.privilege,
            active: self.active,
            visible: self.visible,
            class: self.class,
            id: self.id,
            target: self.target,
            attributes: self.attributes,
            pages: self.pages,
            order: self.order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_new() {
        let page = Page::new();
        assert!(page.label.is_none());
        assert!(page.uri.is_none());
        assert!(page.title.is_none());
        assert!(page.fragment.is_none());
        assert!(page.route.is_none());
        assert!(page.resource.is_none());
        assert!(page.privilege.is_none());
        assert!(!page.active);
        assert!(page.visible);
        assert!(page.class.is_none());
        assert!(page.id.is_none());
        assert!(page.target.is_none());
        assert!(page.attributes.is_empty());
        assert!(page.pages.is_empty());
        assert_eq!(page.order, 0);
    }

    #[test]
    fn test_page_builder_all_properties() {
        let page = Page::builder()
            .label("Test")
            .uri("/test")
            .title("Test Title")
            .fragment("section1")
            .route("test_route")
            .resource("test_resource")
            .privilege("test_privilege")
            .active(true)
            .visible(false)
            .class("nav-item")
            .id("test-id")
            .target("_blank")
            .attribute("data-x", "y")
            .order(10)
            .build();

        assert_eq!(page.label.as_deref(), Some("Test"));
        assert_eq!(page.uri.as_deref(), Some("/test"));
        assert_eq!(page.title.as_deref(), Some("Test Title"));
        assert_eq!(page.fragment.as_deref(), Some("section1"));
        assert_eq!(page.route.as_deref(), Some("test_route"));
        assert_eq!(page.resource.as_deref(), Some("test_resource"));
        assert_eq!(page.privilege.as_deref(), Some("test_privilege"));
        assert!(page.active);
        assert!(!page.visible);
        assert_eq!(page.class.as_deref(), Some("nav-item"));
        assert_eq!(page.id.as_deref(), Some("test-id"));
        assert_eq!(page.target.as_deref(), Some("_blank"));
        assert_eq!(page.attributes.get("data-x"), Some(&"y".to_string()));
        assert_eq!(page.order, 10);
    }

    #[test]
    fn test_page_builder_with_child_pages() {
        let page = Page::builder()
            .label("Parent")
            .page(Page::builder().label("Child 1").order(2).build())
            .page(Page::builder().label("Child 2").order(1).build())
            .build();

        assert_eq!(page.pages.len(), 2);
        // Builder sorts children by order
        assert_eq!(page.pages[0].label.as_deref(), Some("Child 2"));
        assert_eq!(page.pages[1].label.as_deref(), Some("Child 1"));
    }

    #[test]
    fn test_page_hierarchy() {
        let mut root = Page::builder().label("Root").build();
        let child1 = Page::builder().label("Child 1").build();
        let child2 = Page::builder().label("Child 2").build();

        root.add_page(child1);
        root.add_page(child2);

        assert_eq!(root.pages.len(), 2);
        assert_eq!(root.count(), 3);
        assert!(root.has_pages());
    }

    #[test]
    fn test_find_page() {
        let mut root = Page::builder().label("Root").build();
        let child = Page::builder().label("Target").uri("/target").build();
        root.add_page(child);

        let found = root.find_page(|p| p.uri.as_deref() == Some("/target"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().label.as_deref(), Some("Target"));
    }

    #[test]
    fn test_find_page_self() {
        let root = Page::builder().label("Root").uri("/root").build();
        let found = root.find_page(|p| p.uri.as_deref() == Some("/root"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().label.as_deref(), Some("Root"));
    }

    #[test]
    fn test_find_page_not_found() {
        let root = Page::builder().label("Root").build();
        let found = root.find_page(|p| p.uri.as_deref() == Some("/nope"));
        assert!(found.is_none());
    }

    #[test]
    fn test_find_page_deep_nested() {
        let mut root = Page::builder().label("Root").build();
        let mut l1 = Page::builder().label("L1").build();
        let mut l2 = Page::builder().label("L2").build();
        l2.add_page(Page::builder().label("Deep").uri("/deep").build());
        l1.add_page(l2);
        root.add_page(l1);

        let found = root.find_page(|p| p.uri.as_deref() == Some("/deep"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().label.as_deref(), Some("Deep"));
    }

    #[test]
    fn test_find_page_mut() {
        let mut root = Page::builder().label("Root").build();
        root.add_page(Page::builder().label("Child").uri("/child").build());

        let found = root.find_page_mut(|p| p.uri.as_deref() == Some("/child"));
        assert!(found.is_some());
        found.unwrap().active = true;

        let child = root.find_page(|p| p.uri.as_deref() == Some("/child")).unwrap();
        assert!(child.active);
    }

    #[test]
    fn test_remove_page() {
        let mut root = Page::builder().label("Root").build();
        root.add_page(Page::builder().label("Child").build());

        assert_eq!(root.pages.len(), 1);
        let removed = root.remove_page(0).unwrap();
        assert_eq!(removed.label.as_deref(), Some("Child"));
        assert_eq!(root.pages.len(), 0);
    }

    #[test]
    fn test_remove_page_invalid_index() {
        let mut root = Page::builder().label("Root").build();
        let result = root.remove_page(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_page_ordering() {
        let mut root = Page::builder().label("Root").build();
        root.add_page(Page::builder().label("Third").order(3).build());
        root.add_page(Page::builder().label("First").order(1).build());
        root.add_page(Page::builder().label("Second").order(2).build());

        assert_eq!(root.pages[0].label.as_deref(), Some("First"));
        assert_eq!(root.pages[1].label.as_deref(), Some("Second"));
        assert_eq!(root.pages[2].label.as_deref(), Some("Third"));
    }

    #[test]
    fn test_traverse() {
        let mut root = Page::builder().label("Root").build();
        let mut child1 = Page::builder().label("Child 1").build();
        child1.add_page(Page::builder().label("Grandchild").build());
        root.add_page(child1);
        root.add_page(Page::builder().label("Child 2").build());

        let mut count = 0;
        root.traverse(&mut |_| {
            count += 1;
        });

        assert_eq!(count, 4); // Root + 2 children + 1 grandchild
    }

    #[test]
    fn test_traverse_with_depth() {
        let mut root = Page::builder().label("Root").build();
        let mut child = Page::builder().label("Child").build();
        child.add_page(Page::builder().label("Grandchild").build());
        root.add_page(child);

        let mut items = Vec::new();
        root.traverse_with_depth(0, &mut |page, depth| {
            items.push((page.label.as_deref().unwrap_or("").to_string(), depth));
        });

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], ("Root".to_string(), 0));
        assert_eq!(items[1], ("Child".to_string(), 1));
        assert_eq!(items[2], ("Grandchild".to_string(), 2));
    }

    #[test]
    fn test_direct_field_access() {
        let mut page = Page::new();
        page.label = Some("Home".to_string());
        page.uri = Some("/".to_string());
        page.title = Some("Home Page".to_string());
        page.fragment = Some("top".to_string());
        page.route = Some("home".to_string());
        page.resource = Some("mvc:home".to_string());
        page.privilege = Some("view".to_string());
        page.active = true;
        page.visible = false;
        page.class = Some("nav-home".to_string());
        page.id = Some("home-link".to_string());
        page.target = Some("_self".to_string());
        page.order = 5;

        assert_eq!(page.label.as_deref(), Some("Home"));
        assert_eq!(page.uri.as_deref(), Some("/"));
        assert_eq!(page.title.as_deref(), Some("Home Page"));
        assert_eq!(page.fragment.as_deref(), Some("top"));
        assert_eq!(page.route.as_deref(), Some("home"));
        assert_eq!(page.resource.as_deref(), Some("mvc:home"));
        assert_eq!(page.privilege.as_deref(), Some("view"));
        assert!(page.active);
        assert!(!page.visible);
        assert_eq!(page.class.as_deref(), Some("nav-home"));
        assert_eq!(page.id.as_deref(), Some("home-link"));
        assert_eq!(page.target.as_deref(), Some("_self"));
        assert_eq!(page.order, 5);
    }

    #[test]
    fn test_attributes() {
        let mut page = Page::new();
        page.set_attribute("data-toggle", "dropdown");
        page.set_attribute("data-id", "123");
        assert_eq!(
            page.attributes.get("data-toggle"),
            Some(&"dropdown".to_string())
        );

        let removed = page.remove_attribute("data-id");
        assert_eq!(removed, Some("123".to_string()));
        assert!(!page.attributes.contains_key("data-id"));

        let removed_none = page.remove_attribute("nonexistent");
        assert!(removed_none.is_none());
    }

    #[test]
    fn test_attributes_direct() {
        let mut page = Page::new();
        page.attributes.insert("key".to_string(), "val".to_string());
        assert_eq!(page.attributes.get("key"), Some(&"val".to_string()));
    }

    #[test]
    fn test_href() {
        // With URI and fragment
        let page = Page::builder().uri("/about").fragment("team").build();
        assert_eq!(page.href(), Some("/about#team".to_string()));

        // With URI only
        let page = Page::builder().uri("/about").build();
        assert_eq!(page.href(), Some("/about".to_string()));

        // No URI
        let page = Page::new();
        assert_eq!(page.href(), None);
    }

    #[test]
    fn test_is_active_branch() {
        // Direct active
        let page = Page::builder().active(true).build();
        assert!(page.is_active_branch());

        // Active through child
        let mut parent = Page::builder().label("Parent").build();
        parent.add_page(Page::builder().active(true).build());
        assert!(parent.is_active_branch());

        // Active through grandchild
        let mut root = Page::builder().label("Root").build();
        let mut child = Page::builder().label("Child").build();
        child.add_page(Page::builder().active(true).build());
        root.add_page(child);
        assert!(root.is_active_branch());

        // No active pages
        let page = Page::builder().label("Inactive").build();
        assert!(!page.is_active_branch());
    }

    #[test]
    fn test_has_page() {
        let mut root = Page::builder().label("Root").build();
        let mut child = Page::builder().label("Child").uri("/child").build();
        child.add_page(Page::builder().label("Grandchild").uri("/gc").build());
        root.add_page(child);

        // Non-recursive: only direct children
        assert!(root.has_page(|p| p.uri.as_deref() == Some("/child"), false));
        assert!(!root.has_page(|p| p.uri.as_deref() == Some("/gc"), false));

        // Recursive: finds grandchildren
        assert!(root.has_page(|p| p.uri.as_deref() == Some("/gc"), true));
        assert!(!root.has_page(|p| p.uri.as_deref() == Some("/nope"), true));
    }

    #[test]
    fn test_find_all_pages() {
        let mut root = Page::builder().label("Root").visible(true).build();
        root.add_page(Page::builder().label("A").visible(true).build());
        root.add_page(Page::builder().label("B").visible(false).build());
        root.add_page(Page::builder().label("C").visible(true).build());

        let visible = root.find_all_pages(|p| p.visible);
        assert_eq!(visible.len(), 3); // Root + A + C

        let all = root.find_all_pages(|_| true);
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_add_pages() {
        let mut root = Page::builder().label("Root").build();
        root.add_pages(vec![
            Page::builder().label("A").order(2).build(),
            Page::builder().label("B").order(1).build(),
        ]);
        assert_eq!(root.pages.len(), 2);
        assert_eq!(root.pages[0].label.as_deref(), Some("B")); // sorted by order
    }

    #[test]
    fn test_remove_pages() {
        let mut root = Page::builder().label("Root").build();
        root.add_page(Page::builder().label("A").build());
        root.add_page(Page::builder().label("B").build());
        assert_eq!(root.pages.len(), 2);
        root.remove_pages();
        assert!(root.pages.is_empty());
    }

    #[test]
    fn test_visible_pages() {
        let mut root = Page::builder().label("Root").build();
        root.add_page(Page::builder().label("Visible").visible(true).build());
        root.add_page(Page::builder().label("Hidden").visible(false).build());
        root.add_page(Page::builder().label("Also Visible").visible(true).build());

        let visible = root.visible_pages();
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].label.as_deref(), Some("Visible"));
        assert_eq!(visible[1].label.as_deref(), Some("Also Visible"));
    }

    #[test]
    fn test_page_count_empty() {
        let page = Page::new();
        assert_eq!(page.count(), 1);
    }

    #[test]
    fn test_page_default() {
        let page1 = Page::new();
        let page2 = Page::default();
        assert_eq!(page1, page2);
    }
}

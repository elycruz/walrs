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
/// assert_eq!(page.label(), Some("Home"));
/// assert_eq!(page.uri(), Some("/"));
/// assert!(page.is_visible());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Page {
    /// Display label for the page
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,

    /// URI/URL for the page
    #[serde(skip_serializing_if = "Option::is_none")]
    uri: Option<String>,

    /// Page title (for HTML title tag, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    /// Fragment identifier (e.g., "#section")
    #[serde(skip_serializing_if = "Option::is_none")]
    fragment: Option<String>,

    /// Route name for routing systems
    #[serde(skip_serializing_if = "Option::is_none")]
    route: Option<String>,

    /// ACL resource identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<String>,

    /// ACL privilege identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    privilege: Option<String>,

    /// Whether the page is currently active
    #[serde(default)]
    active: bool,

    /// Whether the page is visible
    #[serde(default = "default_true")]
    visible: bool,

    /// CSS class for the page
    #[serde(skip_serializing_if = "Option::is_none")]
    class: Option<String>,

    /// ID attribute for the page
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,

    /// Target attribute (e.g., "_blank")
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,

    /// Custom attributes as key-value pairs
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    attributes: HashMap<String, String>,

    /// Child pages
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pages: Vec<Page>,

    /// Display order (lower values appear first)
    #[serde(default)]
    order: i32,
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
    /// assert!(page.label().is_none());
    /// assert!(page.is_visible());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the page label.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the page URI.
    pub fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }

    /// Returns the page title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the fragment identifier.
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    /// Returns the route name.
    pub fn route(&self) -> Option<&str> {
        self.route.as_deref()
    }

    /// Returns the resource identifier for ACL.
    pub fn resource(&self) -> Option<&str> {
        self.resource.as_deref()
    }

    /// Returns the privilege identifier for ACL.
    pub fn privilege(&self) -> Option<&str> {
        self.privilege.as_deref()
    }

    /// Returns whether the page is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Returns whether the page is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the CSS class.
    pub fn class(&self) -> Option<&str> {
        self.class.as_deref()
    }

    /// Returns the ID attribute.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Returns the target attribute.
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// Returns a reference to the custom attributes.
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    /// Returns a reference to the child pages.
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    /// Returns a mutable reference to the child pages (internal use).
    pub(crate) fn pages_mut(&mut self) -> &mut Vec<Page> {
        &mut self.pages
    }

    /// Returns the display order.
    pub fn order(&self) -> i32 {
        self.order
    }

    /// Sets the active state.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Sets the visible state.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
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
    /// assert_eq!(parent.pages().len(), 1);
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
    /// assert_eq!(removed.label(), Some("Books"));
    /// assert_eq!(parent.pages().len(), 0);
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
    /// let found = root.find_page(|p| p.uri() == Some("/child"));
    /// assert!(found.is_some());
    /// assert_eq!(found.unwrap().label(), Some("Child"));
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
    ///     if let Some(label) = page.label() {
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
    fn test_page_builder() {
        let page = Page::builder()
            .label("Test")
            .uri("/test")
            .active(true)
            .order(10)
            .build();

        assert_eq!(page.label(), Some("Test"));
        assert_eq!(page.uri(), Some("/test"));
        assert!(page.is_active());
        assert_eq!(page.order(), 10);
    }

    #[test]
    fn test_page_hierarchy() {
        let mut root = Page::builder().label("Root").build();
        let child1 = Page::builder().label("Child 1").build();
        let child2 = Page::builder().label("Child 2").build();

        root.add_page(child1);
        root.add_page(child2);

        assert_eq!(root.pages().len(), 2);
        assert_eq!(root.count(), 3);
    }

    #[test]
    fn test_find_page() {
        let mut root = Page::builder().label("Root").build();
        let child = Page::builder().label("Target").uri("/target").build();
        root.add_page(child);

        let found = root.find_page(|p| p.uri() == Some("/target"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().label(), Some("Target"));
    }

    #[test]
    fn test_remove_page() {
        let mut root = Page::builder().label("Root").build();
        root.add_page(Page::builder().label("Child").build());

        assert_eq!(root.pages().len(), 1);
        let removed = root.remove_page(0).unwrap();
        assert_eq!(removed.label(), Some("Child"));
        assert_eq!(root.pages().len(), 0);
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

        let pages = root.pages();
        assert_eq!(pages[0].label(), Some("First"));
        assert_eq!(pages[1].label(), Some("Second"));
        assert_eq!(pages[2].label(), Some("Third"));
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
}

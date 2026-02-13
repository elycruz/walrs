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

    /// Sets the label.
    pub fn set_label<S: Into<String>>(&mut self, label: S) {
        self.label = Some(label.into());
    }

    /// Sets the URI.
    pub fn set_uri<S: Into<String>>(&mut self, uri: S) {
        self.uri = Some(uri.into());
    }

    /// Sets the title.
    pub fn set_title<S: Into<String>>(&mut self, title: S) {
        self.title = Some(title.into());
    }

    /// Sets the fragment.
    pub fn set_fragment<S: Into<String>>(&mut self, fragment: S) {
        self.fragment = Some(fragment.into());
    }

    /// Sets the route.
    pub fn set_route<S: Into<String>>(&mut self, route: S) {
        self.route = Some(route.into());
    }

    /// Sets the resource.
    pub fn set_resource<S: Into<String>>(&mut self, resource: S) {
        self.resource = Some(resource.into());
    }

    /// Sets the privilege.
    pub fn set_privilege<S: Into<String>>(&mut self, privilege: S) {
        self.privilege = Some(privilege.into());
    }

    /// Sets the active state.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Sets the visible state.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Sets the CSS class.
    pub fn set_class<S: Into<String>>(&mut self, class: S) {
        self.class = Some(class.into());
    }

    /// Sets the ID attribute.
    pub fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = Some(id.into());
    }

    /// Sets the target attribute.
    pub fn set_target<S: Into<String>>(&mut self, target: S) {
        self.target = Some(target.into());
    }

    /// Sets the display order.
    pub fn set_order(&mut self, order: i32) {
        self.order = order;
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
    /// assert_eq!(page.attributes().get("data-id"), Some(&"home".to_string()));
    /// ```
    pub fn set_attribute<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Removes a custom attribute, returning its value if it was present.
    pub fn remove_attribute(&mut self, key: &str) -> Option<String> {
        self.attributes.remove(key)
    }

    /// Returns a mutable reference to the custom attributes.
    pub fn attributes_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.attributes
    }

    /// Gets a page property by name. Returns the property value as a string
    /// if it exists, or `None` if the property is not set or the name is unknown.
    ///
    /// Supported property names: `"label"`, `"uri"`, `"title"`, `"fragment"`,
    /// `"route"`, `"resource"`, `"privilege"`, `"class"`, `"id"`, `"target"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let page = Page::builder().label("Home").uri("/").build();
    /// assert_eq!(page.get("label"), Some("Home"));
    /// assert_eq!(page.get("uri"), Some("/"));
    /// assert_eq!(page.get("unknown"), None);
    /// ```
    pub fn get(&self, property: &str) -> Option<&str> {
        match property {
            "label" => self.label(),
            "uri" => self.uri(),
            "title" => self.title(),
            "fragment" => self.fragment(),
            "route" => self.route(),
            "resource" => self.resource(),
            "privilege" => self.privilege(),
            "class" => self.class(),
            "id" => self.id(),
            "target" => self.target(),
            _ => None,
        }
    }

    /// Sets a page property by name. Returns `true` if the property was
    /// recognized and set, `false` if the property name is unknown.
    ///
    /// Supported property names: `"label"`, `"uri"`, `"title"`, `"fragment"`,
    /// `"route"`, `"resource"`, `"privilege"`, `"class"`, `"id"`, `"target"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Page;
    ///
    /// let mut page = Page::new();
    /// assert!(page.set("label", "Home"));
    /// assert_eq!(page.label(), Some("Home"));
    /// assert!(!page.set("unknown", "value"));
    /// ```
    pub fn set(&mut self, property: &str, value: &str) -> bool {
        match property {
            "label" => self.set_label(value),
            "uri" => self.set_uri(value),
            "title" => self.set_title(value),
            "fragment" => self.set_fragment(value),
            "route" => self.set_route(value),
            "resource" => self.set_resource(value),
            "privilege" => self.set_privilege(value),
            "class" => self.set_class(value),
            "id" => self.set_id(value),
            "target" => self.set_target(value),
            _ => return false,
        }
        true
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
    /// assert!(parent.has_page(|p| p.uri() == Some("/books"), false));
    /// assert!(!parent.has_page(|p| p.uri() == Some("/nope"), false));
    /// ```
    pub fn has_page<F>(&self, predicate: F, recursive: bool) -> bool
    where
        F: Fn(&Page) -> bool + Copy,
    {
        for page in &self.pages {
            if predicate(page) {
                return true;
            }
            if recursive {
                if page.has_page(predicate, true) {
                    return true;
                }
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
    /// let visible = root.find_all_pages(|p| p.is_visible());
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
    /// assert_eq!(parent.pages().len(), 2);
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

    /// Performs a depth-first traversal with depth information.
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
    ///     items.push((page.label().unwrap_or("").to_string(), depth));
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
    /// assert_eq!(root.visible_pages()[0].label(), Some("Visible"));
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
        assert!(page.label().is_none());
        assert!(page.uri().is_none());
        assert!(page.title().is_none());
        assert!(page.fragment().is_none());
        assert!(page.route().is_none());
        assert!(page.resource().is_none());
        assert!(page.privilege().is_none());
        assert!(!page.is_active());
        assert!(page.is_visible());
        assert!(page.class().is_none());
        assert!(page.id().is_none());
        assert!(page.target().is_none());
        assert!(page.attributes().is_empty());
        assert!(page.pages().is_empty());
        assert_eq!(page.order(), 0);
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

        assert_eq!(page.label(), Some("Test"));
        assert_eq!(page.uri(), Some("/test"));
        assert_eq!(page.title(), Some("Test Title"));
        assert_eq!(page.fragment(), Some("section1"));
        assert_eq!(page.route(), Some("test_route"));
        assert_eq!(page.resource(), Some("test_resource"));
        assert_eq!(page.privilege(), Some("test_privilege"));
        assert!(page.is_active());
        assert!(!page.is_visible());
        assert_eq!(page.class(), Some("nav-item"));
        assert_eq!(page.id(), Some("test-id"));
        assert_eq!(page.target(), Some("_blank"));
        assert_eq!(page.attributes().get("data-x"), Some(&"y".to_string()));
        assert_eq!(page.order(), 10);
    }

    #[test]
    fn test_page_builder_with_child_pages() {
        let page = Page::builder()
            .label("Parent")
            .page(Page::builder().label("Child 1").order(2).build())
            .page(Page::builder().label("Child 2").order(1).build())
            .build();

        assert_eq!(page.pages().len(), 2);
        // Builder sorts children by order
        assert_eq!(page.pages()[0].label(), Some("Child 2"));
        assert_eq!(page.pages()[1].label(), Some("Child 1"));
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
        assert!(root.has_pages());
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
    fn test_find_page_self() {
        let root = Page::builder().label("Root").uri("/root").build();
        let found = root.find_page(|p| p.uri() == Some("/root"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().label(), Some("Root"));
    }

    #[test]
    fn test_find_page_not_found() {
        let root = Page::builder().label("Root").build();
        let found = root.find_page(|p| p.uri() == Some("/nope"));
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

        let found = root.find_page(|p| p.uri() == Some("/deep"));
        assert!(found.is_some());
        assert_eq!(found.unwrap().label(), Some("Deep"));
    }

    #[test]
    fn test_find_page_mut() {
        let mut root = Page::builder().label("Root").build();
        root.add_page(Page::builder().label("Child").uri("/child").build());

        let found = root.find_page_mut(|p| p.uri() == Some("/child"));
        assert!(found.is_some());
        found.unwrap().set_active(true);

        let child = root.find_page(|p| p.uri() == Some("/child")).unwrap();
        assert!(child.is_active());
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

    #[test]
    fn test_setters() {
        let mut page = Page::new();
        page.set_label("Home");
        page.set_uri("/");
        page.set_title("Home Page");
        page.set_fragment("top");
        page.set_route("home");
        page.set_resource("mvc:home");
        page.set_privilege("view");
        page.set_active(true);
        page.set_visible(false);
        page.set_class("nav-home");
        page.set_id("home-link");
        page.set_target("_self");
        page.set_order(5);

        assert_eq!(page.label(), Some("Home"));
        assert_eq!(page.uri(), Some("/"));
        assert_eq!(page.title(), Some("Home Page"));
        assert_eq!(page.fragment(), Some("top"));
        assert_eq!(page.route(), Some("home"));
        assert_eq!(page.resource(), Some("mvc:home"));
        assert_eq!(page.privilege(), Some("view"));
        assert!(page.is_active());
        assert!(!page.is_visible());
        assert_eq!(page.class(), Some("nav-home"));
        assert_eq!(page.id(), Some("home-link"));
        assert_eq!(page.target(), Some("_self"));
        assert_eq!(page.order(), 5);
    }

    #[test]
    fn test_get_set_dynamic() {
        let mut page = Page::new();
        assert!(page.set("label", "Test"));
        assert!(page.set("uri", "/test"));
        assert!(page.set("title", "Title"));
        assert!(page.set("fragment", "frag"));
        assert!(page.set("route", "r"));
        assert!(page.set("resource", "res"));
        assert!(page.set("privilege", "priv"));
        assert!(page.set("class", "cls"));
        assert!(page.set("id", "myid"));
        assert!(page.set("target", "_blank"));
        assert!(!page.set("unknown", "value"));

        assert_eq!(page.get("label"), Some("Test"));
        assert_eq!(page.get("uri"), Some("/test"));
        assert_eq!(page.get("title"), Some("Title"));
        assert_eq!(page.get("fragment"), Some("frag"));
        assert_eq!(page.get("route"), Some("r"));
        assert_eq!(page.get("resource"), Some("res"));
        assert_eq!(page.get("privilege"), Some("priv"));
        assert_eq!(page.get("class"), Some("cls"));
        assert_eq!(page.get("id"), Some("myid"));
        assert_eq!(page.get("target"), Some("_blank"));
        assert_eq!(page.get("unknown"), None);
    }

    #[test]
    fn test_attributes() {
        let mut page = Page::new();
        page.set_attribute("data-toggle", "dropdown");
        page.set_attribute("data-id", "123");
        assert_eq!(
            page.attributes().get("data-toggle"),
            Some(&"dropdown".to_string())
        );

        let removed = page.remove_attribute("data-id");
        assert_eq!(removed, Some("123".to_string()));
        assert!(page.attributes().get("data-id").is_none());

        let removed_none = page.remove_attribute("nonexistent");
        assert!(removed_none.is_none());
    }

    #[test]
    fn test_attributes_mut() {
        let mut page = Page::new();
        page.attributes_mut().insert("key".to_string(), "val".to_string());
        assert_eq!(page.attributes().get("key"), Some(&"val".to_string()));
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
        assert!(root.has_page(|p| p.uri() == Some("/child"), false));
        assert!(!root.has_page(|p| p.uri() == Some("/gc"), false));

        // Recursive: finds grandchildren
        assert!(root.has_page(|p| p.uri() == Some("/gc"), true));
        assert!(!root.has_page(|p| p.uri() == Some("/nope"), true));
    }

    #[test]
    fn test_find_all_pages() {
        let mut root = Page::builder().label("Root").visible(true).build();
        root.add_page(Page::builder().label("A").visible(true).build());
        root.add_page(Page::builder().label("B").visible(false).build());
        root.add_page(Page::builder().label("C").visible(true).build());

        let visible = root.find_all_pages(|p| p.is_visible());
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
        assert_eq!(root.pages().len(), 2);
        assert_eq!(root.pages()[0].label(), Some("B")); // sorted by order
    }

    #[test]
    fn test_remove_pages() {
        let mut root = Page::builder().label("Root").build();
        root.add_page(Page::builder().label("A").build());
        root.add_page(Page::builder().label("B").build());
        assert_eq!(root.pages().len(), 2);
        root.remove_pages();
        assert!(root.pages().is_empty());
    }

    #[test]
    fn test_traverse_with_depth() {
        let mut root = Page::builder().label("Root").build();
        let mut child = Page::builder().label("Child").build();
        child.add_page(Page::builder().label("Grandchild").build());
        root.add_page(child);

        let mut items = Vec::new();
        root.traverse_with_depth(0, &mut |page, depth| {
            items.push((page.label().unwrap_or("").to_string(), depth));
        });

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], ("Root".to_string(), 0));
        assert_eq!(items[1], ("Child".to_string(), 1));
        assert_eq!(items[2], ("Grandchild".to_string(), 2));
    }

    #[test]
    fn test_visible_pages() {
        let mut root = Page::builder().label("Root").build();
        root.add_page(Page::builder().label("Visible").visible(true).build());
        root.add_page(Page::builder().label("Hidden").visible(false).build());
        root.add_page(Page::builder().label("Also Visible").visible(true).build());

        let visible = root.visible_pages();
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].label(), Some("Visible"));
        assert_eq!(visible[1].label(), Some("Also Visible"));
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

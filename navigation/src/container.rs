use crate::error::{NavigationError, Result};
use crate::page::Page;
use serde::{Deserialize, Serialize};

/// A container for managing a navigation tree.
///
/// `Container` is the root of a navigation structure and provides methods
/// for managing, searching, and traversing navigation pages. It implements
/// the same hierarchical structure as individual pages.
///
/// # Examples
///
/// ```
/// use walrs_navigation::{Container, Page};
///
/// let mut nav = Container::new();
///
/// nav.add_page(Page::builder().label("Home").uri("/").build());
/// nav.add_page(Page::builder().label("About").uri("/about").build());
///
/// assert_eq!(nav.count(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Container {
    /// Root-level pages in the container
    #[serde(default)]
    pages: Vec<Page>,
}

impl Container {
    /// Creates a new empty navigation container.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::Container;
    ///
    /// let nav = Container::new();
    /// assert_eq!(nav.count(), 0);
    /// ```
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    /// Creates a container from a vector of pages.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let pages = vec![
    ///     Page::builder().label("Home").build(),
    ///     Page::builder().label("About").build(),
    /// ];
    ///
    /// let nav = Container::from_pages(pages);
    /// assert_eq!(nav.count(), 2);
    /// ```
    pub fn from_pages(pages: Vec<Page>) -> Self {
        let mut container = Self { pages };
        container.sort_pages();
        container
    }

    /// Adds a page to the container.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// let page = Page::builder().label("Products").uri("/products").build();
    ///
    /// nav.add_page(page);
    /// assert_eq!(nav.count(), 1);
    /// ```
    pub fn add_page(&mut self, page: Page) {
        self.pages.push(page);
        self.sort_pages();
    }

    /// Removes a page at the given index.
    ///
    /// # Errors
    ///
    /// Returns `NavigationError::InvalidIndex` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Home").build());
    ///
    /// let removed = nav.remove_page(0).unwrap();
    /// assert_eq!(removed.label.as_deref(), Some("Home"));
    /// assert_eq!(nav.count(), 0);
    /// ```
    pub fn remove_page(&mut self, index: usize) -> Result<Page> {
        if index >= self.pages.len() {
            return Err(NavigationError::InvalidIndex(index));
        }
        Ok(self.pages.remove(index))
    }

    /// Returns a reference to the pages in the container.
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    /// Returns a mutable reference to the pages in the container.
    pub fn pages_mut(&mut self) -> &mut [Page] {
        &mut self.pages
    }

    /// Returns the total number of pages in the container (including all descendants).
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// let mut parent = Page::builder().label("Products").build();
    /// parent.add_page(Page::builder().label("Books").build());
    /// parent.add_page(Page::builder().label("Electronics").build());
    ///
    /// nav.add_page(parent);
    /// assert_eq!(nav.count(), 3); // 1 parent + 2 children
    /// ```
    pub fn count(&self) -> usize {
        self.pages.iter().map(|p| p.count()).sum()
    }

    /// Returns whether the container has any pages.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Finds a page by predicate, searching recursively through all pages.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Home").uri("/").build());
    /// nav.add_page(Page::builder().label("About").uri("/about").build());
    ///
    /// let found = nav.find_page(|p| p.uri.as_deref() == Some("/about"));
    /// assert!(found.is_some());
    /// assert_eq!(found.unwrap().label.as_deref(), Some("About"));
    /// ```
    pub fn find_page<F>(&self, predicate: F) -> Option<&Page>
    where
        F: Fn(&Page) -> bool + Copy,
    {
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
        for page in &mut self.pages {
            if let Some(found) = page.find_page_mut(predicate) {
                return Some(found);
            }
        }
        None
    }

    /// Finds a page by URI.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Home").uri("/").build());
    ///
    /// let found = nav.find_by_uri("/");
    /// assert!(found.is_some());
    /// ```
    pub fn find_by_uri(&self, uri: &str) -> Option<&Page> {
        self.find_page(|p| p.uri.as_deref() == Some(uri))
    }

    /// Finds a page by label.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Home").build());
    ///
    /// let found = nav.find_by_label("Home");
    /// assert!(found.is_some());
    /// ```
    pub fn find_by_label(&self, label: &str) -> Option<&Page> {
        self.find_page(|p| p.label.as_deref() == Some(label))
    }

    /// Finds a page by ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().id("home").label("Home").build());
    ///
    /// let found = nav.find_by_id("home");
    /// assert!(found.is_some());
    /// assert_eq!(found.unwrap().label.as_deref(), Some("Home"));
    /// ```
    pub fn find_by_id(&self, id: &str) -> Option<&Page> {
        self.find_page(|p| p.id.as_deref() == Some(id))
    }

    /// Clears all pages from the container.
    pub fn clear(&mut self) {
        self.pages.clear();
    }

    /// Adds multiple pages at once.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_pages(vec![
    ///     Page::builder().label("Home").build(),
    ///     Page::builder().label("About").build(),
    /// ]);
    /// assert_eq!(nav.count(), 2);
    /// ```
    pub fn add_pages(&mut self, pages: Vec<Page>) {
        for page in pages {
            self.pages.push(page);
        }
        self.sort_pages();
    }

    /// Replaces all pages in the container with the given pages.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Old").build());
    ///
    /// nav.set_pages(vec![
    ///     Page::builder().label("New 1").build(),
    ///     Page::builder().label("New 2").build(),
    /// ]);
    /// assert_eq!(nav.count(), 2);
    /// ```
    pub fn set_pages(&mut self, pages: Vec<Page>) {
        self.pages = pages;
        self.sort_pages();
    }

    /// Checks if the container contains a page matching the predicate,
    /// optionally searching recursively.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// let mut parent = Page::builder().label("Products").build();
    /// parent.add_page(Page::builder().label("Books").uri("/books").build());
    /// nav.add_page(parent);
    ///
    /// // Non-recursive: only root pages
    /// assert!(!nav.has_page(|p| p.uri.as_deref() == Some("/books"), false));
    /// // Recursive: finds nested pages too
    /// assert!(nav.has_page(|p| p.uri.as_deref() == Some("/books"), true));
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

    /// Finds the first page matching a property value, searching recursively.
    ///
    /// Uses the `Page::get` method for dynamic property lookup.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Home").uri("/").build());
    /// nav.add_page(Page::builder().label("About").uri("/about").build());
    ///
    /// let found = nav.find_one_by("label", "About");
    /// assert!(found.is_some());
    /// assert_eq!(found.unwrap().uri.as_deref(), Some("/about"));
    /// ```
    pub fn find_one_by(&self, property: &str, value: &str) -> Option<&Page> {
        self.find_page(|p| p.get(property) == Some(value))
    }

    /// Finds all pages matching a property value, searching recursively.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("A").class("nav").build());
    /// nav.add_page(Page::builder().label("B").class("nav").build());
    /// nav.add_page(Page::builder().label("C").class("other").build());
    ///
    /// let found = nav.find_all_by("class", "nav");
    /// assert_eq!(found.len(), 2);
    /// ```
    pub fn find_all_by(&self, property: &str, value: &str) -> Vec<&Page> {
        let mut result = Vec::new();
        for page in &self.pages {
            result.extend(page.find_all_pages(|p| p.get(property) == Some(value)));
        }
        result
    }

    /// Finds a page by route name.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Home").route("home").build());
    ///
    /// let found = nav.find_by_route("home");
    /// assert!(found.is_some());
    /// ```
    pub fn find_by_route(&self, route: &str) -> Option<&Page> {
        self.find_page(|p| p.route.as_deref() == Some(route))
    }

    /// Returns only visible root-level pages.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Visible").visible(true).build());
    /// nav.add_page(Page::builder().label("Hidden").visible(false).build());
    ///
    /// assert_eq!(nav.visible_pages().len(), 1);
    /// ```
    pub fn visible_pages(&self) -> Vec<&Page> {
        self.pages.iter().filter(|p| p.visible).collect()
    }

    /// Performs a depth-first traversal of all pages.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Home").build());
    /// nav.add_page(Page::builder().label("About").build());
    ///
    /// let mut labels = Vec::new();
    /// nav.traverse(&mut |page| {
    ///     if let Some(label) = page.label.as_deref() {
    ///         labels.push(label.to_string());
    ///     }
    /// });
    ///
    /// assert_eq!(labels.len(), 2);
    /// ```
    pub fn traverse<F>(&self, f: &mut F)
    where
        F: FnMut(&Page),
    {
        for page in &self.pages {
            page.traverse(f);
        }
    }

    /// Performs a depth-first traversal of all pages with depth information.
    ///
    /// The callback receives the page and its depth (0-based from root pages).
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// let mut parent = Page::builder().label("Parent").build();
    /// parent.add_page(Page::builder().label("Child").build());
    /// nav.add_page(parent);
    ///
    /// let mut items = Vec::new();
    /// nav.traverse_with_depth(&mut |page, depth| {
    ///     items.push((page.label.as_deref().unwrap_or("").to_string(), depth));
    /// });
    ///
    /// assert_eq!(items, vec![
    ///     ("Parent".to_string(), 0),
    ///     ("Child".to_string(), 1),
    /// ]);
    /// ```
    pub fn traverse_with_depth<F>(&self, f: &mut F)
    where
        F: FnMut(&Page, usize),
    {
        for page in &self.pages {
            page.traverse_with_depth(0, f);
        }
    }

    /// Returns the breadcrumb trail to the active page.
    ///
    /// Returns a vector of references to pages forming the path from the
    /// root to the currently active page.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// let mut products = Page::builder().label("Products").uri("/products").build();
    /// let books = Page::builder().label("Books").uri("/products/books").active(true).build();
    /// products.add_page(books);
    /// nav.add_page(products);
    ///
    /// let crumbs = nav.breadcrumbs();
    /// assert_eq!(crumbs.len(), 2);
    /// assert_eq!(crumbs[0].label.as_deref(), Some("Products"));
    /// assert_eq!(crumbs[1].label.as_deref(), Some("Books"));
    /// ```
    pub fn breadcrumbs(&self) -> Vec<&Page> {
        for page in &self.pages {
            let mut trail = Vec::new();
            if Self::find_active_trail(page, &mut trail) {
                return trail;
            }
        }
        Vec::new()
    }

    /// Recursively finds the trail to an active page.
    fn find_active_trail<'a>(page: &'a Page, trail: &mut Vec<&'a Page>) -> bool {
        trail.push(page);
        if page.active {
            return true;
        }
        for child in &page.pages {
            if Self::find_active_trail(child, trail) {
                return true;
            }
        }
        trail.pop();
        false
    }

    /// Returns an iterator over the root-level pages.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Home").build());
    /// nav.add_page(Page::builder().label("About").build());
    ///
    /// let mut count = 0;
    /// for _page in nav.iter() {
    ///     count += 1;
    /// }
    /// assert_eq!(count, 2);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Page> {
        self.pages.iter()
    }

    /// Sorts pages by their order value.
    fn sort_pages(&mut self) {
        self.pages.sort_by_key(|p| p.order);
    }

    /// Sets the active page based on the current URI.
    ///
    /// This will mark all pages as inactive except the one matching the given URI.
    ///
    /// # Examples
    ///
    /// ```
    /// use walrs_navigation::{Container, Page};
    ///
    /// let mut nav = Container::new();
    /// nav.add_page(Page::builder().label("Home").uri("/").build());
    /// nav.add_page(Page::builder().label("About").uri("/about").build());
    ///
    /// nav.set_active_by_uri("/about");
    ///
    /// let about = nav.find_by_uri("/about").unwrap();
    /// assert!(about.active);
    /// ```
    pub fn set_active_by_uri(&mut self, uri: &str) {
        // First, deactivate all pages
        for page in &mut self.pages {
            Self::deactivate_recursive(page);
        }

        // Then activate the matching page
        if let Some(page) = self.find_page_mut(|p| p.uri.as_deref() == Some(uri)) {
            page.active = true;
        }
    }

    /// Recursively deactivates a page and all its descendants.
    fn deactivate_recursive(page: &mut Page) {
        page.active = false;
        for child in &mut page.pages {
            Self::deactivate_recursive(child);
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

// Implement IntoIterator for Container
impl IntoIterator for Container {
    type Item = Page;
    type IntoIter = std::vec::IntoIter<Page>;

    fn into_iter(self) -> Self::IntoIter {
        self.pages.into_iter()
    }
}

// Implement FromIterator
impl FromIterator<Page> for Container {
    fn from_iter<I: IntoIterator<Item = Page>>(iter: I) -> Self {
        Self::from_pages(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_new() {
        let nav = Container::new();
        assert!(nav.is_empty());
        assert_eq!(nav.count(), 0);
    }

    #[test]
    fn test_container_default() {
        let nav1 = Container::new();
        let nav2 = Container::default();
        assert_eq!(nav1, nav2);
    }

    #[test]
    fn test_add_page() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());
        nav.add_page(Page::builder().label("About").build());

        assert_eq!(nav.count(), 2);
        assert!(!nav.is_empty());
    }

    #[test]
    fn test_add_pages() {
        let mut nav = Container::new();
        nav.add_pages(vec![
            Page::builder().label("A").order(2).build(),
            Page::builder().label("B").order(1).build(),
        ]);
        assert_eq!(nav.count(), 2);
        assert_eq!(nav.pages()[0].label.as_deref(), Some("B"));
    }

    #[test]
    fn test_set_pages() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Old").build());
        nav.set_pages(vec![
            Page::builder().label("New 1").build(),
            Page::builder().label("New 2").build(),
        ]);
        assert_eq!(nav.count(), 2);
        assert!(nav.find_by_label("Old").is_none());
    }

    #[test]
    fn test_remove_page() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());

        let removed = nav.remove_page(0).unwrap();
        assert_eq!(removed.label.as_deref(), Some("Home"));
        assert!(nav.is_empty());
    }

    #[test]
    fn test_remove_invalid_index() {
        let mut nav = Container::new();
        let result = nav.remove_page(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_by_uri() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").uri("/").build());
        nav.add_page(Page::builder().label("About").uri("/about").build());

        let found = nav.find_by_uri("/about");
        assert!(found.is_some());
        assert_eq!(found.unwrap().label.as_deref(), Some("About"));

        let not_found = nav.find_by_uri("/notfound");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_by_label() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());

        let found = nav.find_by_label("Home");
        assert!(found.is_some());
    }

    #[test]
    fn test_find_by_id() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().id("home").label("Home").build());

        let found = nav.find_by_id("home");
        assert!(found.is_some());
        assert_eq!(found.unwrap().label.as_deref(), Some("Home"));
    }

    #[test]
    fn test_find_by_route() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").route("home").build());

        let found = nav.find_by_route("home");
        assert!(found.is_some());

        let not_found = nav.find_by_route("nope");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_has_page() {
        let mut nav = Container::new();
        let mut parent = Page::builder().label("Products").build();
        parent.add_page(Page::builder().label("Books").uri("/books").build());
        nav.add_page(parent);

        // Non-recursive
        assert!(nav.has_page(|p| p.label.as_deref() == Some("Products"), false));
        assert!(!nav.has_page(|p| p.uri.as_deref() == Some("/books"), false));

        // Recursive
        assert!(nav.has_page(|p| p.uri.as_deref() == Some("/books"), true));
        assert!(!nav.has_page(|p| p.uri.as_deref() == Some("/nope"), true));
    }

    #[test]
    fn test_nested_pages() {
        let mut nav = Container::new();
        let mut products = Page::builder().label("Products").uri("/products").build();
        products.add_page(Page::builder().label("Books").uri("/products/books").build());
        nav.add_page(products);

        assert_eq!(nav.count(), 2);

        let found = nav.find_by_uri("/products/books");
        assert!(found.is_some());
        assert_eq!(found.unwrap().label.as_deref(), Some("Books"));
    }

    #[test]
    fn test_clear() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());
        nav.add_page(Page::builder().label("About").build());

        assert_eq!(nav.count(), 2);
        nav.clear();
        assert!(nav.is_empty());
    }

    #[test]
    fn test_traverse() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());
        nav.add_page(Page::builder().label("About").build());

        let mut count = 0;
        nav.traverse(&mut |_| {
            count += 1;
        });

        assert_eq!(count, 2);
    }

    #[test]
    fn test_traverse_with_depth() {
        let mut nav = Container::new();
        let mut parent = Page::builder().label("Parent").build();
        parent.add_page(Page::builder().label("Child").build());
        nav.add_page(parent);

        let mut items = Vec::new();
        nav.traverse_with_depth(&mut |page, depth| {
            items.push((page.label.as_deref().unwrap_or("").to_string(), depth));
        });

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ("Parent".to_string(), 0));
        assert_eq!(items[1], ("Child".to_string(), 1));
    }

    #[test]
    fn test_breadcrumbs() {
        let mut nav = Container::new();
        let mut products = Page::builder().label("Products").uri("/products").build();
        let books = Page::builder()
            .label("Books")
            .uri("/products/books")
            .active(true)
            .build();
        products.add_page(books);
        nav.add_page(products);

        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs.len(), 2);
        assert_eq!(crumbs[0].label.as_deref(), Some("Products"));
        assert_eq!(crumbs[1].label.as_deref(), Some("Books"));
    }

    #[test]
    fn test_breadcrumbs_no_active() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());
        let crumbs = nav.breadcrumbs();
        assert!(crumbs.is_empty());
    }

    #[test]
    fn test_breadcrumbs_root_active() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").active(true).build());
        let crumbs = nav.breadcrumbs();
        assert_eq!(crumbs.len(), 1);
        assert_eq!(crumbs[0].label.as_deref(), Some("Home"));
    }

    #[test]
    fn test_visible_pages() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Visible").visible(true).build());
        nav.add_page(Page::builder().label("Hidden").visible(false).build());

        let visible = nav.visible_pages();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].label.as_deref(), Some("Visible"));
    }

    #[test]
    fn test_ordering() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Third").order(3).build());
        nav.add_page(Page::builder().label("First").order(1).build());
        nav.add_page(Page::builder().label("Second").order(2).build());

        let pages = nav.pages();
        assert_eq!(pages[0].label.as_deref(), Some("First"));
        assert_eq!(pages[1].label.as_deref(), Some("Second"));
        assert_eq!(pages[2].label.as_deref(), Some("Third"));
    }

    #[test]
    fn test_into_iterator() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());
        nav.add_page(Page::builder().label("About").build());

        let labels: Vec<_> = nav
            .into_iter()
            .filter_map(|p| p.label.clone())
            .collect();

        assert_eq!(labels, vec!["Home", "About"]);
    }

    #[test]
    fn test_iter() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").uri("/").build());
        nav.add_page(Page::builder().label("About").uri("/about").build());

        // Test basic iteration
        let labels: Vec<_> = nav.iter().filter_map(|p| p.label.as_deref()).collect();
        assert_eq!(labels, vec!["Home", "About"]);

        // Test that iter() borrows and doesn't consume
        assert_eq!(nav.count(), 2);

        // Test iteration count
        let mut count = 0;
        for _page in nav.iter() {
            count += 1;
        }
        assert_eq!(count, 2);

        // Test empty container
        let empty_nav = Container::new();
        assert_eq!(empty_nav.iter().count(), 0);
    }

    #[test]
    fn test_from_iterator() {
        let pages = vec![
            Page::builder().label("Home").build(),
            Page::builder().label("About").build(),
        ];

        let nav: Container = pages.into_iter().collect();
        assert_eq!(nav.count(), 2);
    }

    #[test]
    fn test_set_active_by_uri() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").uri("/").active(true).build());
        nav.add_page(Page::builder().label("About").uri("/about").build());

        nav.set_active_by_uri("/about");

        assert!(!nav.find_by_uri("/").unwrap().active);
        assert!(nav.find_by_uri("/about").unwrap().active);
    }

    #[test]
    fn test_set_active_by_uri_nested() {
        let mut nav = Container::new();
        let mut parent = Page::builder().label("Products").uri("/products").build();
        parent.add_page(
            Page::builder()
                .label("Books")
                .uri("/products/books")
                .build(),
        );
        nav.add_page(parent);

        nav.set_active_by_uri("/products/books");

        let books = nav.find_by_uri("/products/books").unwrap();
        assert!(books.active);
    }

    #[test]
    fn test_set_active_by_uri_nonexistent() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").uri("/").build());
        nav.set_active_by_uri("/nope");
        // No panic, home should not be active
        assert!(!nav.find_by_uri("/").unwrap().active);
    }

    #[test]
    fn test_pages_mut() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());

        let pages = nav.pages_mut();
        pages[0].label = Some("Modified".to_string());

        assert_eq!(nav.pages()[0].label.as_deref(), Some("Modified"));
    }

    #[test]
    fn test_find_page_mut() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").uri("/").build());

        let page = nav.find_page_mut(|p| p.uri.as_deref() == Some("/"));
        assert!(page.is_some());
        page.unwrap().label = Some("Modified".to_string());

        assert_eq!(nav.find_by_uri("/").unwrap().label.as_deref(), Some("Modified"));
    }

    #[test]
    fn test_find_page_mut_not_found() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());

        let page = nav.find_page_mut(|p| p.uri.as_deref() == Some("/nope"));
        assert!(page.is_none());
    }

    #[test]
    fn test_from_pages() {
        let pages = vec![
            Page::builder().label("B").order(2).build(),
            Page::builder().label("A").order(1).build(),
        ];
        let nav = Container::from_pages(pages);
        assert_eq!(nav.pages()[0].label.as_deref(), Some("A"));
        assert_eq!(nav.pages()[1].label.as_deref(), Some("B"));
    }

    #[test]
    fn test_find_one_by() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").uri("/").build());
        nav.add_page(Page::builder().label("About").uri("/about").build());

        let found = nav.find_one_by("label", "About");
        assert!(found.is_some());
        assert_eq!(found.unwrap().uri.as_deref(), Some("/about"));

        let not_found = nav.find_one_by("label", "Missing");
        assert!(not_found.is_none());

        let unknown = nav.find_one_by("unknown_prop", "value");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_find_all_by() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("A").class("nav").build());
        nav.add_page(Page::builder().label("B").class("nav").build());
        nav.add_page(Page::builder().label("C").class("other").build());

        let found = nav.find_all_by("class", "nav");
        assert_eq!(found.len(), 2);

        let not_found = nav.find_all_by("class", "missing");
        assert!(not_found.is_empty());
    }

    #[test]
    fn test_find_all_by_nested() {
        let mut nav = Container::new();
        let mut parent = Page::builder().label("Products").class("nav").build();
        parent.add_page(Page::builder().label("Books").class("nav").build());
        parent.add_page(Page::builder().label("Electronics").class("other").build());
        nav.add_page(parent);

        let found = nav.find_all_by("class", "nav");
        assert_eq!(found.len(), 2); // Products + Books
    }

    #[test]
    fn test_find_one_by_nested() {
        let mut nav = Container::new();
        let mut parent = Page::builder().label("Products").build();
        parent.add_page(Page::builder().label("Books").uri("/books").build());
        nav.add_page(parent);

        let found = nav.find_one_by("uri", "/books");
        assert!(found.is_some());
        assert_eq!(found.unwrap().label.as_deref(), Some("Books"));
    }
}

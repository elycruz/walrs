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
    /// assert_eq!(removed.label(), Some("Home"));
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
    /// let found = nav.find_page(|p| p.uri() == Some("/about"));
    /// assert!(found.is_some());
    /// assert_eq!(found.unwrap().label(), Some("About"));
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
        self.find_page(|p| p.uri() == Some(uri))
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
        self.find_page(|p| p.label() == Some(label))
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
    /// assert_eq!(found.unwrap().label(), Some("Home"));
    /// ```
    pub fn find_by_id(&self, id: &str) -> Option<&Page> {
        self.find_page(|p| p.id() == Some(id))
    }

    /// Clears all pages from the container.
    pub fn clear(&mut self) {
        self.pages.clear();
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
    ///     if let Some(label) = page.label() {
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
        self.pages.sort_by_key(|p| p.order());
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
    /// assert!(about.is_active());
    /// ```
    pub fn set_active_by_uri(&mut self, uri: &str) {
        // First, deactivate all pages
        for page in &mut self.pages {
            Self::deactivate_recursive(page);
        }

        // Then activate the matching page
        if let Some(page) = self.find_page_mut(|p| p.uri() == Some(uri)) {
            page.set_active(true);
        }
    }

    /// Recursively deactivates a page and all its descendants.
    fn deactivate_recursive(page: &mut Page) {
        page.set_active(false);
        for child in page.pages_mut() {
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
    fn test_add_page() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());
        nav.add_page(Page::builder().label("About").build());

        assert_eq!(nav.count(), 2);
        assert!(!nav.is_empty());
    }

    #[test]
    fn test_remove_page() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());

        let removed = nav.remove_page(0).unwrap();
        assert_eq!(removed.label(), Some("Home"));
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
        assert_eq!(found.unwrap().label(), Some("About"));

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
    fn test_nested_pages() {
        let mut nav = Container::new();
        let mut products = Page::builder().label("Products").uri("/products").build();
        products.add_page(Page::builder().label("Books").uri("/products/books").build());
        nav.add_page(products);

        assert_eq!(nav.count(), 2);

        let found = nav.find_by_uri("/products/books");
        assert!(found.is_some());
        assert_eq!(found.unwrap().label(), Some("Books"));
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
    fn test_ordering() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Third").order(3).build());
        nav.add_page(Page::builder().label("First").order(1).build());
        nav.add_page(Page::builder().label("Second").order(2).build());

        let pages = nav.pages();
        assert_eq!(pages[0].label(), Some("First"));
        assert_eq!(pages[1].label(), Some("Second"));
        assert_eq!(pages[2].label(), Some("Third"));
    }

    #[test]
    fn test_into_iterator() {
        let mut nav = Container::new();
        nav.add_page(Page::builder().label("Home").build());
        nav.add_page(Page::builder().label("About").build());

        let labels: Vec<_> = nav
            .into_iter()
            .filter_map(|p| p.label().map(String::from))
            .collect();

        assert_eq!(labels, vec!["Home", "About"]);
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
}

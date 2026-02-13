//! View helpers for rendering navigation structures.
//!
//! This module provides functions for rendering [`Container`] and [`Page`]
//! structures into common HTML formats:
//!
//! - **Menus**: Hierarchical `<ul>/<li>` navigation menus
//! - **Breadcrumbs**: Linear breadcrumb trails to the active page
//! - **Sitemaps**: Flat or hierarchical lists of all pages
//!
//! All functions properly escape HTML special characters to prevent XSS
//! attacks when rendering user-controlled data.
//!
//! # Examples
//!
//! ```
//! use walrs_navigation::{Container, Page};
//! use walrs_navigation::view;
//!
//! let mut nav = Container::new();
//! nav.add_page(Page::builder().label("Home").uri("/").active(true).build());
//! nav.add_page(Page::builder().label("About").uri("/about").build());
//!
//! let menu = view::render_menu(&nav);
//! assert!(menu.contains("<ul"));
//! assert!(menu.contains("Home"));
//!
//! let crumbs = view::render_breadcrumbs(&nav, " &gt; ");
//! assert!(crumbs.contains("Home"));
//! ```

use crate::container::Container;
use crate::page::Page;

/// Escapes HTML special characters to prevent XSS attacks.
///
/// # Examples
///
/// ```
/// use walrs_navigation::view::html_escape;
///
/// assert_eq!(html_escape("<script>"), "&lt;script&gt;");
/// assert_eq!(html_escape("a & b"), "a &amp; b");
/// assert_eq!(html_escape("\"hello\""), "&quot;hello&quot;");
/// ```
pub fn html_escape(text: &str) -> String {
  // '&' must be replaced first to avoid double-escaping
  text
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#x27;")
}

/// Renders the navigation container as an HTML `<ul>` menu.
///
/// Only visible pages are rendered. Active pages receive a CSS class
/// `"active"`. The rendered HTML is safe against XSS attacks.
///
/// # Examples
///
/// ```
/// use walrs_navigation::{Container, Page};
/// use walrs_navigation::view::render_menu;
///
/// let mut nav = Container::new();
/// nav.add_page(Page::builder().label("Home").uri("/").active(true).build());
/// let mut about = Page::builder().label("About").uri("/about").build();
/// about.add_page(Page::builder().label("Team").uri("/about/team").build());
/// nav.add_page(about);
///
/// let html = render_menu(&nav);
/// assert!(html.contains("<ul>"));
/// assert!(html.contains("class=\"active\""));
/// assert!(html.contains("Home"));
/// assert!(html.contains("Team"));
/// ```
pub fn render_menu(nav: &Container) -> String {
  let mut html = String::new();
  html.push_str("<ul>\n");
  for page in nav.pages() {
    if page.is_visible() {
      render_menu_page(page, &mut html, 1);
    }
  }
  html.push_str("</ul>");
  html
}

/// Renders the navigation container as an HTML `<ul>` menu with custom
/// CSS classes.
///
/// # Arguments
///
/// * `nav` - The navigation container
/// * `ul_class` - CSS class for the outer `<ul>` element
/// * `active_class` - CSS class applied to active `<li>` elements
///
/// # Examples
///
/// ```
/// use walrs_navigation::{Container, Page};
/// use walrs_navigation::view::render_menu_with_class;
///
/// let mut nav = Container::new();
/// nav.add_page(Page::builder().label("Home").uri("/").active(true).build());
///
/// let html = render_menu_with_class(&nav, "navbar-nav", "current");
/// assert!(html.contains("class=\"navbar-nav\""));
/// assert!(html.contains("class=\"current\""));
/// ```
pub fn render_menu_with_class(nav: &Container, ul_class: &str, active_class: &str) -> String {
  let mut html = String::new();
  html.push_str(&format!("<ul class=\"{}\">\n", html_escape(ul_class)));
  for page in nav.pages() {
    if page.is_visible() {
      render_menu_page_with_class(page, &mut html, 1, active_class);
    }
  }
  html.push_str("</ul>");
  html
}

fn render_menu_page(page: &Page, html: &mut String, depth: usize) {
  let indent = "  ".repeat(depth);
  let label = html_escape(page.label().unwrap_or(""));
  let href = html_escape(&page.href().unwrap_or_else(|| "#".to_string()));

  let class_attr = if page.is_active() {
    " class=\"active\""
  } else {
    ""
  };

  html.push_str(&format!(
    "{}<li{}><a href=\"{}\">{}</a>",
    indent, class_attr, href, label
  ));

  let visible_children: Vec<_> = page.pages().iter().filter(|p| p.is_visible()).collect();
  if !visible_children.is_empty() {
    html.push_str(&format!("\n{}<ul>\n", "  ".repeat(depth + 1)));
    for child in visible_children {
      render_menu_page(child, html, depth + 2);
    }
    html.push_str(&format!("{}</ul>\n{}", "  ".repeat(depth + 1), indent));
  }

  html.push_str("</li>\n");
}

fn render_menu_page_with_class(
  page: &Page,
  html: &mut String,
  depth: usize,
  active_class: &str,
) {
  let indent = "  ".repeat(depth);
  let label = html_escape(page.label().unwrap_or(""));
  let href = html_escape(&page.href().unwrap_or_else(|| "#".to_string()));

  let class_attr = if page.is_active() {
    format!(" class=\"{}\"", html_escape(active_class))
  } else {
    String::new()
  };

  html.push_str(&format!(
    "{}<li{}><a href=\"{}\">{}</a>",
    indent, class_attr, href, label
  ));

  let visible_children: Vec<_> = page.pages().iter().filter(|p| p.is_visible()).collect();
  if !visible_children.is_empty() {
    html.push_str(&format!("\n{}<ul>\n", "  ".repeat(depth + 1)));
    for child in visible_children {
      render_menu_page_with_class(child, html, depth + 2, active_class);
    }
    html.push_str(&format!("{}</ul>\n{}", "  ".repeat(depth + 1), indent));
  }

  html.push_str("</li>\n");
}

/// Renders the breadcrumb trail as an HTML string.
///
/// The separator is placed between breadcrumb items. The active page
/// is rendered as a `<span>` rather than a link.
///
/// # Examples
///
/// ```
/// use walrs_navigation::{Container, Page};
/// use walrs_navigation::view::render_breadcrumbs;
///
/// let mut nav = Container::new();
/// let mut products = Page::builder().label("Products").uri("/products").build();
/// products.add_page(
///     Page::builder().label("Books").uri("/products/books").active(true).build()
/// );
/// nav.add_page(products);
///
/// let html = render_breadcrumbs(&nav, " &gt; ");
/// assert!(html.contains("Products"));
/// assert!(html.contains("Books"));
/// ```
pub fn render_breadcrumbs(nav: &Container, separator: &str) -> String {
  let crumbs = nav.breadcrumbs();
  if crumbs.is_empty() {
    return String::new();
  }

  let mut html = String::new();
  let last_idx = crumbs.len() - 1;
  for (i, page) in crumbs.iter().enumerate() {
    let label = html_escape(page.label().unwrap_or(""));
    if i == last_idx {
      // Active page: render as span
      html.push_str(&format!("<span class=\"active\">{}</span>", label));
    } else {
      let href = html_escape(&page.href().unwrap_or_else(|| "#".to_string()));
      html.push_str(&format!("<a href=\"{}\">{}</a>", href, label));
      html.push_str(separator);
    }
  }
  html
}

/// Renders a flat sitemap as an HTML `<ul>` list.
///
/// Lists all visible pages in the navigation tree in a flat structure.
///
/// # Examples
///
/// ```
/// use walrs_navigation::{Container, Page};
/// use walrs_navigation::view::render_sitemap;
///
/// let mut nav = Container::new();
/// nav.add_page(Page::builder().label("Home").uri("/").build());
/// nav.add_page(Page::builder().label("About").uri("/about").build());
///
/// let html = render_sitemap(&nav);
/// assert!(html.contains("Home"));
/// assert!(html.contains("About"));
/// ```
pub fn render_sitemap(nav: &Container) -> String {
  let mut html = String::new();
  html.push_str("<ul class=\"sitemap\">\n");
  nav.traverse(&mut |page| {
    if page.is_visible() {
      let label = html_escape(page.label().unwrap_or(""));
      let href = html_escape(&page.href().unwrap_or_else(|| "#".to_string()));
      html.push_str(&format!("  <li><a href=\"{}\">{}</a></li>\n", href, label));
    }
  });
  html.push_str("</ul>");
  html
}

/// Renders a hierarchical sitemap as nested `<ul>` lists.
///
/// Preserves the tree structure of the navigation, rendering child pages
/// as nested lists.
///
/// # Examples
///
/// ```
/// use walrs_navigation::{Container, Page};
/// use walrs_navigation::view::render_sitemap_hierarchical;
///
/// let mut nav = Container::new();
/// let mut parent = Page::builder().label("Products").uri("/products").build();
/// parent.add_page(Page::builder().label("Books").uri("/products/books").build());
/// nav.add_page(parent);
///
/// let html = render_sitemap_hierarchical(&nav);
/// assert!(html.contains("Products"));
/// assert!(html.contains("Books"));
/// ```
pub fn render_sitemap_hierarchical(nav: &Container) -> String {
  let mut html = String::new();
  html.push_str("<ul class=\"sitemap\">\n");
  for page in nav.pages() {
    if page.is_visible() {
      render_sitemap_page(page, &mut html, 1);
    }
  }
  html.push_str("</ul>");
  html
}

fn render_sitemap_page(page: &Page, html: &mut String, depth: usize) {
  let indent = "  ".repeat(depth);
  let label = html_escape(page.label().unwrap_or(""));
  let href = html_escape(&page.href().unwrap_or_else(|| "#".to_string()));

  html.push_str(&format!(
    "{}<li><a href=\"{}\">{}</a>",
    indent, href, label
  ));

  let visible_children: Vec<_> = page.pages().iter().filter(|p| p.is_visible()).collect();
  if !visible_children.is_empty() {
    html.push_str(&format!("\n{}<ul>\n", "  ".repeat(depth + 1)));
    for child in visible_children {
      render_sitemap_page(child, html, depth + 2);
    }
    html.push_str(&format!("{}</ul>\n{}", "  ".repeat(depth + 1), indent));
  }

  html.push_str("</li>\n");
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Page;

  fn sample_nav() -> Container {
    let mut nav = Container::new();
    nav.add_page(
      Page::builder()
        .label("Home")
        .uri("/")
        .active(true)
        .order(1)
        .build(),
    );
    let mut products = Page::builder()
      .label("Products")
      .uri("/products")
      .order(2)
      .build();
    products.add_page(
      Page::builder()
        .label("Books")
        .uri("/products/books")
        .build(),
    );
    products.add_page(
      Page::builder()
        .label("Electronics")
        .uri("/products/electronics")
        .build(),
    );
    nav.add_page(products);
    nav.add_page(
      Page::builder()
        .label("About")
        .uri("/about")
        .order(3)
        .build(),
    );
    nav
  }

  #[test]
  fn test_html_escape() {
    assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    assert_eq!(html_escape("a & b"), "a &amp; b");
    assert_eq!(html_escape("\"hello\""), "&quot;hello&quot;");
    assert_eq!(html_escape("it's"), "it&#x27;s");
    assert_eq!(html_escape(""), "");
    assert_eq!(html_escape("safe text"), "safe text");
  }

  #[test]
  fn test_render_menu() {
    let nav = sample_nav();
    let html = render_menu(&nav);

    assert!(html.starts_with("<ul>"));
    assert!(html.ends_with("</ul>"));
    assert!(html.contains("Home"));
    assert!(html.contains("Products"));
    assert!(html.contains("Books"));
    assert!(html.contains("Electronics"));
    assert!(html.contains("About"));
    assert!(html.contains("class=\"active\""));
    assert!(html.contains("href=\"/\""));
    assert!(html.contains("href=\"/products\""));
  }

  #[test]
  fn test_render_menu_hidden_pages() {
    let mut nav = Container::new();
    nav.add_page(Page::builder().label("Visible").uri("/v").build());
    nav.add_page(Page::builder().label("Hidden").uri("/h").visible(false).build());

    let html = render_menu(&nav);
    assert!(html.contains("Visible"));
    assert!(!html.contains("Hidden"));
  }

  #[test]
  fn test_render_menu_with_class() {
    let nav = sample_nav();
    let html = render_menu_with_class(&nav, "nav-pills", "current");

    assert!(html.contains("class=\"nav-pills\""));
    assert!(html.contains("class=\"current\""));
  }

  #[test]
  fn test_render_menu_empty() {
    let nav = Container::new();
    let html = render_menu(&nav);
    assert_eq!(html, "<ul>\n</ul>");
  }

  #[test]
  fn test_render_menu_with_fragment() {
    let mut nav = Container::new();
    nav.add_page(
      Page::builder()
        .label("Section")
        .uri("/page")
        .fragment("section1")
        .build(),
    );
    let html = render_menu(&nav);
    assert!(html.contains("href=\"/page#section1\""));
  }

  #[test]
  fn test_render_menu_no_uri() {
    let mut nav = Container::new();
    nav.add_page(Page::builder().label("NoLink").build());
    let html = render_menu(&nav);
    assert!(html.contains("href=\"#\""));
  }

  #[test]
  fn test_render_menu_no_label() {
    let mut nav = Container::new();
    nav.add_page(Page::builder().uri("/").build());
    let html = render_menu(&nav);
    assert!(html.contains("<a href=\"/\"></a>"));
  }

  #[test]
  fn test_render_breadcrumbs() {
    let mut nav = Container::new();
    let mut products = Page::builder()
      .label("Products")
      .uri("/products")
      .build();
    products.add_page(
      Page::builder()
        .label("Books")
        .uri("/products/books")
        .active(true)
        .build(),
    );
    nav.add_page(products);

    let html = render_breadcrumbs(&nav, " &gt; ");
    assert!(html.contains("Products"));
    assert!(html.contains("Books"));
    assert!(html.contains(" &gt; "));
    assert!(html.contains("<span class=\"active\">Books</span>"));
    assert!(html.contains("href=\"/products\""));
  }

  #[test]
  fn test_render_breadcrumbs_empty() {
    let nav = Container::new();
    let html = render_breadcrumbs(&nav, " > ");
    assert!(html.is_empty());
  }

  #[test]
  fn test_render_breadcrumbs_no_active() {
    let mut nav = Container::new();
    nav.add_page(Page::builder().label("Home").uri("/").build());
    let html = render_breadcrumbs(&nav, " > ");
    assert!(html.is_empty());
  }

  #[test]
  fn test_render_breadcrumbs_single_active() {
    let mut nav = Container::new();
    nav.add_page(
      Page::builder()
        .label("Home")
        .uri("/")
        .active(true)
        .build(),
    );
    let html = render_breadcrumbs(&nav, " > ");
    assert_eq!(html, "<span class=\"active\">Home</span>");
  }

  #[test]
  fn test_render_sitemap() {
    let nav = sample_nav();
    let html = render_sitemap(&nav);

    assert!(html.contains("class=\"sitemap\""));
    assert!(html.contains("Home"));
    assert!(html.contains("Products"));
    assert!(html.contains("Books"));
    assert!(html.contains("Electronics"));
    assert!(html.contains("About"));
  }

  #[test]
  fn test_render_sitemap_hides_invisible() {
    let mut nav = Container::new();
    nav.add_page(Page::builder().label("V").uri("/v").build());
    nav.add_page(Page::builder().label("H").uri("/h").visible(false).build());

    let html = render_sitemap(&nav);
    assert!(html.contains("V"));
    assert!(!html.contains("H"));
  }

  #[test]
  fn test_render_sitemap_hierarchical() {
    let nav = sample_nav();
    let html = render_sitemap_hierarchical(&nav);

    assert!(html.contains("class=\"sitemap\""));
    assert!(html.contains("Products"));
    assert!(html.contains("Books"));
    // Should have nested <ul> for sub-pages
    let ul_count = html.matches("<ul").count();
    assert!(ul_count >= 2); // outer + products sub-list
  }

  #[test]
  fn test_render_sitemap_hierarchical_hides_invisible() {
    let mut nav = Container::new();
    let mut parent = Page::builder().label("P").uri("/p").build();
    parent.add_page(Page::builder().label("Hidden").uri("/h").visible(false).build());
    parent.add_page(Page::builder().label("Visible").uri("/v").build());
    nav.add_page(parent);

    let html = render_sitemap_hierarchical(&nav);
    assert!(html.contains("Visible"));
    assert!(!html.contains("Hidden"));
  }

  #[test]
  fn test_xss_prevention() {
    let mut nav = Container::new();
    nav.add_page(
      Page::builder()
        .label("<script>alert('xss')</script>")
        .uri("/test?a=1&b=2")
        .build(),
    );

    let html = render_menu(&nav);
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&amp;"));
  }

  #[test]
  fn test_render_menu_deep_nesting() {
    let mut nav = Container::new();
    let mut l1 = Page::builder().label("L1").uri("/l1").build();
    let mut l2 = Page::builder().label("L2").uri("/l2").build();
    l2.add_page(Page::builder().label("L3").uri("/l3").build());
    l1.add_page(l2);
    nav.add_page(l1);

    let html = render_menu(&nav);
    assert!(html.contains("L1"));
    assert!(html.contains("L2"));
    assert!(html.contains("L3"));
    // Should have 3 levels of <ul>
    let ul_count = html.matches("<ul>").count();
    assert_eq!(ul_count, 3);
  }
}

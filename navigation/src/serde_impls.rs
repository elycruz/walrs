#[cfg(any(feature = "json", feature = "yaml"))]
use crate::container::Container;
#[cfg(any(feature = "json", feature = "yaml"))]
use crate::error::{NavigationError, Result};
#[cfg(any(feature = "json", feature = "yaml"))]
use crate::page::Page;

/// Implements TryFrom for creating a Container from JSON string.
#[cfg(feature = "json")]
impl TryFrom<&str> for Container {
  type Error = NavigationError;

  fn try_from(json: &str) -> Result<Self> {
    serde_json::from_str::<Vec<Page>>(json)
      .map(Container::from_pages)
      .map_err(|e| NavigationError::DeserializationError(e.to_string()))
  }
}

/// Implements TryFrom for creating a Container from JSON bytes.
#[cfg(feature = "json")]
impl TryFrom<&[u8]> for Container {
  type Error = NavigationError;

  fn try_from(bytes: &[u8]) -> Result<Self> {
    serde_json::from_slice::<Vec<Page>>(bytes)
      .map(Container::from_pages)
      .map_err(|e| NavigationError::DeserializationError(e.to_string()))
  }
}

/// Implements TryFrom for creating a Page from JSON string.
#[cfg(feature = "json")]
impl TryFrom<&str> for Page {
  type Error = NavigationError;

  fn try_from(json: &str) -> Result<Self> {
    serde_json::from_str(json).map_err(|e| NavigationError::DeserializationError(e.to_string()))
  }
}

#[cfg(feature = "json")]
impl Container {
  /// Creates a Container from a JSON string.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_navigation::Container;
  ///
  /// let json = r#"[
  ///     {
  ///         "label": "Home",
  ///         "uri": "/"
  ///     },
  ///     {
  ///         "label": "About",
  ///         "uri": "/about"
  ///     }
  /// ]"#;
  ///
  /// let nav = Container::from_json(json).unwrap();
  /// assert_eq!(nav.count(), 2);
  /// ```
  pub fn from_json(json: &str) -> Result<Self> {
    Self::try_from(json)
  }

  /// Serializes the Container to JSON.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_navigation::{Container, Page};
  ///
  /// let mut nav = Container::new();
  /// nav.add_page(Page::builder().label("Home").uri("/").build());
  ///
  /// let json = nav.to_json().unwrap();
  /// assert!(json.contains("Home"));
  /// ```
  pub fn to_json(&self) -> Result<String> {
    serde_json::to_string(&self.pages())
      .map_err(|e| NavigationError::SerializationError(e.to_string()))
  }

  /// Serializes the Container to pretty JSON.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_navigation::{Container, Page};
  ///
  /// let mut nav = Container::new();
  /// nav.add_page(Page::builder().label("Home").uri("/").build());
  ///
  /// let json = nav.to_json_pretty().unwrap();
  /// assert!(json.contains("Home"));
  /// ```
  pub fn to_json_pretty(&self) -> Result<String> {
    serde_json::to_string_pretty(&self.pages())
      .map_err(|e| NavigationError::SerializationError(e.to_string()))
  }
}

#[cfg(feature = "yaml")]
impl Container {
  /// Creates a Container from a YAML string.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_navigation::Container;
  ///
  /// let yaml = r#"
  /// - label: Home
  ///   uri: /
  /// - label: About
  ///   uri: /about
  /// "#;
  ///
  /// let nav = Container::from_yaml(yaml).unwrap();
  /// assert_eq!(nav.count(), 2);
  /// ```
  pub fn from_yaml(yaml: &str) -> Result<Self> {
    serde_yaml::from_str::<Vec<Page>>(yaml)
      .map(Container::from_pages)
      .map_err(|e| NavigationError::DeserializationError(e.to_string()))
  }

  /// Serializes the Container to YAML.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_navigation::{Container, Page};
  ///
  /// let mut nav = Container::new();
  /// nav.add_page(Page::builder().label("Home").uri("/").build());
  ///
  /// let yaml = nav.to_yaml().unwrap();
  /// assert!(yaml.contains("Home"));
  /// ```
  pub fn to_yaml(&self) -> Result<String> {
    serde_yaml::to_string(&self.pages())
      .map_err(|e| NavigationError::SerializationError(e.to_string()))
  }
}

#[cfg(feature = "json")]
impl Page {
  /// Creates a Page from a JSON string.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_navigation::Page;
  ///
  /// let json = r#"{"label": "Home", "uri": "/"}"#;
  /// let page = Page::from_json(json).unwrap();
  /// assert_eq!(page.label.as_deref(), Some("Home"));
  /// ```
  pub fn from_json(json: &str) -> Result<Self> {
    Self::try_from(json)
  }

  /// Serializes the Page to JSON.
  pub fn to_json(&self) -> Result<String> {
    serde_json::to_string(self).map_err(|e| NavigationError::SerializationError(e.to_string()))
  }

  /// Serializes the Page to pretty JSON.
  pub fn to_json_pretty(&self) -> Result<String> {
    serde_json::to_string_pretty(self)
      .map_err(|e| NavigationError::SerializationError(e.to_string()))
  }
}

#[cfg(feature = "yaml")]
impl Page {
  /// Creates a Page from a YAML string.
  ///
  /// # Examples
  ///
  /// ```
  /// use walrs_navigation::Page;
  ///
  /// let yaml = "label: Home\nuri: /";
  /// let page = Page::from_yaml(yaml).unwrap();
  /// assert_eq!(page.label.as_deref(), Some("Home"));
  /// ```
  pub fn from_yaml(yaml: &str) -> Result<Self> {
    serde_yaml::from_str(yaml).map_err(|e| NavigationError::DeserializationError(e.to_string()))
  }

  /// Serializes the Page to YAML.
  pub fn to_yaml(&self) -> Result<String> {
    serde_yaml::to_string(self).map_err(|e| NavigationError::SerializationError(e.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use crate::container::Container;
  use crate::page::Page;

  #[test]
  #[cfg(feature = "json")]
  fn test_container_from_json() {
    let json = r#"[
            {
                "label": "Home",
                "uri": "/"
            },
            {
                "label": "About",
                "uri": "/about",
                "pages": [
                    {
                        "label": "Team",
                        "uri": "/about/team"
                    }
                ]
            }
        ]"#;

    let nav = Container::from_json(json).unwrap();
    assert_eq!(nav.count(), 3);
    assert_eq!(nav.pages().len(), 2);

    let about = nav.find_by_uri("/about").unwrap();
    assert_eq!(about.pages.len(), 1);
  }

  #[test]
  #[cfg(feature = "yaml")]
  fn test_container_from_yaml() {
    let yaml = r#"
- label: Home
  uri: /
- label: About
  uri: /about
  pages:
    - label: Team
      uri: /about/team
"#;

    let nav = Container::from_yaml(yaml).unwrap();
    assert_eq!(nav.count(), 3);
  }

  #[test]
  #[cfg(feature = "json")]
  fn test_page_from_json() {
    let json = r#"{"label": "Home", "uri": "/", "active": true}"#;
    let page = Page::from_json(json).unwrap();
    assert_eq!(page.label.as_deref(), Some("Home"));
    assert_eq!(page.uri.as_deref(), Some("/"));
    assert!(page.active);
  }

  #[test]
  #[cfg(feature = "yaml")]
  fn test_page_from_yaml() {
    let yaml = "label: Home\nuri: /\nactive: true";
    let page = Page::from_yaml(yaml).unwrap();
    assert_eq!(page.label.as_deref(), Some("Home"));
    assert!(page.active);
  }

  #[test]
  #[cfg(feature = "json")]
  fn test_container_to_json() {
    let mut nav = Container::new();
    nav.add_page(Page::builder().label("Home").uri("/").build());

    let json = nav.to_json().unwrap();
    assert!(json.contains("Home"));

    // Round trip
    let nav2 = Container::from_json(&json).unwrap();
    assert_eq!(nav2.count(), 1);
  }

  #[test]
  #[cfg(feature = "yaml")]
  fn test_container_to_yaml() {
    let mut nav = Container::new();
    nav.add_page(Page::builder().label("Home").uri("/").build());

    let yaml = nav.to_yaml().unwrap();
    assert!(yaml.contains("Home"));

    // Round trip
    let nav2 = Container::from_yaml(&yaml).unwrap();
    assert_eq!(nav2.count(), 1);
  }

  #[test]
  #[cfg(feature = "json")]
  fn test_page_to_json() {
    let page = Page::builder().label("Home").uri("/").build();
    let json = page.to_json().unwrap();
    assert!(json.contains("Home"));

    // Round trip
    let page2 = Page::from_json(&json).unwrap();
    assert_eq!(page2.label.as_deref(), Some("Home"));
  }

  #[test]
  #[cfg(feature = "json")]
  fn test_invalid_json() {
    let invalid_json = "not valid json";
    let result = Container::from_json(invalid_json);
    assert!(result.is_err());
  }

  #[test]
  #[cfg(feature = "yaml")]
  fn test_invalid_yaml() {
    let invalid_yaml = "- invalid\n  - bad: [unclosed";
    let result = Container::from_yaml(invalid_yaml);
    assert!(result.is_err());
  }

  #[test]
  #[cfg(feature = "json")]
  fn test_try_from_str() {
    let json = r#"[{"label": "Home", "uri": "/"}]"#;
    let nav = Container::try_from(json).unwrap();
    assert_eq!(nav.count(), 1);
  }

  #[test]
  #[cfg(feature = "json")]
  fn test_try_from_bytes() {
    let json = br#"[{"label": "Home", "uri": "/"}]"#;
    let nav = Container::try_from(&json[..]).unwrap();
    assert_eq!(nav.count(), 1);
  }

  #[test]
  #[cfg(feature = "json")]
  fn test_container_to_json_pretty() {
    let mut nav = Container::new();
    nav.add_page(Page::builder().label("Home").uri("/").build());

    let json = nav.to_json_pretty().unwrap();
    assert!(json.contains("Home"));
    // Pretty JSON should contain newlines and indentation
    assert!(json.contains('\n'));

    // Round trip
    let nav2 = Container::from_json(&json).unwrap();
    assert_eq!(nav2.count(), 1);
  }

  #[test]
  #[cfg(feature = "json")]
  fn test_page_to_json_pretty() {
    let page = Page::builder().label("Home").uri("/").build();
    let json = page.to_json_pretty().unwrap();
    assert!(json.contains("Home"));
    // Pretty JSON should contain newlines and indentation
    assert!(json.contains('\n'));

    // Round trip
    let page2 = Page::from_json(&json).unwrap();
    assert_eq!(page2.label.as_deref(), Some("Home"));
  }
}

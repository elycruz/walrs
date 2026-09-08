use std::fmt::Debug;

/// Simple trait to contain shared definitions for Digraph*DFS structs.
pub trait DigraphDFSShape {
  /// Returns a `Result` indicating whether  a path from 'source vertex' to 'i' exists.
  fn marked(&self, i: usize) -> Result<bool, String>;
}

/// A vertex payload for symbol graphs (`DisymGraph`, and `walrs_graph::SymbolGraph`).
///
/// `id()` is the key used to deduplicate vertices and to look them up by name;
/// the rest of the payload is opaque to the graph.
///
/// ```rust
/// use walrs_digraph::Symbol;
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct Role { name: String, level: u8 }
///
/// impl Symbol for Role {
///   fn id(&self) -> &str { &self.name }
/// }
///
/// let r = Role { name: "admin".into(), level: 9 };
/// assert_eq!(r.id(), "admin");
/// assert_eq!("guest".to_string().id(), "guest");
/// ```
pub trait Symbol: Clone + Debug + PartialEq {
  /// Returns the symbol's unique id/name.
  fn id(&self) -> &str;
}

impl Symbol for String {
  fn id(&self) -> &str {
    self.as_str()
  }
}

#[cfg(test)]
mod test {
  use super::Symbol;

  #[test]
  fn test_string_symbol_id() {
    let s = "hello".to_string();
    assert_eq!(s.id(), "hello");
    assert!(std::ptr::eq(s.id().as_ptr(), s.as_str().as_ptr()));
  }
}

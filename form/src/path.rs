//! Path parsing for FormData.
use thiserror::Error;
/// Path segment types.
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
  Field(String),
  Index(usize),
}
/// Path parsing errors.
#[derive(Debug, Error)]
pub enum PathError {
  #[error("Invalid syntax: {0}")]
  InvalidSyntax(String),
  #[error("Invalid index: {0}")]
  InvalidIndex(String),
}
/// Parse a path string into segments.
pub fn parse_path(path: &str) -> Result<Vec<PathSegment>, PathError> {
  if path.is_empty() {
    return Ok(Vec::new());
  }
  let mut segments = Vec::new();
  let mut current = String::new();
  let mut chars = path.chars().peekable();
  while let Some(c) = chars.next() {
    match c {
      '.' => {
        if !current.is_empty() {
          segments.push(PathSegment::Field(current.clone()));
          current.clear();
        }
      }
      '[' => {
        if !current.is_empty() {
          segments.push(PathSegment::Field(current.clone()));
          current.clear();
        }
        let mut index_str = String::new();
        for ic in chars.by_ref() {
          if ic == ']' {
            break;
          }
          index_str.push(ic);
        }
        let index: usize = index_str
          .parse()
          .map_err(|_| PathError::InvalidIndex(index_str.clone()))?;
        segments.push(PathSegment::Index(index));
      }
      ']' => {
        return Err(PathError::InvalidSyntax("Unexpected ']'".to_string()));
      }
      _ => {
        current.push(c);
      }
    }
  }
  if !current.is_empty() {
    segments.push(PathSegment::Field(current));
  }
  Ok(segments)
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_simple_field() {
    let segments = parse_path("email").unwrap();
    assert_eq!(segments, vec![PathSegment::Field("email".to_string())]);
  }
  #[test]
  fn test_dot_notation() {
    let segments = parse_path("user.email").unwrap();
    assert_eq!(
      segments,
      vec![
        PathSegment::Field("user".to_string()),
        PathSegment::Field("email".to_string()),
      ]
    );
  }
  #[test]
  fn test_array_indexing() {
    let segments = parse_path("items[0]").unwrap();
    assert_eq!(
      segments,
      vec![
        PathSegment::Field("items".to_string()),
        PathSegment::Index(0),
      ]
    );
  }
}

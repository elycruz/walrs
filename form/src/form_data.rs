//! Form data transfer object.
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::HashMap;
use walrs_form_core::Value;
use crate::path::{parse_path, PathSegment};
/// Form data transfer object.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FormData(HashMap<String, Value>);
impl FormData {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn insert<K: Into<String>>(&mut self, key: K, value: Value) -> &mut Self {
        self.0.insert(key.into(), value);
        self
    }
    pub fn get_direct(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }
    pub fn get(&self, path: &str) -> Option<&Value> {
        let segments = parse_path(path).ok()?;
        if segments.is_empty() {
            return None;
        }
        let mut current: Option<&Value> = None;
        for (i, segment) in segments.iter().enumerate() {
            match segment {
                PathSegment::Field(name) => {
                    if i == 0 {
                        current = self.0.get(name);
                    } else {
                        current = current?.as_object()?.get(name);
                    }
                }
                PathSegment::Index(idx) => {
                    current = current?.as_array()?.get(*idx);
                }
            }
        }
        current
    }
    pub fn set(&mut self, path: &str, value: Value) -> &mut Self {
        let segments = match parse_path(path) {
            Ok(s) if !s.is_empty() => s,
            _ => return self,
        };
        if segments.len() == 1 {
            if let PathSegment::Field(name) = &segments[0] {
                self.0.insert(name.clone(), value);
            }
            return self;
        }
        let first = match &segments[0] {
            PathSegment::Field(name) => name.clone(),
            _ => return self,
        };
        let root = self.0.entry(first).or_insert_with(|| {
            match segments.get(1) {
                Some(PathSegment::Index(_)) => Value::Array(Vec::new()),
                _ => Value::Object(Map::new()),
            }
        });
        set_nested(root, &segments[1..], value);
        self
    }
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.remove(key)
    }
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0.iter()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn into_inner(self) -> HashMap<String, Value> {
        self.0
    }
    pub fn as_inner(&self) -> &HashMap<String, Value> {
        &self.0
    }
}
fn set_nested(current: &mut Value, segments: &[PathSegment], value: Value) {
    if segments.is_empty() {
        *current = value;
        return;
    }
    match &segments[0] {
        PathSegment::Field(name) => {
            if !current.is_object() {
                *current = Value::Object(Map::new());
            }
            let obj = current.as_object_mut().unwrap();
            if segments.len() == 1 {
                obj.insert(name.clone(), value);
            } else {
                let next = obj.entry(name.clone()).or_insert_with(|| {
                    match segments.get(1) {
                        Some(PathSegment::Index(_)) => Value::Array(Vec::new()),
                        _ => Value::Object(Map::new()),
                    }
                });
                set_nested(next, &segments[1..], value);
            }
        }
        PathSegment::Index(idx) => {
            if !current.is_array() {
                *current = Value::Array(Vec::new());
            }
            let arr = current.as_array_mut().unwrap();
            while arr.len() <= *idx {
                arr.push(Value::Null);
            }
            if segments.len() == 1 {
                arr[*idx] = value;
            } else {
                set_nested(&mut arr[*idx], &segments[1..], value);
            }
        }
    }
}
impl From<HashMap<String, Value>> for FormData {
    fn from(map: HashMap<String, Value>) -> Self {
        Self(map)
    }
}
impl From<serde_json::Value> for FormData {
    fn from(value: serde_json::Value) -> Self {
        if let Value::Object(map) = value {
            let converted: HashMap<String, Value> = map.into_iter().collect();
            Self(converted)
        } else {
            Self::new()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_simple_insert_and_get() {
        let mut data = FormData::new();
        data.insert("email", json!("test@example.com"));
        assert_eq!(data.get("email").unwrap().as_str(), Some("test@example.com"));
    }
    #[test]
    fn test_dot_notation_get() {
        let mut data = FormData::new();
        data.insert("user", json!({"email": "test@example.com"}));
        assert_eq!(
            data.get("user.email").unwrap().as_str(),
            Some("test@example.com")
        );
    }
}

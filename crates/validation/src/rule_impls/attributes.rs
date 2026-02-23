#![cfg(feature = "serde_json_bridge")]

use serde::Serialize;
use serde_json::value::to_value as to_json_value;

use crate::rule::Rule;
use crate::traits::ToAttributesList;

// ============================================================================
// ToAttributesList Implementation for Rule
// ============================================================================

impl<T: Serialize> ToAttributesList for Rule<T> {
  /// Converts rule variants to HTML attribute key-value pairs.
  ///
  /// Returns a list of attribute name/value pairs suitable for HTML form elements.
  /// Composite rules (`All`/`Any`) flatten their children's attributes.
  /// Non-attribute-mappable variants return `None`.
  ///
  /// # HTML Attribute Mappings
  ///
  /// | Rule Variant | HTML Attribute(s) |
  /// |--------------|-------------------|
  /// | `Required` | `required=true` |
  /// | `MinLength(n)` | `minlength=n` |
  /// | `MaxLength(n)` | `maxlength=n` |
  /// | `ExactLength(n)` | `minlength=n`, `maxlength=n` |
  /// | `Pattern(p)` | `pattern=p` |
  /// | `Email` | `type=email` |
  /// | `Url` | `type=url` |
  /// | `Min(v)` | `min=v` |
  /// | `Max(v)` | `max=v` |
  /// | `Range { min, max }` | `min=min`, `max=max` |
  /// | `Step(v)` | `step=v` |
  /// | `All(rules)` | Flattened child attributes |
  /// | `Any(rules)` | Flattened child attributes |
  /// | Other variants | `None` |
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validation::{Rule, ToAttributesList};
  ///
  /// let rule = Rule::<String>::MinLength(3);
  /// let attrs = rule.to_attributes_list().unwrap();
  /// assert_eq!(attrs.len(), 1);
  /// assert_eq!(attrs[0].0, "minlength");
  /// assert_eq!(attrs[0].1, serde_json::json!(3));
  ///
  /// // Composite rules flatten attributes
  /// let composite = Rule::<String>::MinLength(3).and(Rule::MaxLength(50));
  /// let attrs = composite.to_attributes_list().unwrap();
  /// assert_eq!(attrs.len(), 2);
  /// ```
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    match self {
      // Presence
      Rule::Required => Some(vec![(
        "required".to_string(),
        serde_json::Value::Bool(true),
      )]),

      // Length Rules
      Rule::MinLength(n) => Some(vec![("minlength".to_string(), serde_json::Value::from(*n))]),
      Rule::MaxLength(n) => Some(vec![("maxlength".to_string(), serde_json::Value::from(*n))]),
      Rule::ExactLength(n) => Some(vec![
        ("minlength".to_string(), serde_json::Value::from(*n)),
        ("maxlength".to_string(), serde_json::Value::from(*n)),
      ]),

      // String Rules
      Rule::Pattern(p) => Some(vec![(
        "pattern".to_string(),
        serde_json::Value::from(p.clone()),
      )]),
      Rule::Email(_) => Some(vec![("type".to_string(), serde_json::Value::from("email"))]),
      Rule::Url(_) => Some(vec![("type".to_string(), serde_json::Value::from("url"))]),

      // Numeric Rules
      Rule::Min(v) => to_json_value(v)
        .ok()
        .map(|val| vec![("min".to_string(), val)]),
      Rule::Max(v) => to_json_value(v)
        .ok()
        .map(|val| vec![("max".to_string(), val)]),
      Rule::Range { min, max } => {
        let min_val = to_json_value(min).ok()?;
        let max_val = to_json_value(max).ok()?;
        Some(vec![
          ("min".to_string(), min_val),
          ("max".to_string(), max_val),
        ])
      }
      Rule::Step(v) => to_json_value(v)
        .ok()
        .map(|val| vec![("step".to_string(), val)]),

      // Comparison - no direct HTML attribute equivalent
      Rule::Equals(_) => None,
      Rule::OneOf(_) => None,

      // Composite Rules - flatten children
      Rule::All(rules) => {
        let mut attrs = Vec::new();
        for rule in rules {
          if let Some(child_attrs) = rule.to_attributes_list() {
            attrs.extend(child_attrs);
          }
        }
        if attrs.is_empty() { None } else { Some(attrs) }
      }
      Rule::Any(rules) => {
        let mut attrs = Vec::new();
        for rule in rules {
          if let Some(child_attrs) = rule.to_attributes_list() {
            attrs.extend(child_attrs);
          }
        }
        if attrs.is_empty() { None } else { Some(attrs) }
      }

      // Not - negation doesn't map to HTML attributes
      Rule::Not(_) => None,

      // Conditional - doesn't map to HTML attributes
      Rule::When { .. } => None,

      // Custom/Runtime variants - not attribute-mappable
      Rule::Custom(_) => None,
      Rule::Ref(_) => None,

      // Uri/Ip - no HTML attribute equivalent
      Rule::Uri(_) => None,
      Rule::Ip(_) => None,
      Rule::Hostname(_) => None,
      Rule::Date(_) => None,
      Rule::DateRange(_) => None,

      // WithMessage - delegate to inner rule
      Rule::WithMessage { rule, .. } => rule.to_attributes_list(),
    }
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use crate::rule::{Condition, Rule};
  use crate::message::Message;
  use crate::traits::ToAttributesList;

  #[test]
  fn test_to_attributes_list_required() {
    let rule = Rule::<String>::Required;
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "required");
    assert_eq!(attrs[0].1, serde_json::Value::Bool(true));
  }

  #[test]
  fn test_to_attributes_list_min_length() {
    let rule = Rule::<String>::MinLength(3);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "minlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(3));
  }

  #[test]
  fn test_to_attributes_list_max_length() {
    let rule = Rule::<String>::MaxLength(50);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "maxlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(50));
  }

  #[test]
  fn test_to_attributes_list_exact_length() {
    let rule = Rule::<String>::ExactLength(10);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].0, "minlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(10));
    assert_eq!(attrs[1].0, "maxlength");
    assert_eq!(attrs[1].1, serde_json::Value::from(10));
  }

  #[test]
  fn test_to_attributes_list_pattern() {
    let rule = Rule::<String>::Pattern(r"^\w+$".to_string());
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "pattern");
    assert_eq!(attrs[0].1, serde_json::Value::from(r"^\w+$"));
  }

  #[test]
  fn test_to_attributes_list_email() {
    let rule = Rule::<String>::Email(Default::default());
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "type");
    assert_eq!(attrs[0].1, serde_json::Value::from("email"));
  }

  #[test]
  fn test_to_attributes_list_url() {
    let rule = Rule::<String>::Url(Default::default());
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "type");
    assert_eq!(attrs[0].1, serde_json::Value::from("url"));
  }

  #[test]
  fn test_to_attributes_list_min() {
    let rule = Rule::<i32>::Min(0);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "min");
    assert_eq!(attrs[0].1, serde_json::Value::from(0));
  }

  #[test]
  fn test_to_attributes_list_max() {
    let rule = Rule::<i32>::Max(100);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "max");
    assert_eq!(attrs[0].1, serde_json::Value::from(100));
  }

  #[test]
  fn test_to_attributes_list_range() {
    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].0, "min");
    assert_eq!(attrs[0].1, serde_json::Value::from(0));
    assert_eq!(attrs[1].0, "max");
    assert_eq!(attrs[1].1, serde_json::Value::from(100));
  }

  #[test]
  fn test_to_attributes_list_step() {
    let rule = Rule::<i32>::Step(5);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "step");
    assert_eq!(attrs[0].1, serde_json::Value::from(5));
  }

  #[test]
  fn test_to_attributes_list_all_composite() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(50));
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].0, "minlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(3));
    assert_eq!(attrs[1].0, "maxlength");
    assert_eq!(attrs[1].1, serde_json::Value::from(50));
  }

  #[test]
  fn test_to_attributes_list_any_composite() {
    let rule = Rule::<String>::Email(Default::default()).or(Rule::Url(Default::default()));
    let attrs = rule.to_attributes_list().unwrap();
    assert!(attrs.iter().any(|(k, v)| k == "type" && v == "email"));
    assert!(attrs.iter().any(|(k, v)| k == "type" && v == "url"));
  }

  #[test]
  fn test_to_attributes_list_complex_composite() {
    let rule = Rule::<String>::Required
      .and(Rule::MinLength(5))
      .and(Rule::MaxLength(100))
      .and(Rule::Pattern(r"^\w+$".to_string()));
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 4);
    assert!(attrs.iter().any(|(k, _)| k == "required"));
    assert!(attrs.iter().any(|(k, _)| k == "minlength"));
    assert!(attrs.iter().any(|(k, _)| k == "maxlength"));
    assert!(attrs.iter().any(|(k, _)| k == "pattern"));
  }

  #[test]
  fn test_to_attributes_list_non_mappable_returns_none() {
    let rule = Rule::<String>::Equals("test".to_string());
    assert!(rule.to_attributes_list().is_none());

    let rule = Rule::<String>::OneOf(vec!["a".to_string(), "b".to_string()]);
    assert!(rule.to_attributes_list().is_none());

    let rule = Rule::<String>::MinLength(3).not();
    assert!(rule.to_attributes_list().is_none());

    let rule = Rule::<String>::When {
      condition: Condition::IsNotEmpty,
      then_rule: Box::new(Rule::MinLength(3)),
      else_rule: None,
    };
    assert!(rule.to_attributes_list().is_none());
  }

  #[test]
  fn test_to_attributes_list_with_message_delegates() {
    let inner_rule = Rule::<String>::MinLength(5);
    let rule = Rule::WithMessage {
      rule: Box::new(inner_rule),
      message: Message::Static("Custom message.".to_string()),
      locale: None,
    };
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "minlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(5));
  }

  #[test]
  fn test_to_attributes_list_empty_all_returns_none() {
    let rule = Rule::<String>::All(vec![Rule::Equals("test".to_string())]);
    assert!(rule.to_attributes_list().is_none());
  }

  #[test]
  fn test_to_attributes_list_numeric_types() {
    let rule = Rule::<f64>::Min(0.5);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs[0].0, "min");
    assert_eq!(attrs[0].1, serde_json::Value::from(0.5));

    let rule = Rule::<f64>::Range { min: 0.0, max: 1.0 };
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].1, serde_json::Value::from(0.0));
    assert_eq!(attrs[1].1, serde_json::Value::from(1.0));

    let rule = Rule::<f64>::Step(0.1);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs[0].0, "step");
    assert_eq!(attrs[0].1, serde_json::Value::from(0.1));
  }
}


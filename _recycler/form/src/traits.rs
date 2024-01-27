use std::borrow::Cow;
use serde_json::Map;
use crate::walrs_inputfilter::types::{InputConstraints, InputValue};

pub trait FormControl<'a, 'b, Value: 'b, Constraints: 'a>
  where
    Value: 'b + InputValue,
    Constraints: 'a + InputConstraints<'a, 'b, Value>,
{
  /// Returns the control's validation constraints struct.
  fn get_constraints(&self) -> Option<&Constraints>;

  /// Returns an optional ref to the control's validation message.
  fn get_validation_message(&self) -> Option<Cow<'a, str>>;

  /// Sets validation message.
  fn set_validation_message(&mut self, msg: Option<String>);

  /// Validate this control against it's validation constraints.
  fn validate(&mut self, value: Option<&'b Value>) -> Result<(), String> {
    match self.get_constraints() {
      Some(constraints) => match constraints.validate(value) {
        Ok(()) => Ok(()),
        Err(msgs) => Err(msgs[0].1.to_string()),
      },
      _ => Ok(()),
    }
  }

  /// Runs control's validation, stores the result, and returns a bool indicating whether control's
  /// `value`/control's validity itself, is valid or not.
  fn check_validity(&mut self) -> bool {
    let rslt = match self.validate(self.get_value().as_deref()) {
      Ok(()) => None,
      Err(err) => Some(err),
    };
    let out = rslt.is_none();
    self.set_validation_message(rslt);
    out
  }

  /// Gets control's `value`.
  fn get_value<'c: 'b>(&self) -> Option<Cow<'c, Value>>;

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not, and receiving `bool` signaling
  /// control's validity.`
  fn set_value(&mut self, value: Option<Value>) -> bool;

  /// Used from controls that can contain multiple values, radio button groups, select elements, etc..
  fn set_values(&mut self, _: Option<&[Value]>) -> bool {
    self.check_validity()
  }

  /// Returns attributes map.
  fn get_attributes(&self) -> Option<&Map<String, serde_json::Value>>;

  /// Returns mutable version of attributes map.
  fn get_attributes_mut(&mut self) -> Option<&mut Map<String, serde_json::Value>>;

  /// Sets attributes map.
  fn set_attributes(&mut self, attributes: Option<Map<String, serde_json::Value>>);

  /// Populates internal html attribute cache.
  fn set_attribute(&mut self, key: &str, value: serde_json::Value) {
    if let Some(map) = self.get_attributes_mut() {
        map.insert(key.into(), value);
    }
  }

  /// Removes html attribute entry, in html attrib. cache.
  fn remove_attribute(&mut self, key: &str) -> Option<serde_json::Value> {
    if let Some(map) = self.get_attributes_mut() {
      return map.remove(key);
    }
    None
  }

  /// Returns a boolean indicating whether attribute exists in html attrib. cache or not.
  fn has_attribute(&mut self, key: &str) -> bool {
    self.get_attributes().map_or(false, |attribs| {
      attribs.contains_key(key)
    })
  }
}

pub trait WithName<'a> {
  fn get_name(&self) -> Option<Cow<'a, str>>;
}

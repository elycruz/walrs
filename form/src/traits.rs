use std::borrow::Cow;
use std::collections::HashMap;
use crate::walrs_inputfilter::types::{InputConstraints, InputValue};

pub trait FormControl<'a, Value, Constraints>
  where
    Value: 'a + InputValue,
    Constraints: 'a + InputConstraints<Value>,
{
  /// Returns the control's validation constraints struct.
  fn get_constraints(&self) -> Option<&Constraints>;

  /// Returns an optional ref to the control's validation message.
  fn get_validation_message(&self) -> Option<Cow<'a, str>>;

  /// Sets validation message.
  fn set_validation_message(&mut self, msg: Option<String>);

  /// Validate this control against it's validation constraints.
  fn validate(&mut self, value: Option<&'a Value>) -> Result<(), String> {
    match self.get_constraints() {
      Some(constraints) => match constraints.validate(value) {
        Ok(()) => Ok(()),
        Err((_, msg)) => Err(msg.into()),
      },
      _ => Ok(()),
    }
  }

  /// Runs control's validation, stores the result, and returns a bool indicating whether control's
  /// `value`/control's validity itself, is valid or not.
  fn check_validity(&mut self) -> bool {
    let rslt = match self.validate(self.get_value().map(|x| x.as_ref())) {
      Ok(()) => None,
      Err(err) => Some(err),
    };
    let out = rslt.is_none();
    self.set_validation_message(rslt);
    out
  }

  /// Gets control's `value`.
  fn get_value(&self) -> Option<Cow<'a, Value>>;

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not, and receiving `bool` signaling
  /// control's validity.`
  fn set_value(&mut self, value: Option<Value>) -> bool;

  /// Used from controls that can contain multiple values, radio button groups, select elements, etc..
  fn set_values(&mut self, _: Option<&[Value]>) -> bool {
    self.check_validity()
  }

  fn get_attributes(&self) -> Option<&HashMap<&'a str,  Option<&'a str>>>;

  fn set_attributes(&mut self, attributes: Option<HashMap<&'a str, Option<&'a str>>>);

  /// Populates internal html attribute cache.
  fn set_attribute(&mut self, key: &str, value: &str) {
    self.set_attributes(
      self.get_attributes().map_or_else(
        || Some(HashMap::new()),
        |attribs| {
          let mut out = attribs.clone();
          out.insert(key, Some(value));
          Some(attribs.to_owned())
        })
    );
  }

  /// Removes html attribute entry, in html attrib. cache.
  fn remove_attribute(&mut self, key: &str) -> Option<String> {
    match self.get_attributes()
      .map(|attribs|{
        let mut out = attribs.clone();

        (out.remove(key).flatten().map(|v| v.to_string()), out)
      }) {
      Some((removed_value, new_attribs)) => {
        self.set_attributes(Some(new_attribs));

        removed_value
      },
      _ => None
    }
  }

  /// Returns a boolean indicating whether attribute exists in html attrib. cache or not.
  fn has_attribute(&mut self, key: &str) -> bool {
    self.get_attributes().map_or(false, |attribs| {
      attribs.contains_key(key)
    })
  }
}

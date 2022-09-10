use ecms_inputvalidator::types::{InputConstraints, ValidationResultError};
use std::fmt::{Debug, Display};
use std::string::String;
use ecms_inputvalidator::number::NumberInputConstraints;
use ecms_inputvalidator::text::TextInputConstraints;

pub trait FormControlValue: Clone + Debug + Display + PartialEq {}

impl FormControlValue for bool {}
impl FormControlValue for usize {}
impl FormControlValue for isize {}
impl FormControlValue for f32 {}
impl FormControlValue for f64 {}
impl FormControlValue for &'_ char {}
impl FormControlValue for &'_ str {}
impl FormControlValue for String {}

pub trait FormControlConstraints<Value: FormControlValue>: InputConstraints<Value> {}

impl FormControlConstraints<usize> for NumberInputConstraints<'_, usize> {}
impl FormControlConstraints<&'_ char> for TextInputConstraints<'_> {}
impl FormControlConstraints<&'_ str> for TextInputConstraints<'_> {}
impl FormControlConstraints<String> for TextInputConstraints<'_> {}

pub trait FormControl<'a, Value, ValueConstraints>
where
  Value: 'a + FormControlValue,
  ValueConstraints: 'a + FormControlConstraints<Value>,
{
  /// Returns the control's validation constraints struct.
  fn get_constraints(&self) -> Option<&ValueConstraints>;

  /// Sets validation message.
  fn set_validation_message(&mut self, msg: Option<String>);

  /// Validate this control against it's validation constraints.
  fn validate(&mut self, value: Option<Value>) -> Result<(), String> {
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
    let rslt = match self.validate(self.get_value()) {
      Ok(()) => None,
      Err(err) => Some(err),
    };
    let out = rslt.is_none();
    self.set_validation_message(rslt);
    out
  }

  /// Gets control's `value`.
  fn get_value(&self) -> Option<Value>;

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not, and receiving `bool` signaling
  /// control's validity.`
  fn set_value(&mut self, value: Option<Value>) -> bool;

  /// Used from controls that can contain multiple values, radio button groups, select elements, etc..
  fn set_values(&mut self, values: Option<&[Value]>) -> bool {
    self.check_validity()
  }
}

use ecms_inputconstraints::number::NumberConstraints;
use ecms_inputconstraints::text::TextConstraints;
use ecms_inputconstraints::types::{InputConstraints, ValidationResultTuple};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct PhantomValue {}

impl Display for PhantomValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "do not instantiate - struct is meant to be used as a `PhantomData` type"
    )
  }
}

pub trait FormControlValue: Clone + Debug + Display + PartialEq {}

impl FormControlValue for i64 {}
impl FormControlValue for i128 {}
impl FormControlValue for isize {}
impl FormControlValue for u64 {}
impl FormControlValue for u128 {}
impl FormControlValue for usize {}
impl FormControlValue for f64 {}
impl FormControlValue for &'_ str {}
impl FormControlValue for String {}
impl FormControlValue for PhantomValue {}

#[derive(Clone, Debug, Default)]
pub struct PhantomConstraints<'a> {
  phantom_field: PhantomData<&'a str>,
}

impl<'a> InputConstraints<PhantomValue> for PhantomConstraints<'a> {
  fn validate(&self, _: Option<PhantomValue>) -> Result<(), ValidationResultTuple> {
    Ok(())
  }
}

pub trait FormControlConstraints<Value: FormControlValue>: InputConstraints<Value> {}

impl FormControlConstraints<i64> for NumberConstraints<'_, i64> {}
impl FormControlConstraints<i128> for NumberConstraints<'_, i128> {}
impl FormControlConstraints<u64> for NumberConstraints<'_, u64> {}
impl FormControlConstraints<u128> for NumberConstraints<'_, u128> {}
impl FormControlConstraints<f64> for NumberConstraints<'_, f64> {}
impl FormControlConstraints<&'_ str> for TextConstraints<'_> {}
impl FormControlConstraints<String> for TextConstraints<'_> {}
impl FormControlConstraints<PhantomValue> for PhantomConstraints<'_> {}

pub trait FormControl<'a, Value, ValueConstraints>
where
  Value: 'a + FormControlValue,
  ValueConstraints: 'a + FormControlConstraints<Value>,
{
  /// Returns the control's validation constraints struct.
  fn get_constraints(&self) -> Option<&ValueConstraints>;

  /// Returns an optional ref to the control's validation message.
  fn get_validation_message(&self) -> Option<String>;

  /// Sets validation message.
  fn set_validation_message(&mut self, msg: Option<String>);

  /// Validate this control against it's validation constraints.
  fn validate(&mut self, value: Option<Cow<'a, Value>>) -> Result<(), String> {
    match self.get_constraints() {
      Some(constraints) => match constraints.validate(value.as_deref().map(|x| x.to_owned())) {
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
  fn get_value(&self) -> Option<Cow<'a, Value>>;

  /// Convenience setter for setting `value`, calling `check_validity()`, which updates
  /// `validation_message` based on whether `value` is valid or not, and receiving `bool` signaling
  /// control's validity.`
  fn set_value(&mut self, value: Option<Value>) -> bool;

  /// Used from controls that can contain multiple values, radio button groups, select elements, etc..
  fn set_values(&mut self, _: Option<&[Value]>) -> bool {
    self.check_validity()
  }

  fn get_html_attribs(&self) -> Option<&HashMap<&'a str, Option<&'a str>>>;

  fn set_html_attribs(&mut self, html_attribs: Option<HashMap<&'a str, Option<&'a str>>>);

  /// Populates internal html attribute cache.
  fn set_attribute(&mut self, key: &str, value: &str) {
    self.set_html_attribs(self.get_html_attribs().map_or_else(
      || Some(HashMap::new()),
      |attribs| {
        let mut out = attribs.clone();
        out.insert(key, Some(value));
        Some(attribs.to_owned())
      },
    ));
  }

  /// Removes html attribute entry, in html attrib. cache.
  fn remove_attribute(&mut self, key: &str) -> Option<String> {
    match self.get_html_attribs().map(|attribs| {
      let mut out = attribs.clone();

      (out.remove(key).flatten().map(|v| v.to_string()), out)
    }) {
      Some((removed_value, new_attribs)) => {
        self.set_html_attribs(Some(new_attribs));

        removed_value
      }
      _ => None,
    }
  }

  /// Returns a boolean indicating whether attribute exists in html attrib. cache or not.
  fn has_attribute(&mut self, key: &str) -> bool {
    self
      .get_html_attribs()
      .map_or(false, |attribs| attribs.contains_key(key))
  }
}

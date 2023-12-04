use std::ops::{Add, Div, Mul, Rem, Sub};
use std::borrow::Cow;
use std::fmt::{Debug, Display};
use serde::Serialize;

pub trait InputValue: ToOwned + Debug + Display + PartialEq + PartialOrd + Serialize {}

impl InputValue for i8 {}
impl InputValue for i16 {}
impl InputValue for i32 {}
impl InputValue for i64 {}
impl InputValue for i128 {}
impl InputValue for isize {}

impl InputValue for u8 {}
impl InputValue for u16 {}
impl InputValue for u32 {}
impl InputValue for u64 {}
impl InputValue for u128 {}
impl InputValue for usize {}

impl InputValue for f32 {}
impl InputValue for f64 {}

impl InputValue for bool {}

impl InputValue for char {}
impl InputValue for str {}

impl InputValue for &str {}

pub trait NumberValue: Default + InputValue + Copy + Add + Sub + Mul + Div + Rem<Output = Self> {}

impl NumberValue for i8 {}
impl NumberValue for i16 {}
impl NumberValue for i32 {}
impl NumberValue for i64 {}
impl NumberValue for i128 {}
impl NumberValue for isize {}

impl NumberValue for u8 {}
impl NumberValue for u16 {}
impl NumberValue for u32 {}
impl NumberValue for u64 {}
impl NumberValue for u128 {}
impl NumberValue for usize {}

impl NumberValue for f32 {}
impl NumberValue for f64 {}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ConstraintViolation {
  CustomError,
  PatternMismatch,
  RangeOverflow,
  RangeUnderflow,
  StepMismatch,
  TooLong,
  TooShort,
  NotEqual,
  TypeMismatch,
  ValueMissing,
}

pub type ViolationMessage = String;

pub type ValidationError = (ConstraintViolation, ViolationMessage);

pub type ValidationResult = Result<(), Vec<ValidationError>>;

pub type Filter<T> = dyn Fn(Option<T>) -> Option<T> + Send + Sync;

pub type Validator<T> = dyn Fn(T) -> ValidationResult + Send + Sync;

pub trait ValidateValue<T: InputValue> {
  /// @todo Should accept `&T`, or `Cow<T>`, here, instead of `T` (will allow overall types
  ///   to work with (seamlessly) with unsized (`?Sized`) types.
  fn validate(&self, value: T) -> ValidationResult;
}

pub trait FilterValue<T: InputValue> {
  fn filter(&self, value: Option<Cow<T>>) -> Option<Cow<T>>;
}

pub trait ToAttributesList {
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    None
  }
}

pub trait InputConstraints<'a, 'call_ctx, T: 'call_ctx>: Display + Debug
  where T: ToOwned + Debug + Display + PartialEq + PartialOrd + Serialize {

  fn get_should_break_on_failure(&self) -> bool;
  fn get_required(&self) -> bool;
  fn get_name(&self) -> Option<Cow<'a, str>>;
  fn get_value_missing_handler(&self) -> &'a (dyn Fn(&Self, Option<&T>) -> ViolationMessage + Send + Sync);
  fn get_validators(&self) -> Option<&[&'a Validator<Self::ValidatorT>]>;
  fn get_filters(&self) -> Option<&[&'a Filter<Self::FilterT>]>;

  /// Validates value using implementing structs own custom validation logic (e.g., using it's own "custom" properties etc.).
  /// Note: Gets called in `InputConstraints::validate` method, before any set validators are run.
  ///
  /// ```rust
  /// use walrs_inputfilter::*;
  ///
  /// let input = StringInputBuilder::default()
  ///   .required(true)
  ///   .value_missing(&|_, _| "Value missing".to_string())
  ///   .min_length(3usize)
  ///   .too_short(&|_, _| "Too short".to_string())
  ///   .max_length(55usize)
  ///   .too_long(&|_, _| "Too long".to_string())
  ///   .build()
  ///   .unwrap()
  /// ;
  ///
  /// let too_long_str = &"ab".repeat(30);
  ///
  /// assert_eq!(input.validate1(Some(&"ab")), Err(vec!["Too short".to_string()]));
  /// assert_eq!(input.validate1(Some(&too_long_str)), Err(vec!["Too long".to_string()]));
  /// assert_eq!(input.validate1(None), Err(vec!["Value missing".to_string()]));
  /// ```
  fn validate_custom(&self, value: Self::ValidatorT) -> Result<(), Vec<ValidationError>>;

  /// Validates value against contained validators.
  fn validate_with_validators(&self, value: Self::ValidatorT, validators: Option<&[&'a Validator<Self::ValidatorT>]>) -> Result<(), Vec<ValidationError>> {
    validators.map(|vs| {

      // If not break on failure then capture all validation errors.
      if !self.get_should_break_on_failure() {
        return vs.iter().fold(
          Vec::<ValidationError>::new(),
          |mut agg, f| match (f)(value) {
            Err(mut message_tuples) => {
              agg.append(message_tuples.as_mut());
              agg
            }
            _ => agg,
          });
      }

      // Else break on, and capture, first failure.
      let mut agg = Vec::<ValidationError>::new();
      for f in vs.iter() {
        if let Err(mut message_tuples) = (f)(value) {
          agg.append(message_tuples.as_mut());
          break;
        }
      }
      agg
    })
        .and_then(|messages| if messages.is_empty() { None } else { Some(messages) })
        .map_or(Ok(()), Err)
  }

  /// Validates value against any own `validate_custom` implementation and any set validators -
  /// E.g., runs `validate_custom(...)`, then, if it is `Ok`, `validate_with_validators(...)` method.
  ///
  /// Additionally, note, `break_on_failure` is only guaranteed to be respected for the
  ///   the validators list, and input filters defined in the library;  E.g., It is not guaranteed for
  /// `validate_custom()` call in external libraries (e.g., this is left to implementing struct authors).
  ///
  /// ```rust
  /// use walrs_inputfilter::*;
  /// use walrs_inputfilter::number::{
  ///   NumberValidator,
  ///   NumberValidatorBuilder,
  ///   range_overflow_msg,
  ///   range_underflow_msg,
  ///   step_mismatch_msg
  /// };
  /// use walrs_inputfilter::pattern::PatternValidator;
  /// use walrs_inputfilter::types::ConstraintViolation::{
  ///   ValueMissing, TooShort, TooLong, TypeMismatch, CustomError,
  ///   RangeOverflow, RangeUnderflow, StepMismatch
  /// };
  ///
  /// let num_validator = NumberValidatorBuilder::<isize>::default()
  ///  .min(-100isize)
  ///  .max(100isize)
  ///  .step(5)
  ///  .build()
  ///  .unwrap();
  ///
  /// let input = InputBuilder::<isize>::default()
  ///   .validators(vec![
  ///     &num_validator,
  ///     // Pretend "not allowed" case.
  ///     &|x: &isize| -> Result<(), Vec<ValidationError>> {
  ///       if *x == 45 {
  ///         return Err(vec![(CustomError, "\"45\" not allowed".to_string())]);
  ///       }
  ///      Ok(())
  ///     }
  ///   ])
  ///   .build()
  ///   .unwrap();
  ///
  /// assert_eq!(input.validate(None), Ok(()));
  /// assert_eq!(input.validate(Some(&-101)), Err(vec![(RangeUnderflow, range_underflow_msg(&num_validator, -101))]));
  /// assert_eq!(input.validate(Some(&101)), Err(vec![(RangeOverflow, range_overflow_msg(&num_validator, 101))]));
  /// assert_eq!(input.validate(Some(&100)), Ok(()));
  /// assert_eq!(input.validate(Some(&-99)), Err(vec![(StepMismatch, step_mismatch_msg(&num_validator, -99))]));
  /// assert_eq!(input.validate(Some(&95)), Ok(()));
  /// assert_eq!(input.validate(Some(&45)), Err(vec![(CustomError, "\"45\" not allowed".to_string())]));
  ///
  /// let str_input = StringInputBuilder::default()
  ///  .required(true)
  ///  .value_missing(&|_, _| "Value missing".to_string())
  ///  .min_length(3usize)
  ///  .too_short(&|_, _| "Too short".to_string())
  ///  .max_length(200usize) // Default violation message callback used here.
  ///   // Naive email pattern validator (naive for this example).
  ///  .validators(vec![&|x: &str| {
  ///     if !x.contains('@') {
  ///       return Err(vec![(TypeMismatch, "Invalid email".to_string())]);
  ///     }
  ///     Ok(())
  ///   }])
  ///  .build()
  ///  .unwrap();
  ///
  /// let too_long_str = &"ab".repeat(201);
  ///
  /// assert_eq!(str_input.validate(None), Err(vec![ (ValueMissing, "Value missing".to_string()) ]));
  /// assert_eq!(str_input.validate(Some(&"ab")), Err(vec![ (TooShort, "Too short".to_string()) ]));
  /// assert_eq!(str_input.validate(Some(&too_long_str)), Err(vec![ (TooLong, too_long_msg(&str_input, Some(&too_long_str))) ]));
  /// assert_eq!(str_input.validate(Some(&"abc")), Err(vec![ (TypeMismatch, "Invalid email".to_string()) ]));
  /// assert_eq!(str_input.validate(Some(&"abc@def")), Ok(()));
  /// ```
  fn validate(&self, value: Option<Self::ValidatorT>) -> ValidationResult {
    match value {
      None => {
        if self.get_required() {
          Err(vec![(
            ConstraintViolation::ValueMissing,
            (self.get_value_missing_handler())(self, None),
          )])
        } else {
          Ok(())
        }
      }
      // Else if value is populated validate it
      Some(v) => match self.validate_custom(v) {
        Ok(_) => self.validate_with_validators(v, self.get_validators()),
        Err(messages1) => if self.get_should_break_on_failure() {
          Err(messages1)
        } else {
          match self.validate_with_validators(v, self.get_validators()) {
            Ok(_) => Ok(()),
            Err(mut messages2) => {
              let mut agg = messages1;
              agg.append(messages2.as_mut());
              Err(agg)
            }
          }
        }
      },
    }
  }

  /// Special case of `validate` where the error type enums are ignored (in `Err(...)`) result,
  /// and only the error messages are returned.
  ///
  /// ```rust
  /// use walrs_inputfilter::*;
  ///
  /// let input = StringInputBuilder::default()
  ///   .required(true)
  ///   .value_missing(&|_, _| "Value missing".to_string())
  ///   .validators(vec![&|x: &str| {
  ///     if x.len() < 3 {
  ///       return Err(vec![(
  ///         ConstraintViolation::TooShort,
  ///        "Too short".to_string(),
  ///       )]);
  ///     }
  ///     Ok(())
  ///   }])
  ///   .build()
  ///   .unwrap()
  /// ;
  ///
  /// assert_eq!(input.validate1(Some(&"ab")), Err(vec!["Too short".to_string()]));
  /// assert_eq!(input.validate1(None), Err(vec!["Value missing".to_string()]));
  /// ```
  fn validate1(&self, value: Option<Self::ValidatorT>) -> Result<(), Vec<ViolationMessage>> {
    match self.validate(value) {
      Err(messages) =>
        Err(messages.into_iter().map(|(_, message)| message).collect()),
      Ok(_) => Ok(()),
    }
  }

  fn filter(&self, value: Option<Self::FilterT>) -> Option<Self::FilterT> {
    match self.get_filters() {
      None => value,
      Some(fs) => fs.iter().fold(value, |agg, f| (f)(agg)),
    }
  }

  fn validate_and_filter(&self, x: Option<Self::ValidatorT>) -> Result<Option<Self::FilterT>, Vec<ValidationError>> {
    self.validate(x).map(|_| self.filter(x.map(|_x| Cow::Borrowed(_x))))
  }

  /// Special case of `validate_and_filter` where the error type enums are ignored (in `Err(...)`) result,
  /// and only the error messages are returned, for `Err` case.
  ///
  /// ```rust
  /// use walrs_inputfilter::*;
  /// use std::borrow::Cow;
  ///
  /// let input = StringInputBuilder::default()
  ///   .required(true)
  ///   .value_missing(&|_, _| "Value missing".to_string())
  ///   .validators(vec![&|x: &str| {
  ///     if x.len() < 3 {
  ///       return Err(vec![(
  ///         ConstraintViolation::TooShort,
  ///        "Too short".to_string(),
  ///       )]);
  ///     }
  ///     Ok(())
  ///   }])
  ///  .filters(vec![&|xs: Option<Cow<str>>| {
  ///     xs.map(|xs| Cow::Owned(xs.to_lowercase()))
  ///   }])
  ///   .build()
  ///   .unwrap()
  /// ;
  ///
  /// assert_eq!(input.validate_and_filter1(Some(&"ab")), Err(vec!["Too short".to_string()]));
  /// assert_eq!(input.validate_and_filter1(Some(&"Abba")), Ok(Some("Abba".to_lowercase().into())));
  /// assert_eq!(input.validate_and_filter1(None), Err(vec!["Value missing".to_string()]));
  /// ```
  fn validate_and_filter1(&self, x: Option<Self::ValidatorT>) -> Result<Option<Self::FilterT>, Vec<ViolationMessage>> {
    match self.validate_and_filter(x) {
      Err(messages) =>
        Err(messages.into_iter().map(|(_, message)| message).collect()),
      Ok(filtered) => Ok(filtered),
    }
  }

}



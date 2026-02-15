use crate::{ValidateRef, ValidatorResult, Violation, ViolationMessage, ViolationType};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

/// Trait used by `LengthValidator` to get the length of a value.
/// Inspired by implementation in `validator_types` crate.
pub trait WithLength {
  fn length(&self) -> usize;
}

pub type LengthValidatorCallback<'a, T> =
  dyn Fn(&LengthValidator<'a, T>, &T) -> ViolationMessage + Send + Sync;

/// Validates the length of a value with a length (strings, collections, etc.).
///
/// ```rust
/// use walrs_inputfilter::{
///  len_too_long_msg,
///  len_too_short_msg,
///  Validate,
///  ValidateRef,
///  LengthValidator,
///  LengthValidatorBuilder,
///  Violation,
///  ViolationType
/// };
///
/// let len_one_to_ten = LengthValidatorBuilder::<str>::default()
///  .min_length(1)
///  .max_length(10)
///  .too_short_msg(&len_too_short_msg) // optional
///  .too_long_msg(&len_too_long_msg)   // optional
///  .build()
///  .unwrap();
///
///  assert!(len_one_to_ten.validate_ref("hello").is_ok());
///  assert!(len_one_to_ten.validate_ref("").is_err()); // too short
///  assert!(len_one_to_ten.validate_ref("this string is way too long").is_err());
///
/// // Collection example:
/// // ----
/// let array_len = LengthValidatorBuilder::<[i32]>::default()
///   .min_length(2)
///   .max_length(5)
///   .build()
///   .unwrap();
///
/// assert_eq!(array_len.validate_ref(&[1, 2, 3]), Ok(()));
/// assert_eq!(array_len.validate_ref(&[1]), Err(Violation(
///   ViolationType::TooShort,
///   "Value length `1` is less than allowed minimum `2`.".to_string()
/// )));
/// assert_eq!(array_len.validate_ref(&[1, 2, 3, 4, 5, 6]), Err(Violation(
///   ViolationType::TooLong,
///   "Value length `6` is greater than allowed maximum `5`.".to_string()
/// )));
/// ```
#[must_use]
#[derive(Builder, Clone)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct LengthValidator<'a, T>
where
  T: WithLength + ?Sized + 'static,
{
  #[builder(default = "None")]
  pub min_length: Option<usize>,

  #[builder(default = "None")]
  pub max_length: Option<usize>,

  #[builder(default = "&len_too_short_msg")]
  pub too_short_msg: &'a LengthValidatorCallback<'a, T>,

  #[builder(default = "&len_too_long_msg")]
  pub too_long_msg: &'a LengthValidatorCallback<'a, T>,
}

impl<'a, T: WithLength + ?Sized> LengthValidator<'a, T> {
  /// Creates a `LengthValidator` with no constraints.
  ///
  /// ```rust
  /// use walrs_inputfilter::LengthValidator;
  ///
  /// let default_vldtr = LengthValidator::<str>::new();
  ///
  /// assert!(default_vldtr.min_length.is_none());
  /// assert!(default_vldtr.max_length.is_none());
  /// ```
  pub fn new() -> Self {
    LengthValidatorBuilder::default().build().unwrap()
  }
}

// ====
// validator_types crate rip (modified for our use case):
// ====
macro_rules! validate_type_with_chars {
  ($type_:ty) => {
    impl WithLength for $type_ {
      fn length(&self) -> usize {
        self.chars().count() as usize
      }
    }
  };
}

validate_type_with_chars!(str);
validate_type_with_chars!(&str);
validate_type_with_chars!(String);

macro_rules! validate_type_with_len {
    ($type_:ty) => {
        validate_type_with_len!($type_,);
    };
    ($type_:ty, $($generic:ident),*$(,)*) => {
        impl<$($generic),*> WithLength for $type_ {
            fn length(&self) -> usize {
                self.len() as usize
            }
        }
    };
}

validate_type_with_len!([T], T);
validate_type_with_len!(BTreeSet<T>, T);
validate_type_with_len!(BTreeMap<K, V>, K, V);
validate_type_with_len!(HashSet<T, S>, T, S);
validate_type_with_len!(HashMap<K, V, S>, K, V, S);
validate_type_with_len!(Vec<T>, T);
validate_type_with_len!(VecDeque<T>, T);

// #[cfg(feature = "indexmap")]
// validate_type_with_len!(IndexSet<T>, T);
//
// #[cfg(feature = "indexmap")]
// validate_type_with_len!(IndexMap<K, V>, K, V);

// ====
// /End of validator_types crate rip.
// ====

impl<'a, T> ValidateRef<T> for LengthValidator<'a, T>
where
  T: WithLength + ?Sized,
{
  /// Validates incoming value against contained constraints.
  ///
  /// ```rust
  /// use walrs_inputfilter::{len_too_long_msg, len_too_short_msg};
  /// use walrs_inputfilter::{Violation, ViolationType::{TooLong, TooShort}};
  /// use walrs_inputfilter::{LengthValidator, LengthValidatorBuilder, ValidateRef};
  ///
  /// let no_rules = LengthValidator::<str>::new();
  /// let len_one_to_ten = LengthValidatorBuilder::<str>::default()
  ///   .min_length(1)
  ///   .max_length(10)
  ///   .build()
  ///   .unwrap();
  ///
  /// // No rules - should pass
  /// assert_eq!(no_rules.validate_ref(""), Ok(()));
  ///
  /// // Value too short
  /// assert_eq!(
  ///   len_one_to_ten.validate_ref(""),
  ///   Err(Violation(TooShort, len_too_short_msg(&len_one_to_ten, "")))
  /// );
  ///
  /// // Value too long
  /// let too_long_str = "12345678901";
  /// assert_eq!(
  ///   len_one_to_ten.validate_ref(too_long_str),
  ///   Err(Violation(TooLong, len_too_long_msg(&len_one_to_ten, too_long_str)))
  /// );
  ///
  /// // Value just right
  /// assert_eq!(len_one_to_ten.validate_ref("hello"), Ok(()));
  /// ```
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    let len = value.length();

    if let Some(min_length) = self.min_length {
      if len < min_length {
        return Err(Violation(
          ViolationType::TooShort,
          (self.too_short_msg)(self, value),
        ));
      }
    }

    if let Some(max_length) = self.max_length {
      if len > max_length {
        return Err(Violation(
          ViolationType::TooLong,
          (self.too_long_msg)(self, value),
        ));
      }
    }

    Ok(())
  }
}

#[cfg(feature = "fn_traits")]
impl<'a, T: WithLength + ?Sized> FnOnce<(&T,)> for LengthValidator<'a, T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&T,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<'a, T: WithLength + ?Sized> FnMut<(&T,)> for LengthValidator<'a, T> {
  extern "rust-call" fn call_mut(&mut self, args: (&T,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<'a, T: WithLength + ?Sized> Fn<(&T,)> for LengthValidator<'a, T> {
  extern "rust-call" fn call(&self, args: (&T,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl<'a, T: WithLength + ?Sized> Default for LengthValidator<'a, T> {
  /// Creates a `LengthValidator` with no constraints.
  ///
  /// ```rust
  /// use walrs_inputfilter::LengthValidator;
  ///
  /// let default_vldtr = LengthValidator::<str>::default();
  ///
  /// assert!(default_vldtr.min_length.is_none());
  /// assert!(default_vldtr.max_length.is_none());
  /// ```
  fn default() -> Self {
    LengthValidator::<T>::new()
  }
}

/// Returns default "too short" violation message.
///
/// ```rust
///  use walrs_inputfilter::{len_too_short_msg, LengthValidator, LengthValidatorBuilder};
///
///  let len_one_to_ten = LengthValidatorBuilder::<str>::default()
///    .min_length(1)
///    .max_length(10)
///    .too_short_msg(&len_too_short_msg) // optional
///    .build()
///    .unwrap();
///
///  assert_eq!(len_too_short_msg(&len_one_to_ten, ""), "Value length `0` is less than allowed minimum `1`.");
/// ```
pub fn len_too_short_msg<'a, T: WithLength + ?Sized>(
  rules: &LengthValidator<'a, T>,
  xs: &T,
) -> String {
  format!(
    "Value length `{}` is less than allowed minimum `{}`.",
    xs.length(),
    &rules.min_length.unwrap_or(0)
  )
}

/// Returns default "too long" violation message.
///
/// ```rust
///  use walrs_inputfilter::{len_too_long_msg, LengthValidator, LengthValidatorBuilder};
///
///  let len_one_to_ten = LengthValidatorBuilder::<str>::default()
///    .min_length(1)
///    .max_length(10)
///    .too_long_msg(&len_too_long_msg) // optional
///    .build()
///    .unwrap();
///
///  assert_eq!(len_too_long_msg(&len_one_to_ten, "this string is way too long"), "Value length `27` is greater than allowed maximum `10`.");
/// ```
pub fn len_too_long_msg<'a, T: WithLength + ?Sized>(
  rules: &LengthValidator<'a, T>,
  xs: &T,
) -> String {
  format!(
    "Value length `{}` is greater than allowed maximum `{}`.",
    xs.length(),
    &rules.max_length.unwrap_or(0)
  )
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{
    Violation,
    ViolationType::{TooLong, TooShort},
  };

  #[test]
  fn test_validate_ref() {
    let no_rules = LengthValidator::<str>::new();
    let len_one_to_ten = LengthValidatorBuilder::<str>::default()
      .min_length(1)
      .max_length(10)
      .build()
      .unwrap();

    let gte_than_8 = LengthValidatorBuilder::<str>::default()
      .min_length(8)
      .build()
      .unwrap();

    let ngte_than_8 = LengthValidatorBuilder::<str>::default()
      .max_length(8)
      .build()
      .unwrap();

    let no_rules = LengthValidator::<str>::new();

    let too_long_str = "12345678901";
    let just_right_str = &too_long_str[1..];

    let test_cases = vec![
      ("Default", &no_rules, "", Ok(())),
      (
        "Value too short",
        &len_one_to_ten,
        "",
        Err(Violation(TooShort, len_too_short_msg(&len_one_to_ten, ""))),
      ),
      (
        "Value too long",
        &len_one_to_ten,
        too_long_str,
        Err(Violation(
          TooLong,
          len_too_long_msg(&len_one_to_ten, too_long_str),
        )),
      ),
      ("Value just right (1)", &len_one_to_ten, "a", Ok(())),
      ("Value just right", &len_one_to_ten, just_right_str, Ok(())),
      (">= 8 - pass", &gte_than_8, "12345678", Ok(())),
      (
        ">= 8 - fail",
        &gte_than_8,
        "1234567",
        Err(Violation(
          TooShort,
          len_too_short_msg(&gte_than_8, "1234567"),
        )),
      ),
      ("No rules - empty str", &no_rules, "", Ok(())),
      (
        "No rules - long str",
        &no_rules,
        "this string is way too long",
        Ok(()),
      ),
    ];

    for (name, rules, value, expected) in test_cases {
      assert_eq!(rules.validate_ref(value), expected, "{}", name);
      #[cfg(feature = "fn_traits")]
      assert_eq!(rules(value), expected, "{}", name);
    }
  }

  #[test]
  fn test_with_collection_types() {
    let len_two_to_five = LengthValidatorBuilder::<[i32]>::default()
      .min_length(2)
      .max_length(5)
      .build()
      .unwrap();

    assert_eq!(len_two_to_five.validate_ref(&[1, 2, 3]), Ok(()));
    #[cfg(feature = "fn_traits")]
    assert_eq!((&len_two_to_five)(&[1, 2, 3]), Ok(()));
    #[cfg(feature = "fn_traits")]
    assert_eq!(
      (&len_two_to_five)(&[1]),
      Err(Violation(
        TooShort,
        "Value length `1` is less than allowed minimum `2`.".to_string()
      ))
    );
    #[cfg(feature = "fn_traits")]
    assert_eq!(
      (&len_two_to_five)(&[1, 2, 3, 4, 5, 6]),
      Err(Violation(
        TooLong,
        "Value length `6` is greater than allowed maximum `5`.".to_string()
      ))
    );
  }

  #[test]
  fn test_default_and_new() {
    let default_vldtr = LengthValidator::<str>::default();

    assert!(default_vldtr.min_length.is_none());
    assert!(default_vldtr.max_length.is_none());

    let new_vldtr = LengthValidator::<str>::new();

    assert!(new_vldtr.min_length.is_none());
    assert!(new_vldtr.max_length.is_none());
  }

  #[cfg(feature = "fn_traits")]
  #[test]
  fn test_fn_traits() {
    let mut vldtr = LengthValidatorBuilder::<str>::default().build().unwrap();

    fn call_fn_once(v: impl FnOnce(&str) -> ValidatorResult, s: &str) -> ValidatorResult {
      v(s)
    }

    fn call_fn_mut(v: &mut impl FnMut(&str) -> ValidatorResult, s: &str) -> ValidatorResult {
      v(s)
    }

    assert_eq!(call_fn_mut(&mut vldtr, "abc"), Ok(()));
    assert_eq!(call_fn_once(vldtr, "abc"), Ok(()));
  }
}

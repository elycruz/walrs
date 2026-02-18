use crate::{
  Message, MessageContext, MessageParams, ValidateRef, ValidatorResult, Violation, ViolationType,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

/// Trait used by `LengthValidator` to get the length of a value.
/// Inspired by implementation in `validator_types` crate.
pub trait WithLength {
  fn length(&self) -> usize;
}

/// Validates the length of a value with a length (strings, collections, etc.).
///
/// ```rust
/// use walrs_validator::{
///  Validate,
///  ValidateRef,
///  LengthValidator,
///  LengthValidatorBuilder,
///  Violation,
///  ViolationType,
///  Message,
/// };
///
/// let len_one_to_ten = LengthValidatorBuilder::<str>::default()
///  .min_length(1)
///  .max_length(10)
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

  #[builder(default = "default_len_too_short_msg()")]
  pub too_short_msg: Message<T>,

  #[builder(default = "default_len_too_long_msg()")]
  pub too_long_msg: Message<T>,

  /// Optional locale for internationalized error messages.
  #[builder(default = "None")]
  pub locale: Option<&'a str>,
}

impl<T: WithLength + ?Sized> LengthValidator<'_, T> {
  /// Creates a `LengthValidator` with no constraints.
  ///
  /// ```rust
  /// use walrs_validator::LengthValidator;
  ///
  /// let default_vldtr = LengthValidator::<str>::new();
  ///
  /// assert!(default_vldtr.min_length.is_none());
  /// assert!(default_vldtr.max_length.is_none());
  /// ```
  pub fn new() -> Self {
    LengthValidatorBuilder::default().build().unwrap()
  }

  /// Returns a builder for constructing a `LengthValidator`.
  ///
  /// ```rust
  /// use walrs_validator::LengthValidator;
  ///
  /// let vldtr = LengthValidator::<str>::builder()
  ///   .min_length(1)
  ///   .max_length(10)
  ///   .build()
  ///   .unwrap();
  ///
  /// assert_eq!(vldtr.min_length, Some(1));
  /// assert_eq!(vldtr.max_length, Some(10));
  /// ```
  pub fn builder() -> LengthValidatorBuilder<'static, T> {
    LengthValidatorBuilder::default()
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

// ====
// /End of validator_types crate rip.
// ====

impl<T> ValidateRef<T> for LengthValidator<'_, T>
where
  T: WithLength + ?Sized,
{
  /// Validates incoming value against contained constraints.
  ///
  /// ```rust
  /// use walrs_validator::{Violation, ViolationType::{TooLong, TooShort}};
  /// use walrs_validator::{LengthValidator, LengthValidatorBuilder, ValidateRef};
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
  /// assert!(len_one_to_ten.validate_ref("").is_err());
  ///
  /// // Value too long
  /// assert!(len_one_to_ten.validate_ref("12345678901").is_err());
  ///
  /// // Value just right
  /// assert_eq!(len_one_to_ten.validate_ref("hello"), Ok(()));
  /// ```
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    let len = value.length();

    if let Some(min_length) = self.min_length {
      if len < min_length {
        let params = MessageParams::new("LengthValidator")
          .with_min_length(min_length)
          .with_max_length(self.max_length.unwrap_or(0));
        let ctx = MessageContext::with_locale(value, params, self.locale);
        return Err(Violation(
          ViolationType::TooShort,
          self.too_short_msg.resolve_with_context(&ctx),
        ));
      }
    }

    if let Some(max_length) = self.max_length {
      if len > max_length {
        let params = MessageParams::new("LengthValidator")
          .with_min_length(self.min_length.unwrap_or(0))
          .with_max_length(max_length);
        let ctx = MessageContext::with_locale(value, params, self.locale);
        return Err(Violation(
          ViolationType::TooLong,
          self.too_long_msg.resolve_with_context(&ctx),
        ));
      }
    }

    Ok(())
  }
}

#[cfg(feature = "fn_traits")]
impl<T: WithLength + ?Sized> FnOnce<(&T,)> for LengthValidator<'_, T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&T,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: WithLength + ?Sized> FnMut<(&T,)> for LengthValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (&T,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

#[cfg(feature = "fn_traits")]
impl<T: WithLength + ?Sized> Fn<(&T,)> for LengthValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (&T,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl<T: WithLength + ?Sized> Default for LengthValidator<'_, T> {
  /// Creates a `LengthValidator` with no constraints.
  ///
  /// ```rust
  /// use walrs_validator::LengthValidator;
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
///  use walrs_validator::{len_too_short_msg, LengthValidator, LengthValidatorBuilder};
///
///  let len_one_to_ten = LengthValidatorBuilder::<str>::default()
///    .min_length(1)
///    .max_length(10)
///    .build()
///    .unwrap();
///
///  assert_eq!(len_too_short_msg(0, 1), "Value length `0` is less than allowed minimum `1`.");
/// ```
pub fn len_too_short_msg(actual_len: usize, min_len: usize) -> String {
  format!(
    "Value length `{}` is less than allowed minimum `{}`.",
    actual_len, min_len
  )
}

/// Returns default "too long" violation message.
///
/// ```rust
///  use walrs_validator::{len_too_long_msg, LengthValidator, LengthValidatorBuilder};
///
///  let len_one_to_ten = LengthValidatorBuilder::<str>::default()
///    .min_length(1)
///    .max_length(10)
///    .build()
///    .unwrap();
///
///  assert_eq!(len_too_long_msg(27, 10), "Value length `27` is greater than allowed maximum `10`.");
/// ```
pub fn len_too_long_msg(actual_len: usize, max_len: usize) -> String {
  format!(
    "Value length `{}` is greater than allowed maximum `{}`.",
    actual_len, max_len
  )
}

/// Returns default "too short" Message provider.
///
/// This wraps `len_too_short_msg` in a `Message::Provider` for use with `LengthValidator`.
///
/// ```rust
/// use walrs_validator::{default_len_too_short_msg, Message};
///
/// let msg: Message<str> = default_len_too_short_msg();
/// assert!(msg.is_provider());
/// ```
pub fn default_len_too_short_msg<T: WithLength + ?Sized>() -> Message<T> {
  Message::Provider(std::sync::Arc::new(|ctx: &MessageContext<T>| {
    len_too_short_msg(ctx.value.length(), ctx.params.min_length.unwrap_or(0))
  }))
}

/// Returns default "too long" Message provider.
///
/// This wraps `len_too_long_msg` in a `Message::Provider` for use with `LengthValidator`.
///
/// ```rust
/// use walrs_validator::{default_len_too_long_msg, Message};
///
/// let msg: Message<str> = default_len_too_long_msg();
/// assert!(msg.is_provider());
/// ```
pub fn default_len_too_long_msg<T: WithLength + ?Sized>() -> Message<T> {
  Message::Provider(std::sync::Arc::new(|ctx: &MessageContext<T>| {
    len_too_long_msg(ctx.value.length(), ctx.params.max_length.unwrap_or(0))
  }))
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

    let _ngte_than_8 = LengthValidatorBuilder::<str>::default()
      .max_length(8)
      .build()
      .unwrap();

    let no_rules = LengthValidator::<str>::new();

    let too_long_str = "12345678901";
    let just_right_str = &too_long_str[1..];

    // Test too short
    let result = len_one_to_ten.validate_ref("");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, TooShort);
    assert_eq!(err.1, len_too_short_msg(0, 1));

    // Test too long
    let result = len_one_to_ten.validate_ref(too_long_str);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, TooLong);
    assert_eq!(err.1, len_too_long_msg(11, 10));

    // Test valid values
    assert_eq!(len_one_to_ten.validate_ref("a"), Ok(()));
    assert_eq!(len_one_to_ten.validate_ref(just_right_str), Ok(()));
    assert_eq!(gte_than_8.validate_ref("12345678"), Ok(()));

    // Test gte_than_8 fail
    let result = gte_than_8.validate_ref("1234567");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, TooShort);

    // No rules tests
    assert_eq!(no_rules.validate_ref(""), Ok(()));
    assert_eq!(no_rules.validate_ref("this string is way too long"), Ok(()));

    #[cfg(feature = "fn_traits")]
    {
      assert_eq!((&len_one_to_ten)("hello"), Ok(()));
      assert!((&len_one_to_ten)("").is_err());
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

    let result = len_two_to_five.validate_ref(&[1]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, TooShort);
    assert_eq!(err.1, "Value length `1` is less than allowed minimum `2`.");

    let result = len_two_to_five.validate_ref(&[1, 2, 3, 4, 5, 6]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.0, TooLong);
    assert_eq!(
      err.1,
      "Value length `6` is greater than allowed maximum `5`."
    );

    #[cfg(feature = "fn_traits")]
    {
      assert_eq!((&len_two_to_five)(&[1, 2, 3]), Ok(()));
      assert!((&len_two_to_five)(&[1]).is_err());
      assert!((&len_two_to_five)(&[1, 2, 3, 4, 5, 6]).is_err());
    }
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

  #[test]
  fn test_custom_message() {
    let custom_msg: Message<str> = Message::static_msg("Custom error: too short!");
    let vldtr = LengthValidatorBuilder::<str>::default()
      .min_length(5)
      .too_short_msg(custom_msg)
      .build()
      .unwrap();

    let result = vldtr.validate_ref("abc");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.1, "Custom error: too short!");
  }

  #[test]
  fn test_message_provider() {
    let custom_msg: Message<str> = Message::provider(|ctx| {
      format!(
        "String '{}' is too short (min: {})",
        ctx.value,
        ctx.params.min_length.unwrap_or(0)
      )
    });
    let vldtr = LengthValidatorBuilder::<str>::default()
      .min_length(5)
      .too_short_msg(custom_msg)
      .build()
      .unwrap();

    let result = vldtr.validate_ref("abc");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.1, "String 'abc' is too short (min: 5)");
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

use crate::{ValidateRef, ValidatorResult, Violation, ViolationMessage, ViolationType};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

pub trait WithLength {
  fn length(&self) -> Option<usize>;
}

pub type LengthValidatorCallback<'a, T> =
  dyn Fn(&LengthValidator<'a, T>, &T) -> ViolationMessage + Send + Sync;

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
      fn length(&self) -> Option<usize> {
        Some(self.chars().count() as usize)
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
            fn length(&self) -> Option<usize> {
                Some(self.len() as usize)
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
// /End of validator_types crate rip:
// ====

///
/// Validates incoming value against contained constraints.
///
/// ```rust
/// use walrs_inputfilter::{len_too_long_msg, len_too_short_msg};
/// use walrs_inputfilter::{Violation, ViolationType::{RangeOverflow, RangeUnderflow, TooLong, TooShort}};
/// use walrs_inputfilter::{LengthValidator, LengthValidatorBuilder, ValidateRef};
///
/// let no_rules = LengthValidator::<str>::new();
/// let len_one_to_ten = LengthValidatorBuilder::<str>::default()
///   .min_length(1)
///   .max_length(10)
///   .build()
///   .unwrap();
///
/// let too_long_str = "12345678901";
/// let just_right_str = &too_long_str[1..];
///
/// let test_cases = vec![
///   ("Default", &no_rules, "", Ok(())),
///   ("Value too short", &len_one_to_ten, "", Err(Violation(TooShort, len_too_short_msg(&len_one_to_ten, ""))
///   )),
///   ("Value too long", &len_one_to_ten, too_long_str, Err(Violation(TooLong, len_too_long_msg(&len_one_to_ten, too_long_str))
///   )),
///   ("Value just right (1)", &len_one_to_ten, "a", Ok(())),
///   ("Value just right", &len_one_to_ten, just_right_str , Ok(())),
/// ];
///
/// for (name, rules, value, expected) in test_cases {
///  assert_eq!(rules.validate_ref(value), expected, "{}", name);
///  assert_eq!(rules(value), expected, "{}", name);
/// }
/// ```
impl<'a, T> ValidateRef<T> for LengthValidator<'a, T>
where
  T: WithLength + ?Sized,
{
  fn validate_ref(&self, value: &T) -> ValidatorResult {
    if let Some(len) = value.length() {
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
    }

    Ok(())
  }
}

impl<'a, T: WithLength + ?Sized> FnOnce<(&T,)> for LengthValidator<'a, T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (&T,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl<'a, T: WithLength + ?Sized> FnMut<(&T,)> for LengthValidator<'a, T> {
  extern "rust-call" fn call_mut(&mut self, args: (&T,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl<'a, T: WithLength + ?Sized> Fn<(&T,)> for LengthValidator<'a, T> {
  extern "rust-call" fn call(&self, args: (&T,)) -> Self::Output {
    self.validate_ref(args.0)
  }
}

impl<'a, T: WithLength + ?Sized> Default for LengthValidator<'a, T> {
  fn default() -> Self {
    LengthValidator::<T>::new()
  }
}

pub fn len_too_short_msg<'a, T: WithLength + ?Sized>(
  rules: &LengthValidator<'a, T>,
  xs: &T,
) -> String {
  format!(
    "Value length `{}` is less than allowed minimum `{}`.",
    xs.length().unwrap_or(0),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn len_too_long_msg<'a, T: WithLength + ?Sized>(
  rules: &LengthValidator<'a, T>,
  xs: &T,
) -> String {
  format!(
    "Value length `{}` is greater than allowed maximum `{}`.",
    xs.length().unwrap_or(0),
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
  fn test_validate_ref() -> Result<(), Box<dyn std::error::Error>> {
    let no_rules = LengthValidator::<str>::new();
    let len_one_to_ten = LengthValidatorBuilder::<str>::default()
      .min_length(1)
      .max_length(10)
      .build()
      .unwrap();

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
    ];

    for (name, rules, value, expected) in test_cases {
      assert_eq!(rules.validate_ref(value), expected, "{}", name);
      assert_eq!(rules(value), expected, "{}", name);
    }

    Ok(())
  }
}

use crate::{
  InputValue, Validate, ValidatorResult, Violation, ViolationMessage, ViolationType,
};

pub type LengthValidatorCallback<T> =
  dyn Fn(&LengthValidator<T>, T) -> ViolationMessage + Send + Sync;

#[derive(Builder, Clone)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct LengthValidator<'a, T>
where
  T: WithLength + 'static,
{
  #[builder(default = "false")]
  pub break_on_failure: bool,

  #[builder(default = "None")]
  pub min_length: Option<usize>,

  #[builder(default = "None")]
  pub max_length: Option<usize>,

  #[builder(default = "&len_too_short_msg")]
  pub too_short_msg: &'a LengthValidatorCallback<T>,

  #[builder(default = "&len_too_long_msg")]
  pub too_long_msg: &'a LengthValidatorCallback<T>,
}

impl<T: WithLength> LengthValidator<'_, T> {
  pub fn new() -> Self {
    LengthValidatorBuilder::default().build().unwrap()
  }
}

pub trait WithLength: InputValue {
  fn length(&self) -> Option<usize>;
}

impl WithLength for &str {
  fn length(&self) -> Option<usize> {
    Some(self.len())
  }
}

impl<T: InputValue> WithLength for &[T] {
  fn length(&self) -> Option<usize> {
    Some(self.len())
  }
}

// macro_rules! validate_type_with_chars {
//     ($type_:ty) => {
//         impl WithLength for $type_ {
//             fn length(&self) -> Option<usize> {
//                 Some(self.chars().count() as usize)
//             }
//         }
//     };
// }

// validate_type_with_chars!(str);
// validate_type_with_chars!(&str);
// validate_type_with_chars!(String);

// macro_rules! validate_type_with_len {
//     ($type_:ty) => {
//         validate_type_with_len!($type_,);
//     };
//     ($type_:ty, $($generic:ident),*$(,)*) => {
//         impl<$($generic),*> WithLength for $type_ {
//             fn length(&self) -> Option<usize> {
//                 Some(self.len() as usize)
//             }
//         }
//     };
// }

// validate_type_with_len!(&str);
// validate_type_with_len!([T], T);
// validate_type_with_len!(BTreeSet<T>, T);
// validate_type_with_len!(BTreeMap<K, V>, K, V);
// validate_type_with_len!(HashSet<T, S>, T, S);
// validate_type_with_len!(HashMap<K, V, S>, K, V, S);
// validate_type_with_len!(Vec<T>, T);
// validate_type_with_len!(VecDeque<T>, T);

// #[cfg(feature = "indexmap")]
// validate_type_with_len!(IndexSet<T>, T);
//
// #[cfg(feature = "indexmap")]
// validate_type_with_len!(IndexMap<K, V>, K, V);

///
/// Validates incoming value against contained constraints.
///
/// ```rust
/// use walrs_inputfilter::{len_too_long_msg, len_too_short_msg};
/// use walrs_inputfilter::{Violation, ViolationType::{RangeOverflow, RangeUnderflow, TooLong, TooShort}};
/// use walrs_inputfilter::{LengthValidator, LengthValidatorBuilder, Validate};
///
/// let no_rules = LengthValidator::new();
/// let len_one_to_ten = LengthValidatorBuilder::default()
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
///  assert_eq!(rules.validate(value), expected, "{}", name);
///  assert_eq!(rules(value), expected, "{}", name);
/// }
/// ```
impl<T: WithLength> Validate<T> for LengthValidator<'_, T> {
  fn validate(&self, value: T) -> ValidatorResult {
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

// @todo Consider implementing `ValidateRef` trait for `LengthValidator`.

impl<T: WithLength> FnOnce<(T,)> for LengthValidator<'_, T> {
  type Output = ValidatorResult;

  extern "rust-call" fn call_once(self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: WithLength> FnMut<(T,)> for LengthValidator<'_, T> {
  extern "rust-call" fn call_mut(&mut self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: WithLength> Fn<(T,)> for LengthValidator<'_, T> {
  extern "rust-call" fn call(&self, args: (T,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<T: WithLength> Default for LengthValidator<'_, T> {
  fn default() -> Self {
    LengthValidator::new()
  }
}

pub fn len_too_short_msg<T: WithLength>(rules: &LengthValidator<T>, xs: T) -> String {
  format!(
    "Value length `{}` is less than allowed minimum `{}`.",
    xs.length().unwrap_or(0),
    &rules.min_length.unwrap_or(0)
  )
}

pub fn len_too_long_msg<T: WithLength>(rules: &LengthValidator<T>, xs: T) -> String {
  format!(
    "Value length `{}` is greater than allowed maximum `{}`.",
    xs.length().unwrap_or(0),
    &rules.max_length.unwrap_or(0)
  )
}

#[cfg(test)]
mod test {
  #[test]
  fn test_validate() {}
}

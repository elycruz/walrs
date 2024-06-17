use std::fmt::Display;

pub trait InputValue: Copy + Default + Display + PartialEq + PartialOrd {}

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

impl InputValue for &str {}

type Violation = String;
type Validator<T> = dyn Fn(T) -> Option<Violation>;

#[derive(Clone)]
struct Input<'a, T> {
  required: bool,
  custom: Option<&'a Validator<T>>,
  validators: Option<Vec<&'a Validator<T>>>,
}

impl<'a, T> Input<'a, T> {
  pub fn new() -> Self {
    Self::default()
  }
}

impl<'a, T> Default for Input<'a, T> {
  fn default() -> Self {
    Self {
      required: false,
      custom: None,
      validators: None,
    }
  }
}

trait InputFilter<T> {
  fn validate(&self, value: T) -> Option<Vec<Violation>>;
}

impl<T> InputFilter<T> for Input<'_, T>
where
  T: Copy,
{
  fn validate(&self, value: T) -> Option<Vec<Violation>> {
    let mut violations = vec![];

    // If we have a single/custom validator, run it
    if let Some(violation) = self.custom.and_then(|f| f(value)) {
      violations.push(violation);
    }

    // Validate values against validators
    self
      .validators
      .as_deref()
      .map(|validators|
                     // Fold over validators
                     validators.iter().fold(vec![], |mut violations_2, v_fn| {
                         // If we have a violation, add it
                         if let Some(violation_lvl_2) = v_fn(value) {
                             violations_2.push(violation_lvl_2);
                         }
                         violations_2.clone()
                     }))
      // If we have violations, join them
      .map(|vs| {
        violations.extend(vs);
        violations
      })
      .and_then(|possible_violations| {
        if possible_violations.is_empty() {
          None
        } else {
          Some(possible_violations)
        }
      })
  }
}

fn pass_through_validator<T: Copy>(_: T) -> Option<Violation> {
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  // Returns `range_validator` error message.
  fn range_validator_error<T: InputValue>(min: T, max: T) -> Violation {
    format!("Value must be between {} and {}", min, max)
  }

  /// Range validator getter.
  fn get_range_validator<T>(min: T, max: T) -> Box<dyn Fn(T) -> Option<Violation>>
  where
    T: InputValue + 'static,
  {
    Box::new(move |value: T| {
      if value >= min && value <= max {
        None
      } else {
        Some(range_validator_error(min, max))
      }
    })
  }

  // Returns the error message for `get_char_one_of_validator`
  fn char_one_of_validator_error() -> Violation {
    "Value must be a in haystack".to_string()
  }

  // Vowel validator.
  fn get_char_one_of_validator(haystack: &'static str) -> Box<dyn Fn(char) -> Option<Violation>> {
    Box::new(move |value: char| {
      if haystack.chars().any(|c| c == value) {
        None
      } else {
        Some(char_one_of_validator_error())
      }
    })
  }

  // Returns the error message for `even_validator`
  fn even_validator_error() -> Violation {
    "Value must be even".to_string()
  }

  fn run_validate_test_cases<T: InputValue>(
    input: &Input<T>,
    test_values: &[(T, Option<Vec<Violation>>)],
  ) {
    for (i, (value, _expected)) in test_values.iter().enumerate() {
      println!("Case {}; `#.validate({:}) == {:?}`", i, value, _expected);
      // Test validation result
      input.validate(*value).as_deref().map_or_else(
        || assert!(matches!(None::<Vec<Violation>>, _expected)),
        |vs| {
          if let Some(rhs_vs) = _expected.as_deref() {
            assert_eq!(vs, rhs_vs);
          } else {
            panic!("Expected None, got Some({:?})", vs);
          }
        },
      );
    }
  }

  #[test]
  fn test_new_and_default() {
    fn validate_input<T: InputValue>(input: &Input<T>) {
      assert!(!input.required);
      assert!(input.custom.is_none());
      assert!(input.validators.is_none());
    }

    [Input::<isize>::new(), Input::<isize>::default()]
      .iter()
      .for_each(validate_input);
    [Input::<&str>::new(), Input::<&str>::default()]
      .iter()
      .for_each(validate_input);
  }

  #[test]
  fn test_custom_validator() {
    let mut input = Input::new();
    let vowel_validator = &get_char_one_of_validator("aeiou");
    input.custom = Some(vowel_validator);

    let test_values = vec![
      ('a', None),
      ('b', Some(vec![char_one_of_validator_error()])),
      ('c', Some(vec![char_one_of_validator_error()])),
      ('e', None),
      ('i', None),
      ('o', None),
      ('u', None),
    ];

    println!("Input with `custom` validator - vowel_validator");
    run_validate_test_cases(&input, &test_values);
  }

  #[test]
  fn test_general() {
    // Validates even numbers
    let even_validator = |value: isize| {
      if value % 2 == 0 {
        None
      } else {
        Some("Value must be even".to_string())
      }
    };

    // Validator that checks that value is a vowel
    let vowel_validator = get_char_one_of_validator("aeiou");

    let one_to_ten_validator = get_range_validator(1isize, 10);

    let mut number_input = Input::<isize>::new();
    number_input.custom = Some(&pass_through_validator);
    number_input.validators = Some(vec![&one_to_ten_validator, &even_validator]);

    // Test the number input
    // Create a table of test values and expected results
    let test_values = vec![
      (1, Some(vec![even_validator_error()])),
      (2, None),
      (3, Some(vec![even_validator_error()])),
      (
        11,
        Some(vec![range_validator_error(1, 10), even_validator_error()]),
      ),
    ];

    println!("Input with 1 - 10, even number, validator, and pass-through (custom) validators");
    run_validate_test_cases(&number_input, &test_values);

    // Create an input for vowels
    let mut vowel_input = Input::<char>::new();
    vowel_input.custom = Some(&vowel_validator);

    // Create a table of test values and expected results
    let test_values = vec![
      ('a', None),
      ('b', Some(vec!["Value must be a vowel".to_string()])),
      ('c', Some(vec!["Value must be a vowel".to_string()])),
      ('e', None),
      ('i', None),
      ('o', None),
      ('u', None),
    ];

    println!("Input with a `custom` validator - vowel_validator");
    run_validate_test_cases(&vowel_input, &test_values);
  }
}

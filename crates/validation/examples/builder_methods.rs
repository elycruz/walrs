/// Example demonstrating the new static `builder()` methods on all validators.
///
/// This example shows how to use the builder pattern to construct validators
/// with custom configurations using the new static `builder()` methods.
use regex::Regex;
use std::borrow::Cow;
use walrs_validation::{
  EqualityValidator, LengthValidator, PatternValidator, RangeValidator, StepValidator, Validate,
  ValidateRef,
};

fn main() {
  println!("=== Validator Builder Methods Example ===\n");

  // 1. EqualityValidator::builder()
  println!("1. EqualityValidator");
  let equality_validator = EqualityValidator::<&str>::builder()
    .rhs_value("hello")
    .build()
    .unwrap();
  println!(
    "   Validating 'hello': {:?}",
    equality_validator.validate("hello")
  );
  println!(
    "   Validating 'world': {:?}",
    equality_validator.validate("world")
  );
  println!();

  // 2. LengthValidator::builder()
  println!("2. LengthValidator");
  let length_validator = LengthValidator::<str>::builder()
    .min_length(3)
    .max_length(10)
    .build()
    .unwrap();
  println!(
    "   Validating 'hello': {:?}",
    length_validator.validate_ref("hello")
  );
  println!(
    "   Validating 'hi': {:?}",
    length_validator.validate_ref("hi")
  );
  println!(
    "   Validating 'this is too long': {:?}",
    length_validator.validate_ref("this is too long")
  );
  println!();

  // 3. PatternValidator::builder()
  println!("3. PatternValidator");
  let rx = Regex::new(r"^\w{2,10}$").unwrap();
  let pattern_validator = PatternValidator::builder()
    .pattern(Cow::Owned(rx))
    .build()
    .unwrap();
  println!(
    "   Validating 'hello': {:?}",
    pattern_validator.validate_ref("hello")
  );
  println!(
    "   Validating '123': {:?}",
    pattern_validator.validate_ref("123")
  );
  println!(
    "   Validating '!@#': {:?}",
    pattern_validator.validate_ref("!@#")
  );
  println!();

  // 4. RangeValidator::builder()
  println!("4. RangeValidator");
  let range_validator = RangeValidator::<i32>::builder()
    .min(0)
    .max(100)
    .build()
    .unwrap();
  println!("   Validating 50: {:?}", range_validator.validate(50));
  println!("   Validating -1: {:?}", range_validator.validate(-1));
  println!("   Validating 101: {:?}", range_validator.validate(101));
  println!();

  // 5. StepValidator::builder()
  println!("5. StepValidator");
  let step_validator = StepValidator::<usize>::builder().step(5).build().unwrap();
  println!("   Validating 0: {:?}", step_validator.validate(0));
  println!("   Validating 10: {:?}", step_validator.validate(10));
  println!("   Validating 7: {:?}", step_validator.validate(7));
  println!();

  println!("=== All builder methods work correctly! ===");
}

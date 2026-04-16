//! Integration tests for FormData bridge in derive(Fieldset).

use walrs_fieldfilter::DeriveFieldset as Fieldset;
use walrs_form::FormData;
use walrs_validation::{Value, ViolationType};

#[derive(Debug, Fieldset, PartialEq)]
#[fieldset(into_form_data, try_from_form_data)]
struct UserProfile {
  username: String,
  age: i64,
  score: f64,
  is_active: bool,
  middle_initial: char,
}

#[derive(Debug, Fieldset, PartialEq)]
#[fieldset(into_form_data, try_from_form_data)]
struct OptionalFields {
  name: Option<String>,
  age: Option<i32>,
  score: Option<f64>,
  active: Option<bool>,
}

#[test]
fn test_into_form_data_required_fields() {
  let profile = UserProfile {
    username: "alice".to_string(),
    age: 30,
    score: 95.5,
    is_active: true,
    middle_initial: 'M',
  };

  let form_data = FormData::from(&profile);

  assert_eq!(
    form_data.get_direct("username"),
    Some(&Value::Str("alice".to_string()))
  );
  assert_eq!(form_data.get_direct("age"), Some(&Value::I64(30)));
  assert_eq!(form_data.get_direct("score"), Some(&Value::F64(95.5)));
  assert_eq!(form_data.get_direct("is_active"), Some(&Value::Bool(true)));
  assert_eq!(
    form_data.get_direct("middle_initial"),
    Some(&Value::Str("M".to_string()))
  );
}

#[test]
fn test_try_from_form_data_required_fields() {
  let mut form_data = FormData::new();
  form_data.insert("username", Value::Str("bob".to_string()));
  form_data.insert("age", Value::I64(25));
  form_data.insert("score", Value::F64(88.0));
  form_data.insert("is_active", Value::Bool(false));
  form_data.insert("middle_initial", Value::Str("J".to_string()));

  let profile = UserProfile::try_from(form_data).expect("Should convert successfully");

  assert_eq!(profile.username, "bob");
  assert_eq!(profile.age, 25);
  assert_eq!(profile.score, 88.0);
  assert!(!profile.is_active);
  assert_eq!(profile.middle_initial, 'J');
}

#[test]
fn test_into_form_data_optional_fields() {
  let data = OptionalFields {
    name: Some("test".to_string()),
    age: Some(42),
    score: None,
    active: Some(true),
  };

  let form_data = FormData::from(&data);

  assert_eq!(
    form_data.get_direct("name"),
    Some(&Value::Str("test".to_string()))
  );
  assert_eq!(form_data.get_direct("age"), Some(&Value::I64(42)));
  assert_eq!(form_data.get_direct("score"), Some(&Value::Null));
  assert_eq!(form_data.get_direct("active"), Some(&Value::Bool(true)));
}

#[test]
fn test_try_from_form_data_optional_fields() {
  let mut form_data = FormData::new();
  form_data.insert("name", Value::Str("charlie".to_string()));
  form_data.insert("age", Value::Null);
  form_data.insert("score", Value::F64(99.9));
  form_data.insert("active", Value::Bool(false));

  let data = OptionalFields::try_from(form_data).expect("Should convert successfully");

  assert_eq!(data.name, Some("charlie".to_string()));
  assert_eq!(data.age, None);
  assert_eq!(data.score, Some(99.9));
  assert_eq!(data.active, Some(false));
}

#[test]
fn test_try_from_form_data_missing_fields() {
  let form_data = FormData::new();

  let result = UserProfile::try_from(form_data);

  assert!(result.is_err());
  let violations = result.unwrap_err();
  assert!(!violations.is_empty());
  // Should have violations for all required fields
  assert!(violations.get("username").is_some());
  assert!(violations.get("age").is_some());
  assert!(violations.get("score").is_some());
  assert!(violations.get("is_active").is_some());
  assert!(violations.get("middle_initial").is_some());
}

#[test]
fn test_try_from_form_data_type_mismatch() {
  let mut form_data = FormData::new();
  form_data.insert("username", Value::I64(123)); // Wrong type
  form_data.insert("age", Value::Str("not a number".to_string())); // Wrong type
  form_data.insert("score", Value::Bool(true)); // Wrong type
  form_data.insert("is_active", Value::Str("yes".to_string())); // Wrong type
  form_data.insert("middle_initial", Value::I64(77)); // Wrong type

  let result = UserProfile::try_from(form_data);

  assert!(result.is_err());
  let violations = result.unwrap_err();
  assert!(!violations.is_empty());
  // Should have type mismatch violations for all fields
  assert!(violations.get("username").is_some());
  assert!(violations.get("age").is_some());
  assert!(violations.get("score").is_some());
  assert!(violations.get("is_active").is_some());
  assert!(violations.get("middle_initial").is_some());
}

#[test]
fn test_roundtrip_conversion() {
  let original = UserProfile {
    username: "dave".to_string(),
    age: 35,
    score: 92.3,
    is_active: true,
    middle_initial: 'X',
  };

  let form_data = FormData::from(&original);
  let restored = UserProfile::try_from(form_data).expect("Should convert back successfully");

  assert_eq!(original, restored);
}

#[test]
fn test_optional_roundtrip() {
  let original = OptionalFields {
    name: Some("eve".to_string()),
    age: None,
    score: Some(87.5),
    active: None,
  };

  let form_data = FormData::from(&original);
  let restored = OptionalFields::try_from(form_data).expect("Should convert back successfully");

  assert_eq!(original, restored);
}

#[test]
fn test_numeric_type_conversions() {
  #[derive(Debug, Fieldset, PartialEq)]
  #[fieldset(into_form_data, try_from_form_data)]
  struct NumericTypes {
    i8_val: i8,
    i16_val: i16,
    i32_val: i32,
    i64_val: i64,
    u8_val: u8,
    u16_val: u16,
    u32_val: u32,
    u64_val: u64,
    f32_val: f32,
    f64_val: f64,
  }

  let data = NumericTypes {
    i8_val: -10,
    i16_val: -1000,
    i32_val: -100000,
    i64_val: -10000000,
    u8_val: 10,
    u16_val: 1000,
    u32_val: 100000,
    u64_val: 10000000,
    f32_val: 3.14,
    f64_val: 2.71828,
  };

  let form_data = FormData::from(&data);
  let restored = NumericTypes::try_from(form_data).expect("Should convert successfully");

  assert_eq!(data.i8_val, restored.i8_val);
  assert_eq!(data.i16_val, restored.i16_val);
  assert_eq!(data.i32_val, restored.i32_val);
  assert_eq!(data.i64_val, restored.i64_val);
  assert_eq!(data.u8_val, restored.u8_val);
  assert_eq!(data.u16_val, restored.u16_val);
  assert_eq!(data.u32_val, restored.u32_val);
  assert_eq!(data.u64_val, restored.u64_val);
  assert!((data.f32_val - restored.f32_val).abs() < 0.0001);
  assert!((data.f64_val - restored.f64_val).abs() < 0.0001);
}

#[test]
fn test_overflow_i8_produces_violation() {
  #[derive(Debug, Fieldset, PartialEq)]
  #[fieldset(into_form_data, try_from_form_data)]
  struct SmallInts {
    val_i8: i8,
    val_u8: u8,
  }

  // 200 overflows i8 (max 127)
  let mut form_data = FormData::new();
  form_data.insert("val_i8", Value::I64(200));
  form_data.insert("val_u8", Value::U64(10));

  let result = SmallInts::try_from(form_data);
  assert!(result.is_err());
  let violations = result.unwrap_err();
  let v = violations.get("val_i8").expect("should have val_i8 violation");
  assert_eq!(v[0].violation_type(), ViolationType::TypeMismatch);
}

#[test]
fn test_overflow_u8_produces_violation() {
  #[derive(Debug, Fieldset, PartialEq)]
  #[fieldset(into_form_data, try_from_form_data)]
  struct SmallUints {
    val: u8,
  }

  // 300 overflows u8 (max 255)
  let mut form_data = FormData::new();
  form_data.insert("val", Value::U64(300));

  let result = SmallUints::try_from(form_data);
  assert!(result.is_err());
  let violations = result.unwrap_err();
  let v = violations.get("val").expect("should have val violation");
  assert_eq!(v[0].violation_type(), ViolationType::TypeMismatch);
}

#[test]
fn test_negative_to_unsigned_produces_violation() {
  #[derive(Debug, Fieldset, PartialEq)]
  #[fieldset(into_form_data, try_from_form_data)]
  struct UnsignedField {
    val: u32,
  }

  let mut form_data = FormData::new();
  form_data.insert("val", Value::I64(-5));

  let result = UnsignedField::try_from(form_data);
  assert!(result.is_err());
}

#[test]
fn test_i128_u128_roundtrip() {
  #[derive(Debug, Fieldset, PartialEq)]
  #[fieldset(into_form_data, try_from_form_data)]
  struct BigInts {
    signed: i128,
    unsigned: u128,
  }

  let original = BigInts {
    signed: i128::MAX,
    unsigned: u128::MAX,
  };

  let form_data = FormData::from(&original);

  // Should be stored as strings
  assert!(matches!(
    form_data.get_direct("signed"),
    Some(Value::Str(_))
  ));
  assert!(matches!(
    form_data.get_direct("unsigned"),
    Some(Value::Str(_))
  ));

  let restored = BigInts::try_from(form_data).expect("Should roundtrip successfully");
  assert_eq!(original, restored);
}

#[test]
fn test_option_i128_u128_roundtrip() {
  #[derive(Debug, Fieldset, PartialEq)]
  #[fieldset(into_form_data, try_from_form_data)]
  struct OptBigInts {
    signed: Option<i128>,
    unsigned: Option<u128>,
  }

  let original = OptBigInts {
    signed: Some(i128::MIN),
    unsigned: Some(u128::MAX),
  };

  let form_data = FormData::from(&original);
  let restored = OptBigInts::try_from(form_data).expect("Should roundtrip successfully");
  assert_eq!(original, restored);

  // Test None roundtrip
  let none_data = OptBigInts {
    signed: None,
    unsigned: None,
  };
  let form_data = FormData::from(&none_data);
  let restored = OptBigInts::try_from(form_data).expect("Should roundtrip None successfully");
  assert_eq!(none_data, restored);
}

#[test]
fn test_i128_invalid_string_produces_violation() {
  #[derive(Debug, Fieldset, PartialEq)]
  #[fieldset(into_form_data, try_from_form_data)]
  struct I128Field {
    val: i128,
  }

  let mut form_data = FormData::new();
  form_data.insert("val", Value::Str("not_a_number".to_string()));

  let result = I128Field::try_from(form_data);
  assert!(result.is_err());
  let violations = result.unwrap_err();
  assert!(violations.get("val").is_some());
}

#[test]
fn test_i128_from_i64_value() {
  #[derive(Debug, Fieldset, PartialEq)]
  #[fieldset(into_form_data, try_from_form_data)]
  struct I128Field {
    val: i128,
  }

  // i128 should also accept I64 values (widening)
  let mut form_data = FormData::new();
  form_data.insert("val", Value::I64(42));

  let result = I128Field::try_from(form_data).expect("Should accept I64 for i128");
  assert_eq!(result.val, 42i128);
}

#[test]
fn test_option_overflow_i16_produces_violation() {
  #[derive(Debug, Fieldset, PartialEq)]
  #[fieldset(into_form_data, try_from_form_data)]
  struct OptSmall {
    val: Option<i16>,
  }

  // 40000 overflows i16 (max 32767)
  let mut form_data = FormData::new();
  form_data.insert("val", Value::I64(40000));

  let result = OptSmall::try_from(form_data);
  assert!(result.is_err());
  let violations = result.unwrap_err();
  let v = violations.get("val").expect("should have val violation");
  assert_eq!(v[0].violation_type(), ViolationType::TypeMismatch);
}

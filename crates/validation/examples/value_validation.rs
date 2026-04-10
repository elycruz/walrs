//! # Value Validation Example
//!
//! Demonstrates dynamic `Value` type validation — useful when working with
//! form data or JSON payloads where the exact type is not known at compile time.
//!
//! The `serde_json_bridge` feature (enabled by default) allows converting
//! between `serde_json::Value` and `walrs_validation::Value`.

use walrs_validation::{Rule, Validate, ValidateRef, Value, value};

fn main() {
    // -------------------------------------------------------------------------
    // Required — null / empty values are invalid
    // -------------------------------------------------------------------------
    let required = Rule::<Value>::Required;

    assert!(required.validate(Value::Str("hello".into())).is_ok());
    assert!(required.validate(Value::I64(0)).is_ok()); // zero is present
    assert!(required.validate(Value::Null).is_err());
    assert!(required.validate(Value::Str(String::new())).is_err()); // empty string

    println!("Required Value validation: OK");

    // -------------------------------------------------------------------------
    // MinLength / MaxLength on string values
    // -------------------------------------------------------------------------
    let len_rule = Rule::<Value>::MinLength(3).and(Rule::MaxLength(20));

    assert!(len_rule.validate_ref(&Value::Str("hello".into())).is_ok());
    assert!(len_rule.validate_ref(&Value::Str("hi".into())).is_err());
    // Note: MinLength/MaxLength on non-string Values returns a TypeMismatch error
    assert!(len_rule.validate_ref(&Value::I64(42)).is_err());

    println!("Value length validation: OK");

    // -------------------------------------------------------------------------
    // Min / Max on numeric values
    // -------------------------------------------------------------------------
    let range_rule = Rule::<Value>::Min(Value::I64(0)).and(Rule::Max(Value::I64(100)));

    assert!(range_rule.validate(Value::I64(50)).is_ok());
    assert!(range_rule.validate(Value::I64(-1)).is_err());
    assert!(range_rule.validate(Value::I64(101)).is_err());
    // Note: Numeric rules on non-numeric Values return a TypeMismatch error
    assert!(range_rule.validate(Value::Str("hello".into())).is_err());

    println!("Value numeric range validation: OK");

    // -------------------------------------------------------------------------
    // Pattern on string values
    // -------------------------------------------------------------------------
    let email_pattern = Rule::<Value>::pattern(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();

    assert!(email_pattern.validate(Value::Str("user@example.com".into())).is_ok());
    assert!(email_pattern.validate(Value::Str("not-an-email".into())).is_err());

    println!("Value pattern validation: OK");

    // -------------------------------------------------------------------------
    // value! macro for convenient Value construction
    // -------------------------------------------------------------------------
    let v = value!("hello");
    assert_eq!(v, Value::Str("hello".into()));

    let n = value!(42_i64);
    assert_eq!(n, Value::I64(42));

    let b = value!(true);
    assert_eq!(b, Value::Bool(true));

    println!("value! macro: OK");

    // -------------------------------------------------------------------------
    // serde_json bridge — convert from serde_json::Value
    // -------------------------------------------------------------------------
    #[cfg(feature = "serde_json_bridge")]
    {
        let json_val: serde_json::Value = serde_json::json!("test string");
        let native_val: Value = json_val.into();
        assert_eq!(native_val, Value::Str("test string".into()));

        let json_num: serde_json::Value = serde_json::json!(42);
        let native_num: Value = json_num.into();
        assert_eq!(native_num, Value::I64(42));

        println!("serde_json bridge: OK");
    }

    // -------------------------------------------------------------------------
    // Composed rules on Value
    // -------------------------------------------------------------------------
    let form_field_rule = Rule::<Value>::Required
        .and(Rule::MinLength(2))
        .and(Rule::MaxLength(50));

    assert!(form_field_rule.validate(Value::Str("Alice".into())).is_ok());
    assert!(form_field_rule.validate(Value::Null).is_err());
    assert!(form_field_rule.validate(Value::Str("A".into())).is_err());

    println!("Composed Value rules: OK");

    println!("\nAll value validation examples passed!");
}

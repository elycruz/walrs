#![cfg(feature = "async")]
#![allow(deprecated)]

use std::sync::Arc;
use walrs_fieldfilter::{FieldBuilder, Value, Violation, ViolationType};
use walrs_validation::Rule;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn async_not_banned_rule(banned: &'static str) -> Rule<Value> {
  Rule::custom_async(Arc::new(move |value: &Value| {
    Box::pin(async move {
      if let Some(s) = value.as_str() {
        if s == banned {
          return Err(Violation::new(ViolationType::CustomError, "banned"));
        }
      }
      Ok(())
    })
  }))
}

// ---------------------------------------------------------------------------
// Field<String> async tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn string_field_validate_ref_async_passes() {
  let field = FieldBuilder::<String>::default()
    .rule(Rule::Required.and(Rule::MinLength(3)))
    .build()
    .unwrap();
  assert!(field.validate_ref_async("hello").await.is_ok());
}

#[tokio::test]
async fn string_field_validate_ref_async_fails() {
  let field = FieldBuilder::<String>::default()
    .rule(Rule::MinLength(10))
    .build()
    .unwrap();
  assert!(field.validate_ref_async("short").await.is_err());
}

#[tokio::test]
async fn string_field_with_custom_async_rule() {
  let rule = Rule::<String>::custom_async(Arc::new(|value: &String| {
    Box::pin(async move {
      if value == "forbidden" {
        Err(Violation::new(
          ViolationType::CustomError,
          "forbidden value",
        ))
      } else {
        Ok(())
      }
    })
  }));
  let field = FieldBuilder::<String>::default()
    .rule(rule)
    .build()
    .unwrap();

  assert!(field.validate_ref_async("ok").await.is_ok());
  assert!(field.validate_ref_async("forbidden").await.is_err());
}

#[tokio::test]
async fn string_field_clean_async() {
  let rule = Rule::<String>::custom_async(Arc::new(|value: &String| {
    Box::pin(async move {
      if value.contains("bad") {
        Err(Violation::new(ViolationType::CustomError, "contains bad"))
      } else {
        Ok(())
      }
    })
  }));
  let field = FieldBuilder::<String>::default()
    .rule(Rule::Required.and(rule))
    .filters(vec![walrs_filter::FilterOp::Trim])
    .build()
    .unwrap();

  let result = field.clean_async("  good  ".to_string()).await;
  assert_eq!(result.unwrap(), "good");

  let result = field.clean_async("  bad  ".to_string()).await;
  assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Field<Value> async tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn value_field_validate_ref_async_passes() {
  let field = FieldBuilder::<Value>::default()
    .rule(async_not_banned_rule("bad"))
    .build()
    .unwrap();
  assert!(
    field
      .validate_ref_async(&Value::Str("good".into()))
      .await
      .is_ok()
  );
}

#[tokio::test]
async fn value_field_validate_ref_async_fails() {
  let field = FieldBuilder::<Value>::default()
    .rule(async_not_banned_rule("bad"))
    .build()
    .unwrap();
  assert!(
    field
      .validate_ref_async(&Value::Str("bad".into()))
      .await
      .is_err()
  );
}

#[tokio::test]
async fn value_field_clean_async() {
  let field = FieldBuilder::<Value>::default()
    .rule(Rule::Required.and(async_not_banned_rule("bad")))
    .build()
    .unwrap();

  let ok = field.clean_async(Value::Str("good".into())).await;
  assert!(ok.is_ok());

  let err = field.clean_async(Value::Str("bad".into())).await;
  assert!(err.is_err());
}

#![cfg(feature = "async")]

use indexmap::IndexMap;
use std::sync::Arc;
use walrs_fieldfilter::{
  CrossFieldRule, CrossFieldRuleType, Field, FieldBuilder, FieldFilter, Value, Violation,
  ViolationType,
};
use walrs_validation::Rule;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_data(pairs: &[(&str, Value)]) -> IndexMap<String, Value> {
  pairs
    .iter()
    .map(|(k, v)| (k.to_string(), v.clone()))
    .collect()
}

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
async fn string_field_process_async() {
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

  let result = field.process_async("  good  ".to_string()).await;
  assert_eq!(result.unwrap(), "good");

  let result = field.process_async("  bad  ".to_string()).await;
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
async fn value_field_process_async() {
  let field = FieldBuilder::<Value>::default()
    .rule(Rule::Required.and(async_not_banned_rule("bad")))
    .build()
    .unwrap();

  let ok = field.process_async(Value::Str("good".into())).await;
  assert!(ok.is_ok());

  let err = field.process_async(Value::Str("bad".into())).await;
  assert!(err.is_err());
}

// ---------------------------------------------------------------------------
// FieldFilter async tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn field_filter_validate_async_passes() {
  let mut ff = FieldFilter::new();
  ff.add_field(
    "name",
    FieldBuilder::<Value>::default()
      .rule(Rule::Required)
      .build()
      .unwrap(),
  );

  let data = make_data(&[("name", Value::Str("Alice".into()))]);
  assert!(ff.validate_async(&data).await.is_ok());
}

#[tokio::test]
async fn field_filter_validate_async_fails() {
  let mut ff = FieldFilter::new();
  ff.add_field(
    "name",
    FieldBuilder::<Value>::default()
      .rule(Rule::Required)
      .build()
      .unwrap(),
  );

  let data = make_data(&[("name", Value::Null)]);
  assert!(ff.validate_async(&data).await.is_err());
}

#[tokio::test]
async fn field_filter_with_async_field_rule() {
  let mut ff = FieldFilter::new();
  ff.add_field(
    "username",
    FieldBuilder::<Value>::default()
      .rule(async_not_banned_rule("admin"))
      .build()
      .unwrap(),
  );

  let ok_data = make_data(&[("username", Value::Str("alice".into()))]);
  assert!(ff.validate_async(&ok_data).await.is_ok());

  let bad_data = make_data(&[("username", Value::Str("admin".into()))]);
  assert!(ff.validate_async(&bad_data).await.is_err());
}

#[tokio::test]
async fn field_filter_with_async_cross_field_rule() {
  let mut ff = FieldFilter::new();
  ff.add_field(
    "email",
    FieldBuilder::<Value>::default()
      .rule(Rule::Required)
      .build()
      .unwrap(),
  );
  ff.add_cross_field_rule(CrossFieldRule {
    name: Some("custom_check".into()),
    fields: vec!["email".to_string()],
    rule: CrossFieldRuleType::CustomAsync(Arc::new(|data| {
      Box::pin(async move {
        let email = data.get("email").and_then(|v| v.as_str()).unwrap_or("");
        if email.contains("@blocked.com") {
          Err(Violation::new(ViolationType::CustomError, "blocked domain"))
        } else {
          Ok(())
        }
      })
    })),
  });

  let ok_data = make_data(&[("email", Value::Str("user@good.com".into()))]);
  assert!(ff.validate_async(&ok_data).await.is_ok());

  let bad_data = make_data(&[("email", Value::Str("user@blocked.com".into()))]);
  let err = ff.validate_async(&bad_data).await.unwrap_err();
  assert!(!err.form.is_empty());
}

#[tokio::test]
async fn field_filter_process_async() {
  let mut ff = FieldFilter::new();
  ff.add_field(
    "name",
    FieldBuilder::<Value>::default()
      .rule(Rule::Required.and(async_not_banned_rule("bad")))
      .filters(vec![walrs_filter::FilterOp::Trim])
      .build()
      .unwrap(),
  );

  let data = make_data(&[("name", Value::Str("  good  ".into()))]);
  let result = ff.process_async(data).await;
  assert!(result.is_ok());
  let processed = result.unwrap();
  assert_eq!(processed["name"], Value::Str("good".into()));

  let data = make_data(&[("name", Value::Str("bad".into()))]);
  assert!(ff.process_async(data).await.is_err());
}

// ---------------------------------------------------------------------------
// Mixed sync + async in FieldFilter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn field_filter_mixed_sync_and_async_rules() {
  let mut ff = FieldFilter::new();

  // Field with sync rule
  ff.add_field(
    "age",
    FieldBuilder::<Value>::default()
      .rule(Rule::Required)
      .build()
      .unwrap(),
  );

  // Field with async rule
  ff.add_field(
    "username",
    FieldBuilder::<Value>::default()
      .rule(async_not_banned_rule("root"))
      .build()
      .unwrap(),
  );

  // Sync cross-field rule
  ff.add_cross_field_rule(CrossFieldRule {
    name: Some("one_required".into()),
    fields: vec!["age".to_string(), "username".to_string()],
    rule: CrossFieldRuleType::OneOfRequired(vec!["age".to_string(), "username".to_string()]),
  });

  let data = make_data(&[
    ("age", Value::I64(25)),
    ("username", Value::Str("alice".into())),
  ]);
  assert!(ff.validate_async(&data).await.is_ok());
}

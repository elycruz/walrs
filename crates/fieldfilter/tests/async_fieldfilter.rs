#![cfg(feature = "async")]

use std::sync::Arc;
use walrs_fieldfilter::{FieldBuilder, Violation, ViolationType};
use walrs_validation::Rule;

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

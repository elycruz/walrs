#![cfg(feature = "async")]
#![allow(deprecated)]

use std::sync::Arc;
use walrs_validation::{Rule, ValidateAsync, ValidateRefAsync, Violation, ViolationType};

// ---------------------------------------------------------------------------
// Helper: a simple async validator closure
// ---------------------------------------------------------------------------

fn async_is_not_banned(banned: &'static str) -> Rule<String> {
  Rule::custom_async(Arc::new(move |value: &String| {
    Box::pin(async move {
      if value == banned {
        Err(Violation::new(ViolationType::CustomError, "banned value"))
      } else {
        Ok(())
      }
    })
  }))
}

fn async_str_is_not_banned(banned: &'static str) -> Rule<String> {
  Rule::custom_async(Arc::new(move |value: &String| {
    Box::pin(async move {
      if value.as_str() == banned {
        Err(Violation::new(ViolationType::CustomError, "banned value"))
      } else {
        Ok(())
      }
    })
  }))
}

// ---------------------------------------------------------------------------
// String async validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn custom_async_string_passes() {
  let rule = async_str_is_not_banned("bad");
  assert!(rule.validate_ref_async("good").await.is_ok());
}

#[tokio::test]
async fn custom_async_string_fails() {
  let rule = async_str_is_not_banned("bad");
  let err = rule.validate_ref_async("bad").await.unwrap_err();
  assert_eq!(err.message(), "banned value");
}

#[tokio::test]
async fn sync_rule_works_via_async_string() {
  let rule = Rule::<String>::MinLength(3);
  assert!(rule.validate_ref_async("hello").await.is_ok());
  assert!(rule.validate_ref_async("hi").await.is_err());
}

// ---------------------------------------------------------------------------
// Numeric async validation
// ---------------------------------------------------------------------------

fn async_is_even() -> Rule<i64> {
  Rule::custom_async(Arc::new(|value: &i64| {
    Box::pin(async move {
      if *value % 2 == 0 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::CustomError, "must be even"))
      }
    })
  }))
}

#[tokio::test]
async fn custom_async_numeric_passes() {
  let rule = async_is_even();
  assert!(rule.validate_async(4i64).await.is_ok());
}

#[tokio::test]
async fn custom_async_numeric_fails() {
  let rule = async_is_even();
  let err = rule.validate_async(3i64).await.unwrap_err();
  assert_eq!(err.message(), "must be even");
}

#[tokio::test]
async fn sync_rule_works_via_async_numeric() {
  let rule = Rule::<i64>::Min(0);
  assert!(rule.validate_async(5i64).await.is_ok());
  assert!(rule.validate_async(-1i64).await.is_err());
}

// ---------------------------------------------------------------------------
// Composite rules with async children
// ---------------------------------------------------------------------------

#[tokio::test]
async fn all_with_async_children() {
  let rule = Rule::<String>::MinLength(3).and(async_str_is_not_banned("bad"));
  assert!(rule.validate_ref_async("good").await.is_ok());
  assert!(rule.validate_ref_async("hi").await.is_err()); // too short
  assert!(rule.validate_ref_async("bad").await.is_err()); // banned
}

#[tokio::test]
async fn any_with_async_children() {
  // Either passes min-length OR is not banned
  let rule = Rule::<String>::MinLength(100).or(async_str_is_not_banned("bad"));
  assert!(rule.validate_ref_async("good").await.is_ok()); // fails length, passes async
  assert!(rule.validate_ref_async("bad").await.is_err()); // fails both
}

#[tokio::test]
async fn not_with_async_child() {
  let rule = async_str_is_not_banned("bad").not();
  // Not(passes) => fails
  assert!(rule.validate_ref_async("good").await.is_err());
  // Not(fails) => passes
  assert!(rule.validate_ref_async("bad").await.is_ok());
}

#[tokio::test]
async fn when_with_async_then_branch() {
  let rule = async_str_is_not_banned("bad").when(walrs_validation::Condition::IsNotEmpty);
  // Non-empty triggers then_rule (async)
  assert!(rule.validate_ref_async("good").await.is_ok());
  assert!(rule.validate_ref_async("bad").await.is_err());
  // Empty bypasses then_rule
  assert!(rule.validate_ref_async("").await.is_ok());
}

// ---------------------------------------------------------------------------
// Mixed sync + async in All/Any
// ---------------------------------------------------------------------------

#[tokio::test]
async fn all_mixed_sync_and_async() {
  let rule = Rule::<String>::All(vec![
    Rule::Required,
    Rule::MinLength(2),
    async_str_is_not_banned("bad"),
  ]);
  assert!(rule.validate_ref_async("ok").await.is_ok());
  assert!(rule.validate_ref_async("").await.is_err()); // Required fails
  assert!(rule.validate_ref_async("bad").await.is_err()); // async fails
}

#[tokio::test]
async fn any_mixed_sync_and_async() {
  let rule = Rule::<String>::Any(vec![Rule::ExactLength(3), async_str_is_not_banned("bad")]);
  assert!(rule.validate_ref_async("hi").await.is_ok()); // fails length, passes async
  assert!(rule.validate_ref_async("bad").await.is_ok()); // passes length (3 chars)
}

// ---------------------------------------------------------------------------
// CustomAsync in sync context is skipped (returns Ok)
// ---------------------------------------------------------------------------

#[test]
fn custom_async_in_sync_context_is_skipped() {
  use walrs_validation::ValidateRef;
  let rule = async_str_is_not_banned("bad");
  // CustomAsync is skipped in sync context — always Ok
  assert!(rule.validate_ref("good").is_ok());
  assert!(rule.validate_ref("bad").is_ok());
}

#[test]
fn custom_async_numeric_in_sync_context_is_skipped() {
  use walrs_validation::Validate;
  let rule = async_is_even();
  // CustomAsync is skipped in sync context — always Ok
  assert!(rule.validate(4i64).is_ok());
  assert!(rule.validate(3i64).is_ok());
}

// ---------------------------------------------------------------------------
// When with async else_rule branch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn when_with_async_else_branch_string_passes() {
  // condition: IsNotEmpty, value "" → false → else_rule runs
  let rule = Rule::<String>::MinLength(3).when_else(
    walrs_validation::Condition::IsNotEmpty,
    async_str_is_not_banned("bad"),
  );
  // Empty string → condition false → else_rule: "bad" check → "" ≠ "bad" → Ok
  assert!(rule.validate_ref_async("").await.is_ok());
}

#[tokio::test]
async fn when_with_async_else_branch_string_with_equals_condition() {
  let rule = Rule::<String>::MinLength(100).when_else(
    walrs_validation::Condition::Equals("trigger".to_string()),
    async_str_is_not_banned("forbidden"),
  );
  // "hello" → condition false → else_rule: "hello" ≠ "forbidden" → Ok
  assert!(rule.validate_ref_async("hello").await.is_ok());
  // "forbidden" → condition false → else_rule: "forbidden" = "forbidden" → Err
  assert!(rule.validate_ref_async("forbidden").await.is_err());
  // "trigger" → condition true → then_rule: len 7 < 100 → Err
  assert!(rule.validate_ref_async("trigger").await.is_err());
}

#[tokio::test]
async fn when_with_async_else_branch_numeric() {
  let rule = Rule::<i64>::Max(200).when_else(
    walrs_validation::Condition::GreaterThan(100),
    async_is_even(),
  );
  // 50 (≤ 100, even) → else branch → Ok
  assert!(rule.validate_async(50i64).await.is_ok());
  // 51 (≤ 100, odd) → else branch → Err
  assert!(rule.validate_async(51i64).await.is_err());
  // 150 (> 100) → then branch: 150 ≤ 200 → Ok
  assert!(rule.validate_async(150i64).await.is_ok());
  // 250 (> 100, > 200) → then branch: 250 > 200 → Err
  assert!(rule.validate_async(250i64).await.is_err());
}

#[tokio::test]
async fn nested_async_else_in_all() {
  let when_rule = Rule::<String>::MinLength(100).when_else(
    walrs_validation::Condition::IsNotEmpty,
    async_str_is_not_banned("bad"),
  );
  let rule = Rule::<String>::All(vec![when_rule, Rule::MinLength(1)]);
  // "good" → condition true → then_rule: len 4 < 100 → Err (All short-circuits)
  assert!(rule.validate_ref_async("good").await.is_err());
  // "" → condition false → else_rule: "" ≠ "bad" → Ok, but MinLength(1) → Err
  assert!(rule.validate_ref_async("").await.is_err());
  // "ok" → condition true → then_rule: len 2 < 100 → Err
  assert!(rule.validate_ref_async("ok").await.is_err());
}

//! Integration tests for `#[derive(Fieldset)]` with `#[fieldset(async)]`.
//!
//! Exercises the async codegen: a sync rule (e.g. `email`) plus a
//! `custom_async = "..."` rule, then asserts both success and failure paths.

#![cfg(all(feature = "derive", feature = "async"))]

use walrs_fieldfilter::{DeriveFieldset, Fieldset, FieldsetAsync};
use walrs_validation::{ValidatorResult, Violation, ViolationType};

// Async validator used by the derive — must take the field's `&str`/`&T` view
// and return a `ValidatorResult`.
async fn check_unique_username(name: &str) -> ValidatorResult {
  // Pretend we hit a database here.
  if name == "taken" {
    Err(Violation::new(
      ViolationType::CustomError,
      "username already taken",
    ))
  } else {
    Ok(())
  }
}

async fn never_allowed(_value: &str) -> ValidatorResult {
  Err(Violation::new(
    ViolationType::CustomError,
    "this field is unconditionally rejected",
  ))
}

#[derive(Debug, DeriveFieldset)]
#[fieldset(async)]
struct Registration {
  #[validate(required, email)]
  #[filter(trim, lowercase)]
  email: String,

  #[validate(required, min_length = 3, custom_async = "check_unique_username")]
  username: String,
}

#[tokio::test]
async fn validate_async_passes_when_all_rules_pass() {
  let r = Registration {
    email: "user@example.com".into(),
    username: "alice".into(),
  };
  assert!(r.validate_async().await.is_ok());
}

#[tokio::test]
async fn validate_async_fails_on_async_rule() {
  let r = Registration {
    email: "user@example.com".into(),
    username: "taken".into(),
  };
  let err = r.validate_async().await.unwrap_err();
  assert!(err.get("username").is_some());
  // email passed, username failed only via the async rule
  assert!(err.get("email").is_none());
}

#[tokio::test]
async fn validate_async_fails_on_sync_rule_in_async_path() {
  let r = Registration {
    email: "not-an-email".into(),
    username: "alice".into(),
  };
  let err = r.validate_async().await.unwrap_err();
  assert!(err.get("email").is_some());
  assert!(err.get("username").is_none());
}

#[tokio::test]
async fn validate_async_collects_both_sync_and_async_failures() {
  let r = Registration {
    email: "".into(),
    username: "taken".into(),
  };
  let err = r.validate_async().await.unwrap_err();
  assert!(err.get("email").is_some());
  assert!(err.get("username").is_some());
}

#[tokio::test]
async fn filter_async_delegates_to_sync_filter() {
  let r = Registration {
    email: "  USER@EXAMPLE.COM  ".into(),
    username: "alice".into(),
  };
  let filtered = r.filter_async().await.unwrap();
  assert_eq!(filtered.email, "user@example.com");
}

#[tokio::test]
async fn clean_async_runs_filter_then_validate() {
  let r = Registration {
    email: "  USER@EXAMPLE.COM  ".into(),
    username: "alice".into(),
  };
  let cleaned = r.clean_async().await.unwrap();
  assert_eq!(cleaned.email, "user@example.com");
  assert_eq!(cleaned.username, "alice");
}

#[tokio::test]
async fn sync_fieldset_impl_still_works() {
  // The sync impl ignores `custom_async` rules entirely.
  let r = Registration {
    email: "user@example.com".into(),
    username: "taken".into(),
  };
  // Sync validate ignores the async rule -> passes.
  assert!(r.validate().is_ok());
}

// --- Single-rule async-only field ---

#[derive(Debug, DeriveFieldset)]
#[fieldset(async)]
struct OnlyAsync {
  #[validate(custom_async = "never_allowed")]
  token: String,
}

#[tokio::test]
async fn async_only_field_runs_in_async_path() {
  let v = OnlyAsync {
    token: "anything".into(),
  };
  let err = v.validate_async().await.unwrap_err();
  assert!(err.get("token").is_some());
}

#[tokio::test]
async fn async_only_field_is_noop_in_sync_path() {
  let v = OnlyAsync {
    token: "anything".into(),
  };
  // Sync codegen has no rules for `token` -> passes.
  assert!(v.validate().is_ok());
}

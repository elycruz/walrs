//! `#[derive(Fieldset)]` with `#[fieldset(async)]` — emits both the sync
//! `Fieldset` impl and an async `FieldsetAsync` impl. The async impl honors
//! `custom_async = "..."` validators.
//!
//! Run with: `cargo run --example derive_async --features derive,async -p walrs_fieldfilter`

use walrs_fieldfilter::{DeriveFieldset, FieldsetAsync};
use walrs_validation::{ValidatorResult, Violation, ViolationType};

async fn check_unique_username(name: &str) -> ValidatorResult {
  // Pretend this is a database lookup.
  if name == "taken" {
    Err(Violation::new(
      ViolationType::CustomError,
      "username already taken",
    ))
  } else {
    Ok(())
  }
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
  let ok = Registration {
    email: "  USER@EXAMPLE.COM  ".into(),
    username: "alice".into(),
  };
  match ok.clean_async().await {
    Ok(cleaned) => println!("ok: email={} username={}", cleaned.email, cleaned.username),
    Err(violations) => {
      eprintln!("unexpected validation failure:");
      for (field, fv) in violations.iter() {
        for v in fv.0.iter() {
          eprintln!("  {field}: {}", v.message());
        }
      }
    }
  }

  let bad = Registration {
    email: "user@example.com".into(),
    username: "taken".into(),
  };
  match bad.validate_async().await {
    Ok(_) => println!("unexpected pass"),
    Err(violations) => {
      println!("expected async failure:");
      for (field, fv) in violations.iter() {
        for v in fv.0.iter() {
          println!("  {field}: {}", v.message());
        }
      }
    }
  }
}

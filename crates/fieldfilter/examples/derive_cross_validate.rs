//! Demonstrates structured `#[cross_validate(...)]` variants on
//! `#[derive(Fieldset)]`.
//!
//! Run with:
//! ```bash
//! cargo run --example derive_cross_validate -p walrs_fieldfilter --features derive
//! ```

use walrs_fieldfilter::{DeriveFieldset, Fieldset};

#[derive(Debug, DeriveFieldset)]
#[cross_validate(fields_equal(password, confirm))]
#[cross_validate(required_if(shipping_street, country = "us"))]
struct Checkout {
  #[validate(required, min_length = 8)]
  password: String,
  #[validate(required)]
  confirm: String,
  #[validate(required)]
  country: String,
  shipping_street: Option<String>,
}

fn report(label: &str, form: &Checkout) {
  match form.validate() {
    Ok(()) => println!("[{label}] OK"),
    Err(v) => {
      println!("[{label}] {} violation(s):", v.len());
      for (field, vs) in v.iter() {
        let key = if field.is_empty() { "<form>" } else { field };
        for violation in vs.iter() {
          println!("  - {key}: {}", violation.message());
        }
      }
    }
  }
}

fn main() {
  // Happy path: passwords match; country is non-US, so shipping not required.
  report(
    "happy",
    &Checkout {
      password: "supersecret".into(),
      confirm: "supersecret".into(),
      country: "ca".into(),
      shipping_street: None,
    },
  );

  // fields_equal violation.
  report(
    "mismatch",
    &Checkout {
      password: "supersecret".into(),
      confirm: "different!".into(),
      country: "ca".into(),
      shipping_street: None,
    },
  );

  // required_if violation: country = "us" but shipping_street missing.
  report(
    "missing-shipping",
    &Checkout {
      password: "supersecret".into(),
      confirm: "supersecret".into(),
      country: "us".into(),
      shipping_street: None,
    },
  );
}

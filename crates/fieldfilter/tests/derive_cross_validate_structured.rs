//! Integration tests for structured `#[cross_validate(...)]` variants on
//! `#[derive(Fieldset)]`.

#![cfg(feature = "derive")]

use walrs_fieldfilter::{DeriveFieldset, Fieldset};

// =========================================================================
// fields_equal
// =========================================================================

#[derive(Debug, DeriveFieldset)]
#[cross_validate(fields_equal(password, confirm))]
struct PwForm {
  password: String,
  confirm: String,
}

#[test]
fn fields_equal_pass() {
  let form = PwForm {
    password: "hunter2".into(),
    confirm: "hunter2".into(),
  };
  assert!(form.validate().is_ok());
}

#[test]
fn fields_equal_fail() {
  let form = PwForm {
    password: "hunter2".into(),
    confirm: "different".into(),
  };
  let err = form.validate().unwrap_err();
  let form_v = err.form_violations().expect("form violations expected");
  assert!(form_v[0].message().contains("password"));
  assert!(form_v[0].message().contains("confirm"));
}

// =========================================================================
// required_if (string condition)
// =========================================================================

#[derive(Debug, DeriveFieldset)]
#[cross_validate(required_if(shipping_street, country = "us"))]
struct CheckoutStr {
  country: String,
  shipping_street: Option<String>,
}

#[test]
fn required_if_str_condition_met_field_present() {
  let form = CheckoutStr {
    country: "us".into(),
    shipping_street: Some("123 Main".into()),
  };
  assert!(form.validate().is_ok());
}

#[test]
fn required_if_str_condition_met_field_missing() {
  let form = CheckoutStr {
    country: "us".into(),
    shipping_street: None,
  };
  let err = form.validate().unwrap_err();
  assert!(err.form_violations().is_some());
}

#[test]
fn required_if_str_condition_not_met() {
  let form = CheckoutStr {
    country: "ca".into(),
    shipping_street: None,
  };
  assert!(form.validate().is_ok());
}

// =========================================================================
// required_unless (bool condition)
// =========================================================================

#[derive(Debug, DeriveFieldset)]
#[cross_validate(required_unless(billing, same_as_shipping = true))]
struct CheckoutBool {
  same_as_shipping: bool,
  billing: Option<String>,
}

#[test]
fn required_unless_bool_condition_met_skips_check() {
  let form = CheckoutBool {
    same_as_shipping: true,
    billing: None,
  };
  assert!(form.validate().is_ok());
}

#[test]
fn required_unless_bool_condition_not_met_field_missing() {
  let form = CheckoutBool {
    same_as_shipping: false,
    billing: None,
  };
  let err = form.validate().unwrap_err();
  assert!(err.form_violations().is_some());
}

#[test]
fn required_unless_bool_condition_not_met_field_present() {
  let form = CheckoutBool {
    same_as_shipping: false,
    billing: Some("123 Main".into()),
  };
  assert!(form.validate().is_ok());
}

// =========================================================================
// one_of_required
// =========================================================================

#[derive(Debug, DeriveFieldset)]
#[cross_validate(one_of_required(email, phone))]
struct Contact {
  email: Option<String>,
  phone: Option<String>,
}

#[test]
fn one_of_required_pass_email() {
  let form = Contact {
    email: Some("a@b.c".into()),
    phone: None,
  };
  assert!(form.validate().is_ok());
}

#[test]
fn one_of_required_pass_phone() {
  let form = Contact {
    email: None,
    phone: Some("555".into()),
  };
  assert!(form.validate().is_ok());
}

#[test]
fn one_of_required_fail_neither() {
  let form = Contact {
    email: None,
    phone: None,
  };
  let err = form.validate().unwrap_err();
  assert!(err.form_violations().is_some());
}

// =========================================================================
// mutually_exclusive
// =========================================================================

#[derive(Debug, DeriveFieldset)]
#[cross_validate(mutually_exclusive(credit_card, paypal))]
struct Payment {
  credit_card: Option<String>,
  paypal: Option<String>,
}

#[test]
fn mutually_exclusive_pass_neither() {
  let form = Payment {
    credit_card: None,
    paypal: None,
  };
  assert!(form.validate().is_ok());
}

#[test]
fn mutually_exclusive_pass_one() {
  let form = Payment {
    credit_card: Some("1234".into()),
    paypal: None,
  };
  assert!(form.validate().is_ok());
}

#[test]
fn mutually_exclusive_fail_both() {
  let form = Payment {
    credit_card: Some("1234".into()),
    paypal: Some("a@b.c".into()),
  };
  let err = form.validate().unwrap_err();
  assert!(err.form_violations().is_some());
}

// =========================================================================
// dependent_required
// =========================================================================

#[derive(Debug, DeriveFieldset)]
#[cross_validate(dependent_required(trigger = ship, dependents(street, zip)))]
struct ShipForm {
  ship: Option<String>,
  street: Option<String>,
  zip: Option<String>,
}

#[test]
fn dependent_required_pass_no_trigger() {
  let form = ShipForm {
    ship: None,
    street: None,
    zip: None,
  };
  assert!(form.validate().is_ok());
}

#[test]
fn dependent_required_pass_trigger_with_deps() {
  let form = ShipForm {
    ship: Some("yes".into()),
    street: Some("123 Main".into()),
    zip: Some("90210".into()),
  };
  assert!(form.validate().is_ok());
}

#[test]
fn dependent_required_fail_trigger_missing_deps() {
  let form = ShipForm {
    ship: Some("yes".into()),
    street: None,
    zip: None,
  };
  let err = form.validate().unwrap_err();
  let form_v = err.form_violations().expect("expected dependents missing");
  // both `street` and `zip` should be reported
  assert_eq!(form_v.len(), 2);
}

// =========================================================================
// integer condition
// =========================================================================

#[derive(Debug, DeriveFieldset)]
#[cross_validate(required_if(license, age = 18))]
struct AgeForm {
  age: i32,
  license: Option<String>,
}

#[test]
fn required_if_int_condition_match() {
  let form = AgeForm {
    age: 18,
    license: None,
  };
  let err = form.validate().unwrap_err();
  assert!(err.form_violations().is_some());
}

#[test]
fn required_if_int_condition_mismatch() {
  let form = AgeForm {
    age: 21,
    license: None,
  };
  assert!(form.validate().is_ok());
}

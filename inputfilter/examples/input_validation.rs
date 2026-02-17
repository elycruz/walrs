//! Example demonstrating the `Input` and `RefInput` structs for
//! validating and filtering values.
//!
//! Run with: `cargo run -p walrs_inputfilter --example input_validation`

use std::borrow::Cow;
use walrs_inputfilter::{
  FilterForSized, FilterForUnsized, InputBuilder, RefInputBuilder, Violation,
  ViolationType::TypeMismatch,
};

fn main() {
  // ----- Input (for Copy/Sized types) -----
  println!("=== Input (Copy/Sized types) ===\n");

  // Validator: even numbers only
  let even_validator = |x: usize| {
    if x.is_multiple_of(2) {
      Ok(())
    } else {
      Err(Violation(
        TypeMismatch,
        format!("{} is not even", x),
      ))
    }
  };

  // Filter: double the value
  let double_filter = |x: usize| x * 2;

  let input = InputBuilder::<usize, usize>::default()
    .required(true)
    .name("even_number")
    .validators(vec![&even_validator])
    .filters(vec![&double_filter])
    .build()
    .unwrap();

  // Test valid value
  match input.filter(4) {
    Ok(result) => println!("filter(4) = {} (valid, doubled)", result),
    Err(errors) => println!("filter(4) errors: {:?}", errors),
  }

  // Test invalid value
  match input.filter(3) {
    Ok(result) => println!("filter(3) = {} (valid, doubled)", result),
    Err(errors) => println!("filter(3) errors: {:?}", errors),
  }

  // Using as a function (Fn trait)
  match input(6) {
    Ok(result) => println!("input(6) = {} (called as function)", result),
    Err(errors) => println!("input(6) errors: {:?}", errors),
  }

  // Optional value handling
  match input.filter_option(None) {
    Ok(result) => println!("filter_option(None) = {:?}", result),
    Err(errors) => println!("filter_option(None) errors: {:?} (required field)", errors),
  }

  // ----- RefInput (for unsized/reference types) -----
  println!("\n=== RefInput (Unsized/Reference types) ===\n");

  // Validator: min length
  let min_length_validator = |s: &str| {
    if s.len() >= 3 {
      Ok(())
    } else {
      Err(Violation(
        TypeMismatch,
        format!("'{}' is too short (min 3 chars)", s),
      ))
    }
  };

  // Filter: uppercase
  let uppercase_filter = |s: Cow<str>| -> Cow<str> { s.to_uppercase().into() };

  let str_input = RefInputBuilder::<str, Cow<str>>::default()
    .required(true)
    .name("username")
    .validators(vec![&min_length_validator])
    .filters(vec![&uppercase_filter])
    .build()
    .unwrap();

  // Test valid value
  match str_input.filter_ref("hello") {
    Ok(result) => println!("filter_ref(\"hello\") = \"{}\" (valid, uppercased)", result),
    Err(errors) => println!("filter_ref(\"hello\") errors: {:?}", errors),
  }

  // Test invalid value
  match str_input.filter_ref("hi") {
    Ok(result) => println!("filter_ref(\"hi\") = \"{}\"", result),
    Err(errors) => println!("filter_ref(\"hi\") errors: {:?}", errors),
  }

  // Using as a function (Fn trait)
  match str_input("world") {
    Ok(result) => println!("str_input(\"world\") = \"{}\" (called as function)", result),
    Err(errors) => println!("str_input(\"world\") errors: {:?}", errors),
  }

  // Optional value handling
  match str_input.filter_ref_option(None) {
    Ok(result) => println!("filter_ref_option(None) = {:?}", result),
    Err(errors) => println!("filter_ref_option(None) errors: {:?} (required field)", errors),
  }

  // ----- Serialization -----
  println!("\n=== Serialization ===\n");

  let json = serde_json::to_string_pretty(&input).unwrap();
  println!("Input serialized to JSON:\n{}", json);

  let json = serde_json::to_string_pretty(&str_input).unwrap();
  println!("\nRefInput serialized to JSON:\n{}", json);

  // ----- ToAttributesList -----
  println!("\n=== HTML Form Attributes ===\n");

  use walrs_inputfilter::traits::ToAttributesList;

  if let Some(attrs) = input.to_attributes_list() {
    println!("Input attributes: {:?}", attrs);
  }

  if let Some(attrs) = str_input.to_attributes_list() {
    println!("RefInput attributes: {:?}", attrs);
  }
}

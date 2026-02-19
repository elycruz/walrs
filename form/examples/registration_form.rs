//! Example: Creating a registration form with fieldsets
//!
//! This example demonstrates how to create a complex registration form
//! with grouped fields using fieldsets.
use walrs_form::{
  ButtonElement, ButtonType, Element, FieldsetElement, Form, FormMethod, InputElement, InputType,
  SelectElement, SelectOption, TextareaElement,
};
fn main() {
  // Create the form
  let mut form = Form::new("registration");
  form.action = Some("/api/register".to_string());
  form.method = Some(FormMethod::Post);
  // Personal Information fieldset
  let mut personal_info = FieldsetElement::with_legend("Personal Information");
  let mut first_name = InputElement::new("first_name", InputType::Text);
  first_name.label = Some("First Name".to_string());
  first_name.required = Some(true);
  personal_info.add_element(first_name.into());
  let mut last_name = InputElement::new("last_name", InputType::Text);
  last_name.label = Some("Last Name".to_string());
  last_name.required = Some(true);
  personal_info.add_element(last_name.into());
  let mut email = InputElement::new("email", InputType::Email);
  email.label = Some("Email Address".to_string());
  email.required = Some(true);
  personal_info.add_element(email.into());
  form.add_element(personal_info.into());
  // Account Information fieldset
  let mut account_info = FieldsetElement::with_legend("Account Information");
  let mut username = InputElement::new("username", InputType::Text);
  username.label = Some("Username".to_string());
  username.required = Some(true);
  account_info.add_element(username.into());
  let mut password = InputElement::new("password", InputType::Password);
  password.label = Some("Password".to_string());
  password.required = Some(true);
  account_info.add_element(password.into());
  let mut confirm_password = InputElement::new("confirm_password", InputType::Password);
  confirm_password.label = Some("Confirm Password".to_string());
  confirm_password.required = Some(true);
  account_info.add_element(confirm_password.into());
  form.add_element(account_info.into());
  // Preferences fieldset
  let mut preferences = FieldsetElement::with_legend("Preferences");
  let mut country = SelectElement::new("country");
  country.label = Some("Country".to_string());
  country.add_option(SelectOption::new("", "Select a country..."));
  country.add_option(SelectOption::new("us", "United States"));
  country.add_option(SelectOption::new("ca", "Canada"));
  country.add_option(SelectOption::new("uk", "United Kingdom"));
  country.add_option(SelectOption::new("au", "Australia"));
  preferences.add_element(country.into());
  let mut bio = TextareaElement::with_size("bio", 4, 50);
  bio.label = Some("Tell us about yourself".to_string());
  preferences.add_element(bio.into());
  form.add_element(preferences.into());
  // Submit button
  let submit = ButtonElement::with_label("Create Account", ButtonType::Submit);
  form.add_element(submit.into());
  // Display form structure
  println!("Registration Form Structure");
  println!("===========================\n");
  for element in form.iter_elements() {
    match element {
      Element::Fieldset(fieldset) => {
        println!("Fieldset: {:?}", fieldset.legend);
        for child in fieldset.iter_elements() {
          print_element(child, 1);
        }
        println!();
      }
      _ => print_element(element, 0),
    }
  }
  // Serialize to JSON
  println!("\nJSON Output:");
  let json = serde_json::to_string_pretty(&form).unwrap();
  println!("{}", json);
}
fn print_element(element: &Element, indent: usize) {
  let prefix = "  ".repeat(indent + 1);
  match element {
    Element::Input(input) => {
      println!(
        "{}Input: {:?} (type: {:?}, required: {:?})",
        prefix, input.name, input._type, input.required
      );
    }
    Element::Select(select) => {
      println!(
        "{}Select: {:?} ({} options)",
        prefix,
        select.name,
        select.options.len()
      );
    }
    Element::Textarea(textarea) => {
      println!(
        "{}Textarea: {:?} (rows: {:?})",
        prefix, textarea.name, textarea.rows
      );
    }
    Element::Button(button) => {
      println!(
        "{}Button: {:?} (type: {:?})",
        prefix, button.label, button._type
      );
    }
    _ => {}
  }
}

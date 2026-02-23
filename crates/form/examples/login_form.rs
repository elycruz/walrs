//! Example: Creating a login form
//!
//! This example demonstrates how to create a simple login form
//! with username, password, and remember me checkbox.
use walrs_form::{
  ButtonElement, ButtonType, Element, Form, FormData, FormMethod, InputElement, InputType,
};
use walrs_validation::Value;
fn main() {
  // Create the form
  let mut form = Form::new("login");
  form.action = Some("/api/login".to_string());
  form.method = Some(FormMethod::Post);
  // Add username field
  let mut username = InputElement::new("username", InputType::Text);
  username.label = Some("Username".to_string());
  username.required = Some(true);
  form.add_element(username.into());
  // Add password field
  let mut password = InputElement::new("password", InputType::Password);
  password.label = Some("Password".to_string());
  password.required = Some(true);
  form.add_element(password.into());
  // Add remember me checkbox
  let mut remember = InputElement::new("remember", InputType::Checkbox);
  remember.label = Some("Remember me".to_string());
  form.add_element(remember.into());
  // Add submit button
  let submit = ButtonElement::with_label("Sign In", ButtonType::Submit);
  form.add_element(submit.into());
  // Display form structure
  println!("Form: {:?}", form.name);
  println!("Action: {:?}", form.action);
  println!("Method: {:?}", form.method);
  println!("\nElements:");
  for element in form.iter_elements() {
    match element {
      Element::Input(input) => {
        println!("  - Input: {:?} (type: {:?})", input.name, input._type);
      }
      Element::Button(button) => {
        println!("  - Button: {:?}", button.label);
      }
      _ => {}
    }
  }
  // Bind some data
  let mut data = FormData::new();
  data.insert("username", Value::from("john_doe"));
  data.insert("remember", Value::from(true));
  form.bind_data(data);
  // Serialize to JSON
  let json_output = serde_json::to_string_pretty(&form).unwrap();
  println!("\nJSON Output:\n{}", json_output);
}

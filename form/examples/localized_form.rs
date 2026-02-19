//! Example: Localized Form Validation
//!
//! This example demonstrates how to create forms with internationalized
//! validation error messages that change based on user locale.
//!
//! Run with: `cargo run --example localized_form -p walrs_form`

use walrs_form::{
  ButtonElement, ButtonType, Element, Form, FormData, FormMethod, InputElement, InputType,
};
use walrs_validator::Rule;

/// A simple struct to represent locale-aware validation results
struct LocalizedFormValidator {
  locale: Option<String>,
}

impl LocalizedFormValidator {
  fn new(locale: Option<&str>) -> Self {
    Self {
      locale: locale.map(|s| s.to_string()),
    }
  }

  /// Validate username with localized error messages
  fn validate_username(&self, value: &str) -> Result<(), String> {
    let rule = Rule::<String>::Required
      .with_message_provider(|ctx| match ctx.locale {
        Some("es") => "El nombre de usuario es obligatorio".to_string(),
        Some("fr") => "Le nom d'utilisateur est requis".to_string(),
        Some("de") => "Benutzername ist erforderlich".to_string(),
        _ => "Username is required".to_string(),
      })
      .and(
        Rule::<String>::MinLength(3)
          .and(Rule::MaxLength(20))
          .with_message_provider(|ctx| {
            let (min, max) = (3, 20);
            match ctx.locale {
              Some("es") => format!(
                "El nombre de usuario debe tener entre {} y {} caracteres",
                min, max
              ),
              Some("fr") => format!(
                "Le nom d'utilisateur doit contenir entre {} et {} caractères",
                min, max
              ),
              Some("de") => format!(
                "Benutzername muss zwischen {} und {} Zeichen haben",
                min, max
              ),
              _ => format!("Username must be between {} and {} characters", min, max),
            }
          }),
      );

    rule
      .validate_ref(value, self.locale.as_deref())
      .map_err(|v| v.message().to_string())
  }

  /// Validate email with localized error messages
  fn validate_email(&self, value: &str) -> Result<(), String> {
    let rule = Rule::<String>::Required
      .with_message_provider(|ctx| match ctx.locale {
        Some("es") => "El correo electrónico es obligatorio".to_string(),
        Some("fr") => "L'adresse e-mail est requise".to_string(),
        Some("de") => "E-Mail ist erforderlich".to_string(),
        _ => "Email is required".to_string(),
      })
      .and(
        Rule::<String>::Email.with_message_provider(|ctx| match ctx.locale {
          Some("es") => "El formato del correo electrónico no es válido".to_string(),
          Some("fr") => "Le format de l'adresse e-mail est invalide".to_string(),
          Some("de") => "Ungültiges E-Mail-Format".to_string(),
          _ => "Invalid email format".to_string(),
        }),
      );

    rule
      .validate_ref(value, self.locale.as_deref())
      .map_err(|v| v.message().to_string())
  }

  /// Validate password with localized error messages
  fn validate_password(&self, value: &str) -> Result<(), Vec<String>> {
    let rule = Rule::<String>::Required
      .with_message_provider(|ctx| match ctx.locale {
        Some("es") => "La contraseña es obligatoria".to_string(),
        Some("fr") => "Le mot de passe est requis".to_string(),
        Some("de") => "Passwort ist erforderlich".to_string(),
        _ => "Password is required".to_string(),
      })
      .and(Rule::<String>::MinLength(8).with_message_provider(|ctx| {
        let min = 8;
        match ctx.locale {
          Some("es") => {
            format!("La contraseña debe tener al menos {} caracteres", min)
          }
          Some("fr") => {
            format!("Le mot de passe doit contenir au moins {} caractères", min)
          }
          Some("de") => format!("Passwort muss mindestens {} Zeichen haben", min),
          _ => format!("Password must be at least {} characters", min),
        }
      }))
      .and(
        Rule::<String>::Pattern(r"[A-Z]".to_string()).with_message_provider(|ctx| {
          match ctx.locale {
            Some("es") => "La contraseña debe contener al menos una letra mayúscula".to_string(),
            Some("fr") => "Le mot de passe doit contenir au moins une lettre majuscule".to_string(),
            Some("de") => "Passwort muss mindestens einen Großbuchstaben enthalten".to_string(),
            _ => "Password must contain at least one uppercase letter".to_string(),
          }
        }),
      )
      .and(
        Rule::<String>::Pattern(r"[0-9]".to_string()).with_message_provider(|ctx| {
          match ctx.locale {
            Some("es") => "La contraseña debe contener al menos un número".to_string(),
            Some("fr") => "Le mot de passe doit contenir au moins un chiffre".to_string(),
            Some("de") => "Passwort muss mindestens eine Zahl enthalten".to_string(),
            _ => "Password must contain at least one number".to_string(),
          }
        }),
      );

    rule
      .validate_ref_all(value, self.locale.as_deref())
      .map_err(|violations| violations.iter().map(|v| v.message().to_string()).collect())
  }
}

fn main() {
  println!("=== Localized Form Validation Example ===\n");

  // ========================================================================
  // Create a registration form
  // ========================================================================
  let mut form = Form::new("registration");
  form.action = Some("/api/register".to_string());
  form.method = Some(FormMethod::Post);

  // Add form elements
  let mut username = InputElement::new("username", InputType::Text);
  username.label = Some("Username".to_string());
  username.required = Some(true);
  form.add_element(username.into());

  let mut email = InputElement::new("email", InputType::Email);
  email.label = Some("Email".to_string());
  email.required = Some(true);
  form.add_element(email.into());

  let mut password = InputElement::new("password", InputType::Password);
  password.label = Some("Password".to_string());
  password.required = Some(true);
  form.add_element(password.into());

  let submit = ButtonElement::with_label("Register", ButtonType::Submit);
  form.add_element(submit.into());

  println!("Form: {}", form.name.as_deref().unwrap_or("unnamed"));
  println!("Elements:");
  for element in form.iter_elements() {
    if let Element::Input(input) = element {
      println!(
        "  - {} (type: {:?}, required: {:?})",
        input.name.as_deref().unwrap_or("unnamed"),
        input._type,
        input.required
      );
    }
  }

  // ========================================================================
  // Simulate form submission with invalid data
  // ========================================================================
  println!("\n--- Simulating Form Submission with Invalid Data ---\n");

  let mut data = FormData::new();
  data.insert("username", serde_json::json!("ab")); // Too short
  data.insert("email", serde_json::json!("not-an-email")); // Invalid format
  data.insert("password", serde_json::json!("weak")); // Too short, no uppercase, no number

  // Test validation in different locales
  let locales = [
    (None, "English (default)"),
    (Some("es"), "Spanish"),
    (Some("fr"), "French"),
    (Some("de"), "German"),
  ];

  for (locale, locale_name) in locales {
    println!("=== Validation Errors in {} ===\n", locale_name);

    let validator = LocalizedFormValidator::new(locale);

    // Validate username
    let username_value = data.get("username").and_then(|v| v.as_str()).unwrap_or("");
    print!("Username \"{}\": ", username_value);
    match validator.validate_username(username_value) {
      Ok(()) => println!("OK"),
      Err(msg) => println!("ERROR - {}", msg),
    }

    // Validate email
    let email_value = data.get("email").and_then(|v| v.as_str()).unwrap_or("");
    print!("Email \"{}\": ", email_value);
    match validator.validate_email(email_value) {
      Ok(()) => println!("OK"),
      Err(msg) => println!("ERROR - {}", msg),
    }

    // Validate password (collect all errors)
    let password_value = data.get("password").and_then(|v| v.as_str()).unwrap_or("");
    println!("Password \"{}\":", password_value);
    match validator.validate_password(password_value) {
      Ok(()) => println!("  OK"),
      Err(errors) => {
        for error in errors {
          println!("  - {}", error);
        }
      }
    }

    println!();
  }

  // ========================================================================
  // Example with valid data
  // ========================================================================
  println!("=== Validation with Valid Data (Spanish) ===\n");

  let mut valid_data = FormData::new();
  valid_data.insert("username", serde_json::json!("john_doe"));
  valid_data.insert("email", serde_json::json!("john@example.com"));
  valid_data.insert("password", serde_json::json!("SecurePass123"));

  let validator = LocalizedFormValidator::new(Some("es"));

  let username_value = valid_data
    .get("username")
    .and_then(|v| v.as_str())
    .unwrap_or("");
  print!("Username \"{}\": ", username_value);
  match validator.validate_username(username_value) {
    Ok(()) => println!("OK"),
    Err(msg) => println!("ERROR - {}", msg),
  }

  let email_value = valid_data
    .get("email")
    .and_then(|v| v.as_str())
    .unwrap_or("");
  print!("Email \"{}\": ", email_value);
  match validator.validate_email(email_value) {
    Ok(()) => println!("OK"),
    Err(msg) => println!("ERROR - {}", msg),
  }

  let password_value = valid_data
    .get("password")
    .and_then(|v| v.as_str())
    .unwrap_or("");
  print!("Password \"{}\": ", password_value);
  match validator.validate_password(password_value) {
    Ok(()) => println!("OK"),
    Err(errors) => {
      for error in errors {
        println!("  - {}", error);
      }
    }
  }

  println!("\n=== Example Complete ===");
}

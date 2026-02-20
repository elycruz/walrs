//! Example: Localized Validation Error Messages
//!
//! This example demonstrates how to create validation rules with
//! internationalized (i18n) error messages that change based on locale.
//!
//! Run with: `cargo run --example localized_messages -p walrs_inputfilter`

use walrs_inputfilter::field::FieldBuilder;
use walrs_validator::Rule;

fn main() {
  println!("=== Localized Validation Messages Example ===\n");

  // ========================================================================
  // Example 1: Basic Locale-Aware Field Validation
  // ========================================================================
  println!("--- Example 1: Basic Locale-Aware Field ---\n");

  // Create a username field with a localized error message
  // Note: We hardcode the min/max values in the closure since MessageParams
  // may not contain them depending on how the rule is composed.
  let username_rule = Rule::<String>::MinLength(3)
    .and(Rule::MaxLength(20))
    .with_message_provider(|ctx| {
      let (min, max) = (3, 20); // Hardcode the constraint values
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
          "Der Benutzername muss zwischen {} und {} Zeichen lang sein",
          min, max
        ),
        _ => format!("Username must be between {} and {} characters", min, max),
      }
    });

  // Test with different locales
  let test_value = "ab".to_string(); // Too short

  // English (default)
  let field_en = FieldBuilder::<String>::default()
    .name("username".to_string())
    .rule(username_rule.clone())
    .build()
    .unwrap();

  println!("Value: \"{}\"", test_value);
  println!("Locale: en (default)");
  if let Err(violations) = field_en.validate(&test_value) {
    for v in violations.iter() {
      println!("  Error: {}", v.message());
    }
  }

  // Spanish
  let field_es = FieldBuilder::<String>::default()
    .name("username".to_string())
    .locale("es".to_string())
    .rule(username_rule.clone())
    .build()
    .unwrap();

  println!("\nLocale: es");
  if let Err(violations) = field_es.validate(&test_value) {
    for v in violations.iter() {
      println!("  Error: {}", v.message());
    }
  }

  // French
  let field_fr = FieldBuilder::<String>::default()
    .name("username".to_string())
    .locale("fr".to_string())
    .rule(username_rule.clone())
    .build()
    .unwrap();

  println!("\nLocale: fr");
  if let Err(violations) = field_fr.validate(&test_value) {
    for v in violations.iter() {
      println!("  Error: {}", v.message());
    }
  }

  // German
  let field_de = FieldBuilder::<String>::default()
    .name("username".to_string())
    .locale("de".to_string())
    .rule(username_rule)
    .build()
    .unwrap();

  println!("\nLocale: de");
  if let Err(violations) = field_de.validate(&test_value) {
    for v in violations.iter() {
      println!("  Error: {}", v.message());
    }
  }

  // ========================================================================
  // Example 2: Multiple Rules with Different Localized Messages
  // ========================================================================
  println!("\n--- Example 2: Password Field with Multiple Rules ---\n");

  let password_rule = Rule::<String>::Required
    .with_message_provider(|ctx| match ctx.locale {
      Some("es") => "La contraseña es obligatoria".to_string(),
      Some("fr") => "Le mot de passe est requis".to_string(),
      _ => "Password is required".to_string(),
    })
    .and(Rule::<String>::MinLength(8).with_message_provider(|ctx| {
      // Hardcode constraint value for the message
      let min = 8;
      match ctx.locale {
        Some("es") => format!("La contraseña debe tener al menos {} caracteres", min),
        Some("fr") => format!("Le mot de passe doit contenir au moins {} caractères", min),
        _ => format!("Password must be at least {} characters", min),
      }
    }))
    .and(
      Rule::<String>::Pattern(r"[A-Z]".to_string()).with_message_provider(|ctx| match ctx.locale {
        Some("es") => "La contraseña debe contener al menos una letra mayúscula".to_string(),
        Some("fr") => "Le mot de passe doit contenir au moins une lettre majuscule".to_string(),
        _ => "Password must contain at least one uppercase letter".to_string(),
      }),
    )
    .and(
      Rule::<String>::Pattern(r"[0-9]".to_string()).with_message_provider(|ctx| match ctx.locale {
        Some("es") => "La contraseña debe contener al menos un número".to_string(),
        Some("fr") => "Le mot de passe doit contenir au moins un chiffre".to_string(),
        _ => "Password must contain at least one number".to_string(),
      }),
    );

  // Test with a weak password
  let weak_password = "abc".to_string();

  let password_field_es = FieldBuilder::<String>::default()
    .name("password".to_string())
    .locale("es".to_string())
    .rule(password_rule.clone())
    .build()
    .unwrap();

  println!("Value: \"{}\" (weak password)", weak_password);
  println!("Locale: es");
  if let Err(violations) = password_field_es.validate(&weak_password) {
    println!("  Violations ({}):", violations.len());
    for v in violations.iter() {
      println!("    - {}", v.message());
    }
  }

  let password_field_fr = FieldBuilder::<String>::default()
    .name("password".to_string())
    .locale("fr".to_string())
    .rule(password_rule)
    .build()
    .unwrap();

  println!("\nLocale: fr");
  if let Err(violations) = password_field_fr.validate(&weak_password) {
    println!("  Violations ({}):", violations.len());
    for v in violations.iter() {
      println!("    - {}", v.message());
    }
  }

  // ========================================================================
  // Example 3: Using a Translation Helper Function
  // ========================================================================
  println!("\n--- Example 3: Translation Helper Pattern ---\n");

  // Define a simple translation function (in real apps, this could use gettext, fluent, etc.)
  fn translate(key: &str, locale: Option<&str>) -> String {
    match (key, locale) {
      ("email.required", Some("es")) => "El correo electrónico es obligatorio".to_string(),
      ("email.required", Some("fr")) => "L'adresse e-mail est requise".to_string(),
      ("email.required", _) => "Email is required".to_string(),

      ("email.invalid", Some("es")) => "El formato del correo electrónico no es válido".to_string(),
      ("email.invalid", Some("fr")) => "Le format de l'adresse e-mail est invalide".to_string(),
      ("email.invalid", _) => "Invalid email format".to_string(),

      _ => format!("[Missing translation: {}]", key),
    }
  }

  let email_rule = Rule::<String>::Required
    .with_message_provider(|ctx| translate("email.required", ctx.locale))
    .and(Rule::<String>::Email.with_message_provider(|ctx| translate("email.invalid", ctx.locale)));

  let invalid_email = "not-an-email".to_string();

  for locale in [None, Some("es"), Some("fr")] {
    let mut builder = FieldBuilder::<String>::default();
    builder.name("email".to_string());
    builder.rule(email_rule.clone());
    if let Some(loc) = locale {
      builder.locale(loc.to_string());
    }
    let field = builder.build().unwrap();

    println!("Value: \"{}\"", invalid_email);
    println!("Locale: {:?}", locale.unwrap_or("en (default)"));
    if let Err(violations) = field.validate(&invalid_email) {
      for v in violations.iter() {
        println!("  Error: {}", v.message());
      }
    }
    println!();
  }

  // ========================================================================
  // Example 4: Runtime Locale Switching
  // ========================================================================
  println!("--- Example 4: Runtime Locale Switching ---\n");

  // Sometimes you need to validate with different locales at runtime
  // without creating new fields. You can use Rule::validate_str directly with locale.

  let age_rule =
    Rule::<String>::Pattern(r"^\d+$".to_string()).with_message_provider(|ctx| match ctx.locale {
      Some("es") => format!("'{}' no es un número válido", ctx.value),
      Some("fr") => format!("'{}' n'est pas un nombre valide", ctx.value),
      _ => format!("'{}' is not a valid number", ctx.value),
    });

  let invalid_age = "twenty";

  println!("Value: \"{}\"", invalid_age);
  for locale in [None, Some("es"), Some("fr")] {
    let result = age_rule.validate_str(invalid_age, locale);
    if let Err(violation) = result {
      println!(
        "  Locale {:?}: {}",
        locale.unwrap_or("en"),
        violation.message()
      );
    }
  }

  println!("\n=== Example Complete ===");
}

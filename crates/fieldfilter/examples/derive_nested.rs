//! Nested struct derive(Fieldset) example.

use walrs_fieldfilter::{DeriveFieldset, Fieldset};

#[derive(Debug, DeriveFieldset)]
struct Address {
    #[validate(required)]
    #[filter(trim)]
    street: String,

    #[validate(required, pattern = r"^\d{5}$")]
    #[filter(trim)]
    zip: String,
}

#[derive(Debug, DeriveFieldset)]
struct Registration {
    #[validate(required)]
    #[filter(trim)]
    name: String,

    #[validate(nested)]
    #[filter(nested)]
    address: Address,
}

fn main() {
    let form = Registration {
        name: "  Bob  ".into(),
        address: Address {
            street: "  123 Main St  ".into(),
            zip: "  90210  ".into(),
        },
    };

    match form.clean() {
        Ok(cleaned) => {
            println!("✓ Validation passed!");
            println!("  Name: {}", cleaned.name);
            println!("  Street: {}", cleaned.address.street);
            println!("  Zip: {}", cleaned.address.zip);
        }
        Err(violations) => {
            eprintln!("✗ Validation failed:");
            for (field, field_violations) in violations.iter() {
                for v in field_violations.0.iter() {
                    eprintln!("  {}: {}", field, v.message());
                }
            }
        }
    }

    // Example with nested validation errors
    println!("\n--- Testing with invalid nested data ---");
    let invalid_form = Registration {
        name: "Charlie".into(),
        address: Address {
            street: "456 Oak Ave".into(),
            zip: "ABC".into(), // Invalid zip code
        },
    };

    match invalid_form.clean() {
        Ok(_) => println!("✓ Unexpected success"),
        Err(violations) => {
            eprintln!("✗ Validation failed (expected):");
            for (field, field_violations) in violations.iter() {
                for v in field_violations.0.iter() {
                    eprintln!("  {}: {}", field, v.message());
                }
            }
        }
    }
}

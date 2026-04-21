# walrs_fieldset_derive

Procedural macro for deriving the `Fieldset` trait on structs with compile-time validation and filtering.

## Overview

`#[derive(Fieldset)]` generates type-safe validation and filtering implementations for your structs, eliminating the need for manual trait implementation. It supports:

- **Validation** — built-in rules (required, email, min/max, pattern, etc.)
- **Filtering** — value transformations (trim, lowercase, slug, etc.)
- **Nested structs** — recursive validation and filtering
- **Cross-field validation** — validate relationships between fields
- **Option<T>** — graceful handling of optional fields
- **Customization** — custom validators, filters, and error messages

## Installation

This crate is not used directly. Enable the `derive` feature in `walrs_fieldfilter`:

```toml
[dependencies]
walrs_fieldfilter = { path = "../fieldfilter", features = ["derive"] }
```

## Usage

### Basic Example

```rust
use walrs_fieldfilter::{DeriveFieldset, Fieldset};

#[derive(Debug, DeriveFieldset)]
struct ContactForm {
    #[validate(required, email)]
    #[filter(trim, lowercase)]
    email: String,

    #[validate(required, min_length = 2)]
    #[filter(trim)]
    name: String,
}

fn main() {
    let form = ContactForm {
        email: "  USER@EXAMPLE.COM  ".into(),
        name: "  Alice  ".into(),
    };
    
    match form.clean() {
        Ok(cleaned) => {
            // cleaned.email == "user@example.com"
            // cleaned.name == "Alice"
            println!("Success: {:?}", cleaned);
        }
        Err(violations) => {
            eprintln!("Validation failed: {}", violations);
        }
    }
}
```

### Nested Structs

Delegate validation and filtering to nested structs with `#[validate(nested)]` and `#[filter(nested)]`:

```rust
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
struct User {
    #[validate(required, email)]
    #[filter(trim, lowercase)]
    email: String,

    #[validate(nested)]
    #[filter(nested)]
    address: Address,
}
```

### Optional Fields

`Option<T>` fields are validated/filtered only if `Some`:

```rust
#[derive(Debug, DeriveFieldset)]
struct Profile {
    #[validate(required, email)]
    #[filter(trim, lowercase)]
    email: String,

    // Only validated if Some
    #[validate(url)]
    #[filter(trim)]
    website: Option<String>,
}
```

### Cross-Field Validation

Use `#[cross_validate(fn_name)]` for validating relationships between fields:

```rust
use walrs_fieldfilter::{DeriveFieldset, Fieldset, RuleResult};
use walrs_validation::{Violation, ViolationType};

#[derive(Debug, DeriveFieldset)]
#[cross_validate(check_passwords_match)]
struct Registration {
    #[validate(required, min_length = 8)]
    password: String,

    #[validate(required)]
    confirm_password: String,
}

fn check_passwords_match(reg: &Registration) -> RuleResult {
    if reg.password == reg.confirm_password {
        Ok(())
    } else {
        Err(Violation::new(
            ViolationType::NotEqual,
            "Passwords do not match",
        ))
    }
}
```

### Break on First Failure

Stop validation after the first field with violations:

```rust
#[derive(Debug, DeriveFieldset)]
#[fieldset(break_on_failure)]
struct StrictForm {
    #[validate(required)]
    field_a: String,

    #[validate(required)]
    field_b: String,
}
```

### Custom Error Messages

Override default messages with `message` or `message_fn`:

```rust
#[derive(Debug, DeriveFieldset)]
struct LoginForm {
    #[validate(required, message = "Email address is required")]
    #[validate(email, message = "Please enter a valid email")]
    #[filter(trim, lowercase)]
    email: String,

    #[validate(required, min_length = 8, message_fn = "password_message")]
    password: String,
}

impl LoginForm {
    fn password_message() -> String {
        "Password must be at least 8 characters".to_string()
    }
}
```

### Numeric Validation

Use `min`, `max`, `range`, and `step` for numeric fields:

```rust
#[derive(Debug, DeriveFieldset)]
struct Settings {
    #[validate(min = 0, max = 100)]
    #[filter(clamp(min = 0, max = 100))]
    volume: u8,

    #[validate(range(min = 1, max = 10))]
    rating: i32,

    #[validate(step = 5)]
    interval: u32,
}
```

### Custom Validators

Provide a custom validation function:

```rust
use walrs_fieldfilter::{DeriveFieldset, Fieldset, RuleResult};
use walrs_validation::{Violation, ViolationType};

#[derive(Debug, DeriveFieldset)]
struct Product {
    #[validate(custom = "validate_sku")]
    sku: String,
}

fn validate_sku(value: &str) -> RuleResult {
    if value.len() == 8 && value.chars().all(|c| c.is_alphanumeric()) {
        Ok(())
    } else {
        Err(Violation::new(
            ViolationType::CustomError,
            "SKU must be 8 alphanumeric characters",
        ))
    }
}
```

### Custom Filters

Provide a custom filter function:

```rust
use walrs_fieldfilter::{DeriveFieldset, Fieldset};

#[derive(Debug, DeriveFieldset)]
struct Message {
    #[filter(custom = "remove_emojis")]
    text: String,
}

fn remove_emojis(text: String) -> String {
    text.chars().filter(|c| !c.is_emoji()).collect()
}

trait EmojiExt {
    fn is_emoji(&self) -> bool;
}

impl EmojiExt for char {
    fn is_emoji(&self) -> bool {
        matches!(*self as u32, 0x1F600..=0x1F64F)
    }
}
```

## Validation Annotations

| Annotation | Description | Example |
|------------|-------------|---------|
| `required` | Field must not be empty | `#[validate(required)]` |
| `min_length = N` | Minimum string/collection length | `#[validate(min_length = 3)]` |
| `max_length = N` | Maximum string/collection length | `#[validate(max_length = 100)]` |
| `exact_length = N` | Exact length | `#[validate(exact_length = 10)]` |
| `email` | Valid email format | `#[validate(email)]` |
| `url` | Valid URL format | `#[validate(url)]` |
| `uri` | Valid URI format | `#[validate(uri)]` |
| `ip` | Valid IP address | `#[validate(ip)]` |
| `hostname` | Valid hostname | `#[validate(hostname)]` |
| `pattern = "regex"` | Matches regex pattern | `#[validate(pattern = r"^\d{5}$")]` |
| `min = N` | Minimum numeric value | `#[validate(min = 0)]` |
| `max = N` | Maximum numeric value | `#[validate(max = 100)]` |
| `range(min = A, max = B)` | Numeric range | `#[validate(range(min = 1, max = 10))]` |
| `step = N` | Numeric step/divisibility | `#[validate(step = 5)]` |
| `one_of = [a, b, c]` | Value in allowed list | `#[validate(one_of = ["red", "green", "blue"])]` |
| `custom = "fn_path"` | Custom validator | `#[validate(custom = "my_validator")]` |
| `nested` | Delegate to nested Fieldset | `#[validate(nested)]` |
| `message = "..."` | Custom error message | `#[validate(required, message = "Required")]` |
| `message_fn = "fn"` | Dynamic message provider | `#[validate(required, message_fn = "msg")]` |
| `locale = "en"` | Message locale | `#[validate(required, locale = "en")]` |

## Filter Annotations

| Annotation | Description | Example |
|------------|-------------|---------|
| `trim` | Remove leading/trailing whitespace | `#[filter(trim)]` |
| `lowercase` | Convert to lowercase | `#[filter(lowercase)]` |
| `uppercase` | Convert to uppercase | `#[filter(uppercase)]` |
| `strip_tags` | Remove HTML tags | `#[filter(strip_tags)]` |
| `html_entities` | Encode HTML entities | `#[filter(html_entities)]` |
| `slug` | URL-safe slug | `#[filter(slug)]` |
| `slug(max_length = N)` | Slug with max length | `#[filter(slug(max_length = 50))]` |
| `truncate(max_length = N)` | Truncate to length | `#[filter(truncate(max_length = 100))]` |
| `replace(from = "x", to = "y")` | String replacement | `#[filter(replace(from = " ", to = "-"))]` |
| `clamp(min = A, max = B)` | Clamp numeric value | `#[filter(clamp(min = 0, max = 100))]` |
| `digits` | Keep only ASCII digits `[0-9]` | `#[filter(digits)]` |
| `alnum` / `alnum(whitespace)` | Keep only Unicode alphanumerics (optionally whitespace) | `#[filter(alnum(whitespace))]` |
| `alpha` / `alpha(whitespace)` | Keep only Unicode alphabetic chars (optionally whitespace) | `#[filter(alpha)]` |
| `strip_newlines` | Remove `\r` and `\n` | `#[filter(strip_newlines)]` |
| `normalize_whitespace` | Collapse whitespace runs and trim | `#[filter(normalize_whitespace)]` |
| `allow_chars = "..."` | Keep only characters in set | `#[filter(allow_chars = "abc123")]` |
| `deny_chars = "..."` | Remove characters in set | `#[filter(deny_chars = "!?.")]` |
| `url_encode` | Percent-encode (RFC 3986) | `#[filter(url_encode)]` |
| `to_bool` | Parse to canonical `"true"`/`"false"` (fallible) | `#[filter(to_bool)]` |
| `to_int` | Parse string as `i64` and canonicalize (fallible) | `#[filter(to_int)]` |
| `to_float` | Parse string as `f64` and canonicalize (fallible) | `#[filter(to_float)]` |
| `url_decode` | Percent-decode (fallible) | `#[filter(url_decode)]` |
| `custom = "fn_path"` | Custom filter | `#[filter(custom = "my_filter")]` |
| `try_custom = "fn_path"` | Fallible custom filter | `#[filter(try_custom = "parse_int")]` |
| `nested` | Delegate to nested Fieldset | `#[filter(nested)]` |

## Struct-Level Attributes

- `#[fieldset(break_on_failure)]` — Stop validation after the first field with violations
- `#[cross_validate(fn_name)]` — Cross-field validation function

## License

Elastic-2.0

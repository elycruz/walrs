# Design: `Fieldset` Derive & `walrs_form_derive`

**Date:** April 12, 2026
**Status:** Design
**Crates affected:** `walrs_form` (traits), `walrs_form_derive` (new proc-macro crate)

---

## Table of Contents

1. [Goal](#goal)
2. [Background](#background)
3. [The `IntoFieldset` Trait](#the-intofieldset-trait)
4. [Derive Macro: `#[derive(Fieldset)]`](#derive-macro-derivefieldset)
   - [4.1 Field-Level Annotations](#41-field-level-annotations)
   - [4.2 Select Element Mapping](#42-select-element-mapping)
   - [4.3 Textarea Element Mapping](#43-textarea-element-mapping)
   - [4.4 Button Elements](#44-button-elements)
   - [4.5 Nested Fieldsets](#45-nested-fieldsets)
   - [4.6 Struct-Level Annotations](#46-struct-level-annotations)
   - [4.7 Skipping Fields](#47-skipping-fields)
5. [Generated Code Examples](#generated-code-examples)
   - [5.1 Simple Form](#51-simple-form)
   - [5.2 Select Element with Options](#52-select-element-with-options)
   - [5.3 Nested Fieldset](#53-nested-fieldset)
6. [Combining with `Filterable`](#combining-with-filterable)
   - [6.1 Both Derives Together](#61-both-derives-together)
   - [6.2 Full Integration Example](#62-full-integration-example)
7. [Type Mapping: Rust Type → Element Type](#type-mapping-rust-type--element-type)
8. [Crate Layout](#crate-layout)
9. [Open Questions](#open-questions)
10. [Out of Scope](#out-of-scope)

---

## Goal

Provide a `#[derive(Fieldset)]` macro that generates `FieldsetElement` (and
`Vec<Element>`) from a Rust struct definition, mapping struct fields to HTML
form elements (`InputElement`, `SelectElement`, `TextareaElement`, etc.) at
compile time.

This allows defining form structure **declaratively** on the data struct itself,
keeping the form definition co-located with the data model.

---

## Background

The `walrs_form` crate already provides:

- `InputElement`, `SelectElement`, `TextareaElement`, `ButtonElement`, `FieldsetElement`
- `Element` enum wrapping all element types
- `Form` struct containing `Vec<Element>`
- Builder patterns for all element structs

Currently, building a form requires manually constructing each element:

```rust
let mut form = Form::new("registration");
form.add_element(InputElement::new("name", InputType::Text).into());
form.add_element(InputElement::new("email", InputType::Email).into());
// ... dozens of lines for a complex form
```

The `Fieldset` derive automates this by reading struct field annotations and
generating the element construction code.

---

## The `IntoFieldset` Trait

Defined in `walrs_form`:

```rust
use crate::{FieldsetElement, Element};

/// Trait for structs that can be converted into form fieldset elements.
///
/// Implementors produce a `FieldsetElement` containing `Element` instances
/// corresponding to their fields. Can be derived via `#[derive(Fieldset)]`.
pub trait IntoFieldset {
    /// Builds a `FieldsetElement` from the struct's field definitions.
    ///
    /// This is a **static** method — it builds the form structure from the
    /// struct's type information (annotations), not from instance data.
    fn fieldset() -> FieldsetElement;

    /// Builds a `Vec<Element>` from the struct's field definitions.
    ///
    /// Convenience method that returns just the elements without the
    /// fieldset wrapper.
    fn elements() -> Vec<Element> {
        Self::fieldset()
            .elements
            .unwrap_or_default()
    }
}
```

### Why `IntoFieldset` and not `Into<Form>`?

- A struct maps to a **group of fields** (a fieldset), not a full form. A
  `Form` also has `action`, `method`, `enctype`, etc. that don't come from the
  data struct.
- Users compose: `form.add_element(MyStruct::fieldset().into())` or use the
  elements directly.

---

## Derive Macro: `#[derive(Fieldset)]`

### 4.1 Field-Level Annotations

Each struct field can be annotated with `#[field(...)]` to control element
generation:

| Annotation | Effect | Default |
|---|---|---|
| `type = "email"` | Sets `InputType::Email` | Inferred from Rust type |
| `type = "password"` | Sets `InputType::Password` | — |
| `type = "number"` | Sets `InputType::Number` | Inferred for numeric types |
| `type = "checkbox"` | Sets `InputType::Checkbox` | Inferred for `bool` |
| `type = "hidden"` | Sets `InputType::Hidden` | — |
| `type = "date"` | Sets `InputType::Date` | — |
| `type = "tel"` | Sets `InputType::Tel` | — |
| `type = "url"` | Sets `InputType::Url` | — |
| `type = "color"` | Sets `InputType::Color` | — |
| `type = "range"` | Sets `InputType::Range` | — |
| `type = "search"` | Sets `InputType::Search` | — |
| `textarea` | Generates `TextareaElement` instead of `InputElement` | — |
| `select` | Generates `SelectElement` (see §4.2) | — |
| `label = "Display Name"` | Sets element label | Humanized field name |
| `help = "Enter your name"` | Sets help_message | `None` |
| `disabled` | Sets `disabled = Some(true)` | `None` |
| `required` | Sets `required = Some(true)` | `None` |
| `name = "custom_name"` | Overrides HTML name attribute | Rust field name |

**Type inference** (when no `type` annotation is present):

| Rust type | Inferred `InputType` |
|---|---|
| `String` | `InputType::Text` |
| `i8`..`i128`, `u8`..`u128`, `f32`, `f64` | `InputType::Number` |
| `bool` | `InputType::Checkbox` |
| `Option<T>` | Same as `T` (not required unless annotated) |

### 4.2 Select Element Mapping

A field annotated with `#[field(select)]` generates a `SelectElement`:

```rust
#[derive(Fieldset)]
struct Preferences {
    #[field(select, label = "Country", options(
        "us" = "United States",
        "uk" = "United Kingdom",
        "ca" = "Canada",
    ))]
    country: String,

    #[field(select, multiple, label = "Languages", options(
        "en" = "English",
        "es" = "Spanish",
        "fr" = "French",
    ))]
    languages: Vec<String>,
}
```

#### Options source strategies

**1. Inline options** (as above):

```rust
#[field(select, options("us" = "United States", "uk" = "United Kingdom"))]
country: String,
```

**2. Function reference** — for dynamic or computed option lists:

```rust
#[field(select, options_fn = "country_options")]
country: String,

// Must return Vec<SelectOption>
fn country_options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("us", "United States"),
        SelectOption::new("uk", "United Kingdom"),
    ]
}
```

**3. Enum options** — for enum-backed selects:

```rust
#[derive(FieldsetOptions)]
enum Country {
    #[option(value = "us", label = "United States")]
    UnitedStates,
    #[option(value = "uk", label = "United Kingdom")]
    UnitedKingdom,
}

#[derive(Fieldset)]
struct Preferences {
    #[field(select, options_enum = "Country")]
    country: String,
}
```

The `#[derive(FieldsetOptions)]` macro generates:

```rust
impl Country {
    pub fn to_select_options() -> Vec<SelectOption> { ... }
}
```

### 4.3 Textarea Element Mapping

```rust
#[derive(Fieldset)]
struct Post {
    #[field(textarea, label = "Content", rows = 10, cols = 80)]
    body: String,
}
```

Generates:

```rust
TextareaElement {
    name: Some("body".into()),
    label: Some("Content".to_string()),
    rows: Some(10),
    cols: Some(80),
    ..Default::default()
}
```

### 4.4 Button Elements

Buttons are not typically part of a data struct. They are added at the
`Form` level, not via `#[derive(Fieldset)]`. However, a struct-level
annotation allows declaring submit/reset buttons:

```rust
#[derive(Fieldset)]
#[fieldset(
    submit(label = "Register"),
    reset(label = "Clear"),
)]
struct Registration {
    #[field(type = "text", label = "Name")]
    name: String,
}
```

This appends `ButtonElement` instances to the generated `FieldsetElement`.

### 4.5 Nested Fieldsets

When a field's type also derives `Fieldset`, it generates a nested
`FieldsetElement`:

```rust
#[derive(Fieldset)]
struct Registration {
    #[field(type = "text")]
    name: String,

    #[field(fieldset, legend = "Shipping Address")]
    address: UserAddress,  // UserAddress: IntoFieldset
}
```

Generated code nests the fieldset:

```rust
{
    let mut nested = UserAddress::fieldset();
    nested.name = Some("address".into());
    nested.legend = Some("Shipping Address".to_string());
    elements.push(Element::Fieldset(nested));
}
```

### 4.6 Struct-Level Annotations

| Annotation | Effect |
|---|---|
| `#[fieldset(name = "...")]` | Sets `FieldsetElement::name` |
| `#[fieldset(legend = "...")]` | Sets `FieldsetElement::legend` |
| `#[fieldset(submit(label = "..."))]` | Appends submit button |
| `#[fieldset(reset(label = "..."))]` | Appends reset button |

### 4.7 Skipping Fields

Fields that should not appear in the form:

```rust
#[derive(Fieldset)]
struct User {
    #[field(skip)]
    internal_id: u64,

    #[field(type = "text")]
    name: String,
}
```

---

## Generated Code Examples

### 5.1 Simple Form

```rust
use walrs_form_derive::Fieldset;

#[derive(Fieldset)]
#[fieldset(legend = "Login")]
struct LoginForm {
    #[field(type = "text", label = "Username", required)]
    username: String,

    #[field(type = "password", label = "Password", required)]
    password: String,
}
```

**Generated:**

```rust
impl walrs_form::IntoFieldset for LoginForm {
    fn fieldset() -> walrs_form::FieldsetElement {
        let mut elements = Vec::new();

        // username
        {
            let mut el = walrs_form::InputElement::new("username", walrs_form::InputType::Text);
            el.label = Some("Username".to_string());
            el.required = Some(true);
            elements.push(walrs_form::Element::Input(el));
        }

        // password
        {
            let mut el = walrs_form::InputElement::new("password", walrs_form::InputType::Password);
            el.label = Some("Password".to_string());
            el.required = Some(true);
            elements.push(walrs_form::Element::Input(el));
        }

        walrs_form::FieldsetElement {
            name: None,
            legend: Some("Login".to_string()),
            disabled: None,
            elements: Some(elements),
            attributes: None,
        }
    }
}
```

### 5.2 Select Element with Options

```rust
#[derive(Fieldset)]
struct Preferences {
    #[field(select, label = "Country", required, options(
        "us" = "United States",
        "ca" = "Canada",
        "uk" = "United Kingdom",
    ))]
    country: String,
}
```

**Generated:**

```rust
impl walrs_form::IntoFieldset for Preferences {
    fn fieldset() -> walrs_form::FieldsetElement {
        let mut elements = Vec::new();

        // country
        {
            let mut el = walrs_form::SelectElement::new("country");
            el.label = Some("Country".to_string());
            el.required = Some(true);
            el.options = vec![
                walrs_form::SelectOption::new("us", "United States"),
                walrs_form::SelectOption::new("ca", "Canada"),
                walrs_form::SelectOption::new("uk", "United Kingdom"),
            ];
            elements.push(walrs_form::Element::Select(el));
        }

        walrs_form::FieldsetElement {
            elements: Some(elements),
            ..Default::default()
        }
    }
}
```

### 5.3 Nested Fieldset

```rust
#[derive(Fieldset)]
#[fieldset(legend = "Address")]
struct Address {
    #[field(type = "text", label = "Street", required)]
    street: String,

    #[field(type = "text", label = "ZIP Code", required)]
    zip: String,
}

#[derive(Fieldset)]
#[fieldset(legend = "Registration")]
struct Registration {
    #[field(type = "text", label = "Name", required)]
    name: String,

    #[field(fieldset, legend = "Shipping Address")]
    shipping: Address,

    #[field(fieldset, legend = "Billing Address")]
    billing: Address,
}
```

The generated `Registration::fieldset()` contains two nested `FieldsetElement`s,
each containing `Address`'s elements with their respective legends.

---

## Combining with `Filterable`

The `Fieldset` derive and `Filterable` derive are **independent** but
**complementary**. A struct can derive both:

### 6.1 Both Derives Together

```rust
use walrs_inputfilter_derive::Filterable;
use walrs_form_derive::Fieldset;

#[derive(Filterable, Fieldset)]
#[fieldset(legend = "User Registration")]
#[cross_validate(passwords_match)]
struct UserRegistration {
    // Filterable: validate + filter
    // Fieldset: generate InputElement(Text)
    #[validate(required, min_length = 2, max_length = 50)]
    #[filter(trim)]
    #[field(type = "text", label = "Full Name")]
    name: String,

    // Filterable: validate + filter
    // Fieldset: generate InputElement(Email)
    #[validate(required, email)]
    #[filter(trim, lowercase)]
    #[field(type = "email", label = "Email Address")]
    email: String,

    // Filterable: validate
    // Fieldset: generate InputElement(Password)
    #[validate(required, min_length = 8)]
    #[field(type = "password", label = "Password")]
    password: String,

    // Filterable: validate
    // Fieldset: generate InputElement(Password)
    #[validate(required)]
    #[field(type = "password", label = "Confirm Password")]
    confirm_password: String,

    // Filterable: validate (numeric)
    // Fieldset: generate InputElement(Number)
    #[validate(min = 0, max = 150)]
    #[field(type = "number", label = "Age")]
    age: i64,

    // Filterable: validate (select value is one_of the options)
    // Fieldset: generate SelectElement with options
    #[validate(required, one_of = ["us", "ca", "uk"])]
    #[field(select, label = "Country", options(
        "us" = "United States",
        "ca" = "Canada",
        "uk" = "United Kingdom",
    ))]
    country: String,

    // Both: nested
    #[validate(nested)]
    #[filter(nested)]
    #[field(fieldset, legend = "Address")]
    address: UserAddress,
}

fn passwords_match(r: &UserRegistration) -> walrs_validation::ValidatorResult {
    if r.password == r.confirm_password {
        Ok(())
    } else {
        Err(walrs_validation::Violation::new(
            walrs_validation::ViolationType::NotEqual,
            "Passwords must match",
        ))
    }
}
```

### 6.2 Full Integration Example

```rust
use walrs_form::Form;
use walrs_inputfilter::Filterable;
use walrs_form::IntoFieldset;

// Build the form structure from the struct definition
fn build_registration_form() -> Form {
    let mut form = Form::new("registration");
    form.action = Some("/api/register".to_string());
    form.method = Some(walrs_form::FormMethod::Post);

    // Generate fieldset from struct annotations
    let fieldset = UserRegistration::fieldset();
    form.add_element(fieldset.into());

    form
}

// Validate incoming data using Filterable
fn handle_registration(input: UserRegistration) -> Result<UserRegistration, walrs_validation::FieldViolations> {
    // process() = filter() + validate()
    input.process()
}

// Combined: build form for rendering, validate on submission
fn main() {
    // 1. Build form structure for frontend rendering
    let form = build_registration_form();
    let form_json = serde_json::to_string_pretty(&form).unwrap();
    println!("Form definition:\n{}", form_json);

    // 2. On form submission, validate the data
    let registration = UserRegistration {
        name: "  Jane Doe  ".to_string(),
        email: "  JANE@EXAMPLE.COM  ".to_string(),
        password: "securepass123".to_string(),
        confirm_password: "securepass123".to_string(),
        age: 30,
        country: "us".to_string(),
        address: UserAddress {
            street: "  123 Main St  ".to_string(),
            zip: "90210".to_string(),
        },
    };

    match registration.process() {
        Ok(clean) => {
            // Filters applied: name trimmed, email trimmed+lowercased
            assert_eq!(clean.name, "Jane Doe");
            assert_eq!(clean.email, "jane@example.com");
            println!("Registration valid!");
        }
        Err(violations) => {
            for (field, field_violations) in &violations.fields {
                println!("Field '{}': {:?}", field, field_violations);
            }
        }
    }
}
```

---

## Type Mapping: Rust Type → Element Type

The derive macro uses this mapping when no explicit `#[field(type = "...")]` is
provided:

| Rust type | Default element | Default `InputType` |
|---|---|---|
| `String` | `InputElement` | `InputType::Text` |
| `i8`..`i128`, `u8`..`u128` | `InputElement` | `InputType::Number` |
| `f32`, `f64` | `InputElement` | `InputType::Number` |
| `bool` | `InputElement` | `InputType::Checkbox` |
| `Option<T>` | Same as `T` | Same as `T` |
| `Vec<T>` | (requires explicit annotation) | — |
| `T: IntoFieldset` | `FieldsetElement` (nested) | — |

---

## Crate Layout

```
crates/
├── form/
│   ├── Cargo.toml             # adds `walrs_form_derive` as optional dep
│   └── src/
│       ├── lib.rs              # re-exports `IntoFieldset` trait;
│       │                       #   conditionally re-exports derive macro
│       ├── into_fieldset.rs    # `IntoFieldset` trait definition (NEW)
│       └── ...                 # existing modules unchanged
├── form_derive/                # (NEW)
│   ├── Cargo.toml              # proc-macro = true
│   └── src/
│       ├── lib.rs              # #[proc_macro_derive(Fieldset, attributes(field, fieldset))]
│       ├── parse.rs            # Attribute parsing
│       ├── gen_fieldset.rs     # Code generation for IntoFieldset::fieldset()
│       ├── gen_input.rs        # InputElement generation
│       ├── gen_select.rs       # SelectElement generation
│       └── gen_textarea.rs     # TextareaElement generation
```

### Cargo feature gate

```toml
# crates/form/Cargo.toml
[features]
default = ["std"]
std = []
derive = ["walrs_form_derive"]
wasm = []

[dependencies]
walrs_form_derive = { path = "../form_derive", optional = true }
```

Users opt in with:

```toml
walrs_form = { version = "...", features = ["derive"] }
```

### `walrs_form_derive/Cargo.toml`

```toml
[package]
name = "walrs_form_derive"
version = "0.1.0"
edition = "2024"
authors = ["Ely De La Cruz <elycruz@elycruz.com>"]
description = "Derive macro for walrs_form Fieldset trait"
license = "Elastic-2.0"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full", "extra-traits"] }
quote = "1"
proc-macro2 = "1"
```

---

## Open Questions

1. **`FieldsetOptions` derive** — Should the `#[derive(FieldsetOptions)]` for
   enum → select options be part of `walrs_form_derive` or a separate concern?
   (Recommendation: same crate, since it's tightly coupled.)

2. **Attribute HTML propagation** — Should `#[field(attr(class = "form-control"))]`
   generate `Attributes` entries on the element? This would allow CSS class and
   other HTML attributes to be specified at the struct level.

3. **Default values** — Should `#[field(default = "value")]` set
   `InputElement::value` in the generated fieldset? This is useful for
   pre-populating forms.

4. **Instance-based fieldset** — The current `IntoFieldset::fieldset()` is a
   static method (type-level). Should we also provide
   `fn fieldset_with_data(&self) -> FieldsetElement` that populates element
   values from the struct's field values?

---

## Out of Scope

1. **Client-side rendering** — The derive generates data structures, not HTML.
   Rendering is the responsibility of a template engine or WASM frontend.
2. **Form-level attributes** — `action`, `method`, `enctype` are not part of
   the data struct; they belong to `Form` and are set separately.
3. **Dynamic options** — Options that change at runtime (e.g., from a database)
   should use `options_fn` or be set manually after calling `fieldset()`.

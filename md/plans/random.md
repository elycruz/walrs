## Random Todos:

- Move Filter enum to filter crate
- Rename `Rule<T>` to `Constraint<T>`?  More precise (slightly more verbose though).
- Rename `validate*` to `check_validity*`/`check*`?
- Disambiguate `Filter` enum from `Filter` trait.
- Make `pub(crate)` updates (`pub(crate) fn validate_(str|len|etc)`).
- Address ` message: Message::Static(String::new()),` in `WithMessage` handling.
- Address "Value used after move" in navigation_benchmarks.rs.
- Consider adding a `builder(name)` method to `*Element` structures - enables builder pattern than can compile `Rule<T>` from field values to be populated for target struct.
- Expose all crates from root crate `lib` using their short names.
- Finalize lib licenses and reference them in all crates (READMEs, LICENSEs, Cargo.toml, etc.).

- Consider changing all 'validation' results to `Option<Violation>` and `Option<Violations>` respectively.
  - Sync validation is not fallible so it shouldn't return `Result<...>`, also since `Result<...>` creates more
    overhead in logic, it shouldn't be used for this use case.

### Support "verbose" validation format, for form and element validation:  E.g.,

```jsonc
// Enables parameterizing front-end forms with validation configiguration to allow browser "constraint validation" 
// to display validation messages (with our without a framework - server side rendered messages and/or with a 
// framework, like React, Next, Vue, etc.).
// 
// Serialize of `HashMap<String, HashMap<String, ConstraintViolation>>` (etc.):
// ----
{
  "formErrors": {
    "user": {
      "username": {
        // Head of last validation messages encountered.
        "validationMessage": "Username must be at least 3 characters long.",
        // Constraints used in validity check.
        "violatedConstraints": { // Allows front-end to reapply constriants on violated fields.
          "minLength": 3
        },
        "validity": {
          "tooShort": true
        }
      }
    }
  }
}
```

Method call (on rust side):

```rust
use walrs_form::{Form, InputElement};

fn main () -> Result<(), Box<dyn std::error::Error>> {
  // User form
  let form = Form::builder()
    .name("user") // Optional, but useful for grouping and identifying form errors.
    .add_element(
      InputElement::builder("username")
        .required(true)
        .min_length(3)
        .build()?
    )
    .build()?;
  
  // Would return `Option<HashMap<String, ConstraintViolation>>` (or `Result<...>`).
  let form_errors = form.check_validity_verbose();
  
  // Here we would serialize the form errors into an overall "formErrors" field, in resulting JSON.
  // ...
}
```

### Default format:

```jsonc
// Enables front-end forms to display validation messages without any additional configuration.
// 
// Serialize of `HashMap<String, String>` (etc.):
// ----
{
  "formErrors": {
    "user": {
      "username": "Username must be at least 3 characters long."
    }
  }
}
```

Simple and succinct just for displaying [validation] messages however front-end dev chooses.

### Using OData-style $metadata for validation metadata:

```json
{
  "@odata.error": {
    "code": "ValidationError",
    "message": "One or more validation errors occurred.",
    "details": [
      {
        "code": "MinLength",
        "message": "Username must be at least 3 characters long.",
        "target": "user.username"
      }
    ]
  }
}
```

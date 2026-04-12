# Design: `walrs_form_serde` — Form Serialization, Schema & Codegen

**Date:** April 12, 2026
**Status:** Design
**Crates affected:** `walrs_form_serde` (new crate), `walrs_form` (feature-gated re-exports)

---

## Table of Contents

1. [Goal](#goal)
2. [Background](#background)
3. [Crate Responsibilities](#crate-responsibilities)
4. [Form Loader: JSON/YAML → Form](#form-loader-jsonyaml--form)
   - [4.1 FormLoader Trait](#41-formloader-trait)
   - [4.2 JSON Loader](#42-json-loader)
   - [4.3 YAML Loader](#43-yaml-loader)
   - [4.4 JSON/YAML Form Definition Schema](#44-jsonyaml-form-definition-schema)
5. [Form Emitter: Form → JSON/YAML](#form-emitter-form--jsonyaml)
6. [JSON Schema Generation: Form → JSON Schema](#json-schema-generation-form--json-schema)
   - [6.1 SchemaGenerator Trait](#61-schemagenerator-trait)
   - [6.2 Mapping Rules](#62-mapping-rules)
   - [6.3 Generated Schema Example](#63-generated-schema-example)
7. [TypeScript Interface Generation](#typescript-interface-generation)
   - [7.1 TsGenerator Trait](#71-tsgenerator-trait)
   - [7.2 Mapping Rules](#72-mapping-rules)
   - [7.3 Generated TypeScript Example](#73-generated-typescript-example)
8. [WASM Bindings](#wasm-bindings)
   - [8.1 Exposed Functions](#81-exposed-functions)
   - [8.2 JavaScript Usage Example](#82-javascript-usage-example)
9. [Crate Layout](#crate-layout)
10. [Feature Flags](#feature-flags)
11. [Examples & Usage](#examples--usage)
12. [Open Questions](#open-questions)
13. [Out of Scope](#out-of-scope)

---

## Goal

Provide a `walrs_form_serde` crate that bridges `walrs_form` types with
external formats:

1. **Load** form definitions from JSON/YAML files into `Form` structs.
2. **Emit** `Form` structs back to JSON/YAML.
3. **Generate JSON Schema** from a `Form` (for frontend validation, API docs).
4. **Generate TypeScript interfaces** from a `Form` (for type-safe frontends).
5. **Expose the above via WASM** for browser/Node.js usage.

---

## Background

The `walrs_form` crate already derives `Serialize` and `Deserialize` on
all form types (`Form`, `Element`, `InputElement`, `SelectElement`, etc.).
However, round-tripping a complex `Form` through JSON requires careful
handling of the `Element` enum (tagged union), and additional capabilities
like JSON Schema and TypeScript generation are not covered by serde alone.

The old `fieldfilter_additions_and_refactor_plan.md` (Step 5) outlined a
`walrs_form_serde` crate. This document refines that design.

---

## Crate Responsibilities

| Responsibility | Feature flag | Dependencies |
|---|---|---|
| JSON loading (`Form` from JSON string/file) | `json` (default) | `serde_json` |
| YAML loading (`Form` from YAML string/file) | `yaml` | `serde_yaml` |
| JSON Schema generation | `json-schema` | `serde_json` |
| TypeScript generation | `typescript` | (none — string codegen) |
| WASM bindings | `wasm` | `wasm-bindgen`, `serde-wasm-bindgen` |

---

## Form Loader: JSON/YAML → Form

### 4.1 FormLoader Trait

```rust
use walrs_form::Form;
use std::path::Path;

/// Errors that can occur during form loading.
#[derive(Debug, thiserror::Error)]
pub enum FormLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "yaml")]
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Trait for loading a Form from an external format.
pub trait FormLoader {
    /// Load a Form from a string.
    fn load_from_str(input: &str) -> Result<Form, FormLoadError>;

    /// Load a Form from a file path.
    fn load_from_file(path: &Path) -> Result<Form, FormLoadError> {
        let content = std::fs::read_to_string(path)?;
        Self::load_from_str(&content)
    }
}
```

### 4.2 JSON Loader

```rust
pub struct JsonFormLoader;

impl FormLoader for JsonFormLoader {
    fn load_from_str(input: &str) -> Result<Form, FormLoadError> {
        let form: Form = serde_json::from_str(input)?;
        Ok(form)
    }
}
```

### 4.3 YAML Loader

```rust
#[cfg(feature = "yaml")]
pub struct YamlFormLoader;

#[cfg(feature = "yaml")]
impl FormLoader for YamlFormLoader {
    fn load_from_str(input: &str) -> Result<Form, FormLoadError> {
        let form: Form = serde_yaml::from_str(input)?;
        Ok(form)
    }
}
```

### 4.4 JSON/YAML Form Definition Schema

Both JSON and YAML loaders consume the same logical structure, derived from
`walrs_form`'s serde representation. Key points:

- `Element` is serialized as a **tagged enum** (using serde's default
  externally-tagged representation):

```json
{
  "name": "registration",
  "action": "/api/register",
  "method": "post",
  "elements": [
    {
      "Input": {
        "name": "email",
        "input_type": "email",
        "label": "Email Address",
        "required": true
      }
    },
    {
      "Select": {
        "name": "country",
        "label": "Country",
        "options": [
          { "value": "us", "label": "United States" },
          { "value": "uk", "label": "United Kingdom" }
        ]
      }
    },
    {
      "Fieldset": {
        "legend": "Address",
        "elements": [
          {
            "Input": {
              "name": "street",
              "input_type": "text",
              "label": "Street"
            }
          }
        ]
      }
    }
  ]
}
```

YAML equivalent:

```yaml
name: registration
action: /api/register
method: post
elements:
  - Input:
      name: email
      input_type: email
      label: Email Address
      required: true
  - Select:
      name: country
      label: Country
      options:
        - value: us
          label: United States
        - value: uk
          label: United Kingdom
```

---

## Form Emitter: Form → JSON/YAML

Since `Form` and all its types derive `Serialize`, emitting is
straightforward:

```rust
pub struct JsonFormEmitter;

impl JsonFormEmitter {
    /// Emit a Form as a pretty-printed JSON string.
    pub fn emit(form: &Form) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(form)
    }

    /// Emit a Form as a compact JSON string.
    pub fn emit_compact(form: &Form) -> Result<String, serde_json::Error> {
        serde_json::to_string(form)
    }
}

#[cfg(feature = "yaml")]
pub struct YamlFormEmitter;

#[cfg(feature = "yaml")]
impl YamlFormEmitter {
    pub fn emit(form: &Form) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(form)
    }
}
```

---

## JSON Schema Generation: Form → JSON Schema

### 6.1 SchemaGenerator Trait

```rust
use serde_json::Value as JsonValue;
use walrs_form::Form;

/// Generates a JSON Schema from a Form definition.
pub trait SchemaGenerator {
    /// Generate a JSON Schema describing the form's expected input data.
    fn generate_schema(form: &Form) -> JsonValue;
}

pub struct JsonSchemaGenerator;

impl SchemaGenerator for JsonSchemaGenerator {
    fn generate_schema(form: &Form) -> JsonValue {
        // Walk form.elements, build JSON Schema properties
        todo!()
    }
}
```

### 6.2 Mapping Rules

| Element type | `InputType` | JSON Schema type | Additional |
|---|---|---|---|
| `InputElement` | `Text`, `Email`, `Password`, `Search`, `Tel`, `Url` | `"string"` | `format` for email, uri, etc. |
| `InputElement` | `Number`, `Range` | `"number"` | `minimum`, `maximum` if present |
| `InputElement` | `Checkbox` | `"boolean"` | — |
| `InputElement` | `Date`, `DateTimeLocal` | `"string"` | `format: "date"` / `"date-time"` |
| `InputElement` | `Hidden` | `"string"` | — |
| `InputElement` | `Color` | `"string"` | `pattern: "^#[0-9a-fA-F]{6}$"` |
| `SelectElement` | — | `"string"` | `enum` from option values |
| `SelectElement` (multiple) | — | `"array"` | `items.enum` from option values |
| `TextareaElement` | — | `"string"` | `maxLength` if set |
| `FieldsetElement` | — | `"object"` | Recurse into nested elements |

### 6.3 Generated Schema Example

For the registration form defined above:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "registration",
  "type": "object",
  "properties": {
    "email": {
      "type": "string",
      "format": "email"
    },
    "country": {
      "type": "string",
      "enum": ["us", "uk"]
    },
    "street": {
      "type": "string"
    }
  },
  "required": ["email"]
}
```

**Note:** Nested fieldsets generate nested `"object"` schemas with their own
`properties` and `required` arrays.

---

## TypeScript Interface Generation

### 7.1 TsGenerator Trait

```rust
use walrs_form::Form;

/// Generates TypeScript type definitions from a Form definition.
pub trait TsGenerator {
    /// Generate TypeScript interface source code.
    fn generate_ts(form: &Form) -> String;
}

pub struct TypeScriptGenerator;

impl TsGenerator for TypeScriptGenerator {
    fn generate_ts(form: &Form) -> String {
        // Walk form.elements, build TS interface
        todo!()
    }
}
```

### 7.2 Mapping Rules

| Element type | `InputType` | TypeScript type |
|---|---|---|
| `InputElement` | `Text`, `Email`, `Password`, etc. | `string` |
| `InputElement` | `Number`, `Range` | `number` |
| `InputElement` | `Checkbox` | `boolean` |
| `InputElement` | `Date`, `DateTimeLocal` | `string` (ISO 8601) |
| `SelectElement` | (single) | Union of literal string types |
| `SelectElement` | (multiple) | Array of literal string types |
| `TextareaElement` | — | `string` |
| `FieldsetElement` | — | Nested interface |

Optional fields (not `required`) generate `fieldName?: type`.

### 7.3 Generated TypeScript Example

```typescript
/** Generated from form: registration */
export interface RegistrationForm {
  email: string;
  country: "us" | "uk";
  address: RegistrationFormAddress;
}

export interface RegistrationFormAddress {
  street: string;
}
```

---

## WASM Bindings

### 8.1 Exposed Functions

Feature-gated behind `wasm`:

```rust
use wasm_bindgen::prelude::*;

/// Load a Form from a JSON string.
#[wasm_bindgen]
pub fn load_form_json(json: &str) -> Result<JsValue, JsError> {
    let form = JsonFormLoader::load_from_str(json)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(serde_wasm_bindgen::to_value(&form)?)
}

/// Emit a Form as a JSON string.
#[wasm_bindgen]
pub fn emit_form_json(form: JsValue) -> Result<String, JsError> {
    let form: Form = serde_wasm_bindgen::from_value(form)?;
    JsonFormEmitter::emit(&form).map_err(|e| JsError::new(&e.to_string()))
}

/// Generate a JSON Schema from a Form (as JsValue).
#[wasm_bindgen]
pub fn generate_json_schema(form: JsValue) -> Result<JsValue, JsError> {
    let form: Form = serde_wasm_bindgen::from_value(form)?;
    let schema = JsonSchemaGenerator::generate_schema(&form);
    Ok(serde_wasm_bindgen::to_value(&schema)?)
}

/// Generate TypeScript interfaces from a Form.
#[wasm_bindgen]
pub fn generate_typescript(form: JsValue) -> Result<String, JsError> {
    let form: Form = serde_wasm_bindgen::from_value(form)?;
    Ok(TypeScriptGenerator::generate_ts(&form))
}
```

### 8.2 JavaScript Usage Example

```javascript
import init, {
  load_form_json,
  generate_json_schema,
  generate_typescript,
} from "walrs_form_serde";

await init();

const formJson = await fetch("/api/forms/registration").then(r => r.text());
const form = load_form_json(formJson);

// Generate JSON Schema for client-side validation
const schema = generate_json_schema(form);
console.log("JSON Schema:", schema);

// Generate TypeScript interfaces
const tsCode = generate_typescript(form);
console.log("TypeScript:\n", tsCode);
```

---

## Crate Layout

```
crates/
├── form_serde/                     # (NEW)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Feature-gated re-exports
│       ├── error.rs                # FormLoadError
│       ├── loader/
│       │   ├── mod.rs              # FormLoader trait
│       │   ├── json.rs             # JsonFormLoader
│       │   └── yaml.rs             # YamlFormLoader (feature = "yaml")
│       ├── emitter/
│       │   ├── mod.rs              # Emitter types
│       │   ├── json.rs             # JsonFormEmitter
│       │   └── yaml.rs             # YamlFormEmitter (feature = "yaml")
│       ├── schema/
│       │   ├── mod.rs              # SchemaGenerator trait
│       │   └── json_schema.rs      # JsonSchemaGenerator
│       ├── codegen/
│       │   ├── mod.rs              # TsGenerator trait
│       │   └── typescript.rs       # TypeScriptGenerator
│       └── wasm.rs                 # WASM bindings (feature = "wasm")
```

---

## Feature Flags

```toml
[package]
name = "walrs_form_serde"
version = "0.1.0"
edition = "2024"
authors = ["Ely De La Cruz <elycruz@elycruz.com>"]
description = "Form serialization, JSON Schema, and TypeScript generation for walrs_form"
license = "Elastic-2.0"

[features]
default = ["json"]
json = ["serde_json"]
yaml = ["serde_yaml"]
json-schema = ["serde_json"]
typescript = []
wasm = ["wasm-bindgen", "serde-wasm-bindgen", "json", "json-schema", "typescript"]

[dependencies]
walrs_form = { path = "../form" }
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", optional = true }
serde_yaml = { version = "0.9", optional = true }
thiserror = "2"
wasm-bindgen = { version = "0.2", optional = true }
serde-wasm-bindgen = { version = "0.6", optional = true }
```

---

## Examples & Usage

### Loading a form from a JSON file

```rust
use walrs_form_serde::{JsonFormLoader, FormLoader};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let form = JsonFormLoader::load_from_file(Path::new("forms/registration.json"))?;
    println!("Loaded form: {}", form.name.as_deref().unwrap_or("unnamed"));
    println!("Elements: {}", form.elements.as_ref().map_or(0, |e| e.len()));
    Ok(())
}
```

### Generating JSON Schema + TypeScript from a Fieldset-derived struct

```rust
use walrs_form::IntoFieldset;
use walrs_form_serde::{JsonSchemaGenerator, SchemaGenerator, TypeScriptGenerator, TsGenerator};

#[derive(walrs_form_derive::Fieldset)]
#[fieldset(legend = "Contact")]
struct ContactForm {
    #[field(type = "text", label = "Name", required)]
    name: String,

    #[field(type = "email", label = "Email", required)]
    email: String,

    #[field(textarea, label = "Message", rows = 5)]
    message: String,
}

fn main() {
    // Build a Form from the derived fieldset
    let mut form = walrs_form::Form::new("contact");
    form.add_element(ContactForm::fieldset().into());

    // Generate JSON Schema
    let schema = JsonSchemaGenerator::generate_schema(&form);
    println!("JSON Schema:\n{}", serde_json::to_string_pretty(&schema).unwrap());

    // Generate TypeScript interfaces
    let ts = TypeScriptGenerator::generate_ts(&form);
    println!("TypeScript:\n{}", ts);
}
```

### YAML round-trip

```rust
use walrs_form_serde::{YamlFormLoader, YamlFormEmitter, FormLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = std::fs::read_to_string("forms/registration.yaml")?;
    let form = YamlFormLoader::load_from_str(&yaml)?;

    // Modify form...

    let output = YamlFormEmitter::emit(&form)?;
    std::fs::write("forms/registration_updated.yaml", output)?;
    Ok(())
}
```

---

## Open Questions

1. **JSON Schema dialect** — Should we target Draft 2020-12, Draft 7, or make
   it configurable? (Recommendation: 2020-12 with a builder option to select
   the dialect.)

2. **TypeScript generation naming** — How should nested interface names be
   formed? `{FormName}{FieldsetName}` (e.g., `RegistrationFormAddress`) or
   flat (e.g., `Address`)? Flat risks collisions.

3. **Validation metadata in schema** — Should the JSON Schema include
   `minLength`, `maxLength`, `pattern`, `minimum`, `maximum` etc. that come
   from `Filterable` validation rules? If so, the form serde crate would need
   to understand `walrs_fieldfilter` types, creating a dependency. Alternative:
   a separate `walrs_form_schema` integration crate.

4. **Form versioning** — Should the JSON/YAML format include a `version` field
   for forward compatibility?

---

## Out of Scope

1. **Form rendering** — This crate produces data structures and schemas, not
   HTML. Rendering is handled by template engines or frontend frameworks.
2. **Database storage** — Persisting forms to a database is outside this crate's
   scope.
3. **Runtime validation** — Validation is handled by `walrs_fieldfilter` /
   `Filterable`. This crate only generates _schemas_ that describe validation
   constraints.
4. **Custom schema extensions** — OpenAPI / Swagger integration is a separate
   concern.

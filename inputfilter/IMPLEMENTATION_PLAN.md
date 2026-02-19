# ECMS Form Ecosystem Implementation Plan

**Date:** February 17, 2026  
**Updated:** February 18, 2026  
**Status:** Implementation Complete (except walrs_form_serde)  
**Goal:** Implement a holistic form ecosystem for validation, filtering, and serialization supporting both frontend (WASM) and backend web applications.

---

## Implementation Status

| Step | Component | Status |
|------|-----------|--------|
| Step 1 | `walrs_form_core` crate | ✅ Complete |
| Step 2 | `Field<T>` in `walrs_inputfilter` | ✅ Complete |
| Step 3 | `FieldFilter` multi-field validation | ✅ Complete |
| Step 4 | `walrs_form` crate | ✅ Complete |
| Step 5 | `walrs_form_serde` crate | ⏳ Not Implemented |

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture Summary](#architecture-summary)
3. [Implementation Steps](#implementation-steps)
4. [Further Considerations](#further-considerations)
5. [Dependencies](#dependencies)
6. [Timeline and Phases](#timeline-and-phases)

---

## Overview

This plan outlines the implementation of the ECMS form ecosystem, replacing the old closure-based `Input`/`RefInput` with a new unified `Field<T>` design based on `Rule<T>` enum composition. The ecosystem will support:

- **Backend**: Actix-web, Axum, and other Rust web frameworks
- **Frontend**: WASM compilation for browser-side validation
- **Config-driven**: YAML/JSON form definitions
- **Type-safe**: Strong typing with serialization support

### Key Design Decisions

1. **Unified Field API**: Single `Field<T>` struct replaces `Input`/`RefInput` split
2. **Rules as Data**: `Rule<T>` enum instead of closure references for serializability
3. **Element Type Discrimination**: Tuple variants `Element::Input(InputType, InputElement)` for pattern matching
4. **WASM-first**: All crates support WASM compilation with JavaScript interop
5. **Minimal Element Structs**: Only essential fields; `Attributes` for additional HTML attributes

---

## Architecture Summary

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                              walrs_form                                       │
│         (Form, Fieldset, Element - structure & validation binding)           │
├──────────────────────────────────────────────────────────────────────────────┤
│              │                    │                    │                      │
│              ▼                    ▼                    ▼                      │
│  ┌───────────────────┐   ┌───────────────┐    ┌───────────────┐              │
│  │  walrs_inputfilter │   │walrs_form_    │    │walrs_form_    │              │
│  │  (Field<T>,        │   │serde          │    │core           │              │
│  │   FieldFilter,     │   │(JSON/YAML     │    │(Value,        │              │
│  │   Filter<T> enum)  │   │ loading)      │    │ Attributes)   │              │
│  └─────────┬──────────┘   └───────────────┘    └───────────────┘              │
│            │                                                                  │
│    ┌───────┴───────┐                                                          │
│    │               │                                                          │
│    ▼               ▼                                                          │
│  ┌─────────────┐ ┌─────────────┐                                              │
│  │walrs_       │ │walrs_       │                                              │
│  │validator    │ │filter       │                                              │
│  │(Rule<T>     │ │(SlugFilter, │                                              │
│  │ enum impl)  │ │ StripTags)  │                                              │
│  └─────────────┘ └─────────────┘                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Crate Responsibilities

| Crate | Responsibility | Key Types |
|-------|----------------|-----------|
| `walrs_form_core` | Shared types, re-exports | `Value` (re-export of `serde_json::Value`), `ValueExt`, `Attributes` |
| `walrs_validator` | Individual validator implementations | `Rule<T>` enum with validation logic |
| `walrs_filter` | Individual filter implementations | `SlugFilter`, `StripTagsFilter`, `XmlEntitiesFilter` |
| `walrs_inputfilter` | Field-level validation & filtering | `Field<T>`, `FieldFilter`, `Filter<T>` enum, cross-field rules |
| `walrs_form` | Form structure and elements | `Form`, `Fieldset`, `Element` enum, `FormData`, element structs |
| `walrs_form_serde` | Serialization/deserialization | YAML/JSON loading, JSON Schema, TypeScript generation |

---

## Implementation Steps

### Step 1: Create `walrs_form_core` Crate

**Purpose:** Shared foundation types and re-exports (notably `serde_json::Value`) for use across all form-related crates.

**Files to Create:**

#### 1.1 `walrs/form_core/Cargo.toml`
```toml
[package]
name = "walrs_form_core"
version = "0.1.0"
edition = "2024"
authors = ["Ely De La Cruz <elycruz@elycruz.com>"]
description = "Core types for walrs form ecosystem"
license = "MIT OR Apache-2.0"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[features]
default = []
```

#### 1.2 `walrs/form_core/src/value.rs`
```rust
/// Re-export serde_json::Value as our dynamic value type for form data.
/// This provides all necessary features: serialization, helper methods
/// (as_str, as_i64, as_f64, as_bool, as_array, as_object), and From impls.
pub use serde_json::Value;

/// Extension trait for Value to add form-specific helper methods
pub trait ValueExt {
    /// Checks if the value is "empty" (null, empty string, empty array, or empty object)
    fn is_empty_value(&self) -> bool;
}

impl ValueExt for Value {
    fn is_empty_value(&self) -> bool {
        match self {
            Value::Null => true,
            Value::String(s) => s.is_empty(),
            Value::Array(arr) => arr.is_empty(),
            Value::Object(obj) => obj.is_empty(),
            _ => false,
        }
    }
}
```

**Features:**
- Uses `serde_json::Value` directly — no custom enum needed
- `serde_json::Value` already provides:
  - Transparent JSON serialization/deserialization
  - Conversion traits: `From<String>`, `From<&str>`, `From<i32>`, `From<i64>`, `From<f64>`, `From<bool>`, `From<Vec<Value>>`, `From<serde_json::Map<String, Value>>`
  - Helper methods: `as_str()`, `as_bool()`, `as_i64()`, `as_f64()`, `as_array()`, `as_object()`, `is_null()`, etc.
- `ValueExt` trait adds `is_empty_value()` for form-specific "empty" checking

#### 1.3 `walrs/form_core/src/attributes.rs`
```rust
/// HTML attributes storage
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Attributes(HashMap<String, String>);
```

**Features:**
- Methods: `new()`, `insert()`, `get()`, `remove()`, `contains_key()`, `iter()`
- `to_html() -> String` renders as space-separated HTML (e.g., `class="form-control" id="email"`)
- Proper HTML escaping for attribute values

#### 1.4 `walrs/form_core/src/lib.rs`
```rust
pub mod value;
pub mod attributes;

pub use value::{Value, ValueExt};
pub use attributes::Attributes;
```

---

### Step 2: Refactor `walrs_inputfilter` to New Unified `Field<T>` Design

**Purpose:** Replace old `Input`/`RefInput` with new `Field<T>` struct using `Rule<T>` enum.

**Files to Create/Modify:**

#### 2.1 `walrs/inputfilter/src/field.rs`
```rust
/// Validation configuration for a single field
#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
#[builder(setter(into, strip_option))]
pub struct Field<T> {
    pub name: Option<String>,
    
    #[builder(default = "false")]
    pub required: bool,

    // TODO: Do we need to add a `default_value`, or `get_default_value`, field or is  default value functionality handled for us by `Rule<T>`?
  
    #[builder(default = "None")]
    pub locale: Option<String>,
  
    #[builder(default)]
    pub rules: Vec<Rule<T>>,
    
    #[builder(default)]
    pub filters: Vec<Filter<T>>,
    
    /// Stops validation at first error when true
    #[builder(default = "false")]
    pub break_on_failure: bool, 
}
```

**Methods:**
- `validate(&self, value: &T) -> Result<(), Violations>` - Applies all rules, collecting violations unless break_on_failure
- `filter(&self, value: T) -> T` - Applies filters sequentially
- `process(&self, value: T) -> Result<T, Violations>` - Filter-then-validate pipeline
- Builder pattern support via `derive_builder`

#### 2.2 `walrs/inputfilter/src/field_filter.rs`
```rust
/// Multi-field validation for forms
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldFilter {
    pub fields: HashMap<String, Field<Value>>,
    pub cross_field_rules: Vec<CrossFieldRule>,
}

/// Cross-field validation rule
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrossFieldRule {
    pub name: Option<String>,
    pub fields: Vec<String>,
    pub rule: CrossFieldRuleType,
}

/// Types of cross-field validation
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CrossFieldRuleType {
    /// Both fields must have equal values (e.g., password confirmation)
    FieldsEqual { 
        field_a: String, 
        field_b: String 
    },
    
    /// Field is required if condition on another field is met
    RequiredIf { 
        field: String, 
        condition: Condition<Value> 
    },
    
    /// Field is required unless condition on another field is met
    RequiredUnless { 
        field: String, 
        condition: Condition<Value> 
    },
    
    /// At least one of the listed fields must have a value
    OneOfRequired(Vec<String>),
    
    /// Only one of the listed fields can have a value
    MutuallyExclusive(Vec<String>),
    
    /// If depends_on field has value, then field is required
    DependentRequired { 
        field: String, 
        depends_on: String 
    },
    
    /// Custom validation (not serializable)
    #[serde(skip)]
    Custom(Arc<dyn Fn(&FormData) -> RuleResult + Send + Sync>),
}
```

**Methods:**
- `FieldFilter::validate(&self, data: &FormData) -> Result<(), FormViolations>` - Validates all fields and cross-field rules
- `FieldFilter::add_field(&mut self, field: Field<Value>)`
- `FieldFilter::add_cross_field_rule(&mut self, rule: CrossFieldRule)`

#### 2.3 Deprecate Old API in `walrs/inputfilter/src/input.rs` and `ref_input.rs`
```rust
#[deprecated(since = "0.2.0", note = "Use Field<T> instead")]
pub struct Input<'a, T, FilterT = T> { ... }

#[deprecated(since = "0.2.0", note = "Use Field<T> instead")]
pub struct RefInput<'a, 'b, T, FT = T> { ... }
```

#### 2.4 `walrs/inputfilter/MIGRATION.md`
Create migration guide showing:
- How to convert closure-based validators to `Rule<T>` enum
- Replacing `Input<'a, T>` with `Field<T>`
- Replacing `RefInput<'a, 'b, T>` with `Field<T>`
- Examples of before/after code
- Breaking changes and migration timeline

#### 2.5 Update `walrs/inputfilter/src/lib.rs`
```rust
pub mod field;
pub mod field_filter;
// ... existing modules

pub use field::*;
pub use field_filter::*;
```

---

### Step 3: Implement `Filter<T>` Enum in `walrs_inputfilter`

**Purpose:** Serializable filter enum that delegates to existing `walrs_filter` implementations.

#### 3.1 `walrs/inputfilter/src/filter_enum.rs`
```rust
/// A composable value transformer
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Filter<T> {
    // ---- String Filters ----
    /// Trim whitespace from start and end
    Trim,
    
    /// Convert to lowercase
    Lowercase,
    
    /// Convert to uppercase
    Uppercase,
    
    /// Remove HTML tags using Ammonia
    StripTags,
    
    /// Encode special characters as XML/HTML entities
    HtmlEntities,
    
    /// Convert to URL-friendly slug
    Slug { 
        max_length: Option<usize> 
    },
    
    // ---- Numeric Filters ----
    /// Clamp value to range
    Clamp { 
        min: T, 
        max: T 
    },
    
    // ---- Composite ----
    /// Apply filters sequentially: f3(f2(f1(value)))
    Chain(Vec<Filter<T>>),
    
    // ---- Custom ----
    /// Custom filter function (not serializable)
    #[serde(skip)]
    Custom(Arc<dyn Fn(T) -> T + Send + Sync>),
}
```

**Methods:**
- `apply(&self, value: T) -> T` - Matches on variant and delegates to walrs_filter implementations
- Integration with `SlugFilter`, `StripTagsFilter`, `XmlEntitiesFilter` from `walrs_filter` crate
- Arc-wrapped filter instances for thread-safe reuse

#### 3.2 Update `walrs/inputfilter/Cargo.toml`
Add dependency:
```toml
walrs_filter = { path = "../filter" }
walrs_form_core = { path = "../form_core" }
```

---

### Step 4: Create `walrs_form` Crate Structure

**Purpose:** Form elements, structure, and data binding with WASM support.

#### 4.1 `walrs/form/Cargo.toml`
```toml
[package]
name = "walrs_form"
version = "0.1.0"
edition = "2024"
authors = ["Ely De La Cruz <elycruz@elycruz.com>"]
description = "Form elements and structure for walrs form ecosystem"
license = "MIT OR Apache-2.0"

[dependencies]
walrs_form_core = { path = "../form_core" }
walrs_inputfilter = { path = "../inputfilter" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["FormData"] }
serde-wasm-bindgen = "0.6"
js-sys = "0.3"

[features]
default = ["std"]
std = []
wasm = ["wasm-bindgen", "web-sys", "serde-wasm-bindgen"]
```

#### 4.2 Type Enums

**`walrs/form/src/input_type.rs`:**
```rust
/// HTML5 input types
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Text,
    Email,
    Password,
    Number,
    Checkbox,
    Radio,
    File, // for file upload validation see laminas framework laminas-form for inspiration
    Date,
    #[serde(rename = "datetime-local")]
    DateTime,
    Month,
    Week,
    Time,
    Tel,
    Url,
    Color,
    Range,
    Search,
    Hidden,
}
```

**`walrs/form/src/select_type.rs`:**
```rust
/// Select element types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectType {
    #[serde(rename = "select")]
    Single,
    #[serde(rename = "select-multiple")]
    Multiple,
}
```

**`walrs/form/src/button_type.rs`:**
```rust
/// Button types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ButtonType {
    Submit,
    Reset,
    Button,
}
```

#### 4.3 Element Structs (Minimal Fields)

Element struct implementation rules:

**Note:** Textarea, Select, and Input elements must contain all of the following fields: `name`, `_type`, `attributes`, `field`, `value` (captures last value injected, and/or, which is being validated, or which is set directly by the user/builder), `validation_message` (captures last validation run's first validation [err] message), `help_message`, `label`,
`disabled` and `required`.  These fields should all be `Option`able except for the `_type` one.

The `attributes` fields need to be `Option<Attributes>` and lazily constructed
when it's needed.

Other criteria:

- All created Element structs should derive, and/or implement, a builder pattern.
- All option values should be skipped in serialization if they are `None` (using `#[serde(skip_serializing_if = "Option::is_none")]`, etc.).
- All creates Element structs should populate the `_type` field with variant that matches it's html counterpart (InputType::Text for InputElement, SelectType::Single for SelectElement, FieldsetType::Fieldset for FieldsetElement, etc.).

**`walrs/form/src/input_element.rs`:**
```rust
/// Input element
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputElement {
    pub name: String,
    
    #[serde(rename = "type")]
    pub _type: InputType,
    
    pub value: Option<Value>,
    pub attributes: Option<Attributes>,
    pub field: Option<Field<Value>>,
}

impl InputElement {
    /// Convenience validation method
    pub fn validate(&self, value: &Value) -> Result<(), Violations> {
        self.field.as_ref()
            .map(|f| f.validate(value))
            .unwrap_or(Ok(()))
    }
}
```

**`walrs/form/src/select_option.rs`:**
```rust
/// HTML option element
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectOption {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>, // Should be any type that has `ToString` impl
  
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// When populated serves parent `SelectOption` serves as an 'optgroup'.
    pub options: Option<Vec<SelectOption>>,
}
```

**`walrs/form/src/select_element.rs`:**
```rust
/// Select element
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
pub struct SelectElement {
    pub name: String,
    
    #[serde(rename = "type")]
    pub _type: SelectType,
    
    /// Single value for Single, array for Multiple
    pub value: Option<Value>,
    pub values: Option<Vec<Value>>,
    pub label: Option<String>,
    pub options: Vec<SelectOption>,
    pub attributes: Attributes,
    pub multiple: Option<bool>,
    pub required: Option<bool>,
    pub disabled: Option<bool>,
    pub field: Option<Field<Value>>,
    /// Stores first encountered validation message from
    /// each 'validate', and/or 'validate_all' call.
    pub validation_message: Option<String>,
    
    /// Help text to display below html representation of this form control.
    pub help_message: Option<String>
}

impl SelectElement {
    pub fn validate(&self, value: &Value) -> Result<(), Violations> {
        self.field.as_ref()
            .map(|f| f.validate(value))
            .unwrap_or(Ok(()))
    }
}
```

**`walrs/form/src/button_element.rs`:**Button and Fieldset elements must contain:
- `name`
- `attributes`
- `disabled`
```rust
/// Button element (no validation needed)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonElement {
    pub name: String,
    pub button_type: ButtonType,
    pub label: Option<String>,
    pub attributes: Option<Attributes>,
    pub disabled: Option<bool>,
}
```

**`walrs/form/src/textarea_element.rs`:**
```rust
/// Textarea element
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextareaElement {
    pub name: String,
    pub value: Option<String>,
    pub rows: Option<u32>,
    pub cols: Option<u32>,
    pub attributes: Attributes,
    pub field: Option<Field<Value>>,
}

impl TextareaElement {
    pub fn validate(&self, value: &Value) -> Result<(), Violations> {
        self.field.as_ref()
            .map(|f| f.validate(value))
            .unwrap_or(Ok(()))
    }
}
```

#### 4.4 Element Enum

**`walrs/form/src/element.rs`:**
```rust
/// Form element types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "element")]
pub enum Element {
    /// Button element (no type-specific validation)
    Button(ButtonElement),
    
    /// Input element with type discriminator for pattern matching
    Input(InputType, InputElement),
    
    /// Select element with type discriminator
    Select(SelectType, SelectElement),
    
    /// Textarea element (no type variants)
    Textarea(TextareaElement),
  
    // Fieldset element that contains other elements
    Fieldset(FieldsetElement),
}
```

**Pattern matching example:**
```rust
match element {
    Element::Input(InputType::Email, input) => validate_email(input),
    Element::Select(SelectType::Multiple, select) => handle_multi_select(select),
    Element::Button(_) => skip_validation(),
    Element::Textarea(_) => validate_text(),
}
```

#### 4.5 FormData with Path Resolution

**`walrs/form/src/path.rs`:**
```rust
/// Path segment types
#[derive(Debug, PartialEq)]
pub enum PathSegment {
    Field(String),
    Index(usize),
}

/// Parse path string into segments
/// Supports: "user.email", "items[0]", "items[0].name"
pub fn parse_path(path: &str) -> Result<Vec<PathSegment>, PathError> {
    // Implementation parses dot notation and array indices
    // Returns error on invalid syntax
}
```

**`walrs/form/src/form_data.rs`:**
```rust
/// Form data transfer object
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FormData(HashMap<String, Value>);

impl FormData {
    pub fn new() -> Self { ... }
    
    /// Get value by path (dot notation and array indexing)
    /// Returns None if path doesn't exist or index out of bounds
    pub fn get(&self, path: &str) -> Option<&Value> { ... }
    
    pub fn get_mut(&mut self, path: &str) -> Option<&mut Value> { ... }
    
    /// Set value by path, creating intermediate structures
    pub fn set(&mut self, path: &str, value: Value) { ... }
    
    pub fn insert(&mut self, key: String, value: Value) { ... }
    
    pub fn merge(&mut self, other: FormData) { ... }
}

// Conversion traits
impl From<HashMap<String, String>> for FormData { ... }
impl From<HashMap<String, Value>> for FormData { ... }
impl From<serde_json::Value> for FormData { ... }

// WASM-specific
#[cfg(target_arch = "wasm32")]
impl TryFrom<web_sys::FormData> for FormData {
    type Error = FormDataError;
    
    fn try_from(js_form_data: web_sys::FormData) -> Result<Self, Self::Error> {
        // Iterate web_sys::FormData entries
        // Return error if file entries encountered
        // Convert other entries to Value
    }
}
```

**Path resolution rules:**
- Dot notation: `"user.email"` → nested object access
- Array indexing: `"items[0]"` → single integer index only
- Combined: `"items[0].name"` → array then object
- Out of bounds: Returns `None`
- Allows negative indexing
- No support for array range syntax

#### 4.6 Fieldset and Form

**`walrs/form/src/fieldset.rs`:**
```rust
/// Group of related form elements
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fieldset { // TODO: Rename to `FieldsetElement` 
    pub name: Option<String>,
    pub legend: Option<String>,
    pub disabled: Option<bool>,
    pub elements: Option<Vec<Element>>,
    pub attributes: Option<Attributes>,
}

impl Fieldset {
    /// Recursively iterate all elements including nested fieldsets
    pub fn iter_elements(&self) -> impl Iterator<Item = &Element> { ... }
    
    /// Get element by path
    pub fn get(&self, path: &str) -> Option<&Element> { ... }
    
    /// Validate all elements
    pub fn validate(&self, data: &FormData) -> Result<(), FormViolations> { ... }
}
```

**`walrs/form/src/form.rs`:**
```rust
/// HTML form
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Form {
    // Fields that are non (at serialization time) should not show up in JSON string
    // ----
    pub name: Option<String>,
    pub action: Option<String>,
    pub method: Option<FormMethod>,
    pub enctype: Option<FormEnctype>,
    pub elements: Option<Vec<Element>>,    // Lazily created
    pub attributes: Option<Attributes>,    // Lazily created,
  
    // Should not show up in JSON
    pub field_filter: Option<FieldFilter>, 
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormMethod {
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "POST")]
    Post,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormEnctype {
    #[serde(rename = "application/x-www-form-urlencoded")]
    UrlEncoded,
    #[serde(rename = "multipart/form-data")]
    MultipartFormData,
    #[serde(rename = "text/plain")]
    TextPlain,
}

impl Form {
    /// Create form and eagerly populate element values from FormData
    pub fn from_data(data: FormData) -> Self { ... }
    
    /// Bind data to existing form (in-place hydration)
    pub fn bind_data(&mut self, data: FormData) { ... }
    
    /// Validate form data
    pub fn validate(&self, data: &FormData) -> Result<(), FormViolations> { ... }
    
    /// Get element by path
    pub fn get_element(&self, path: &str) -> Option<&Element> { ... }
}
```

#### 4.7 WASM Bindings

TODO: Change all WASM binding name prefixes to `Walrs*`, instead of `Js*`.

**`walrs/form/src/wasm.rs`:**
```rust
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

/// JavaScript-compatible wrapper for Form
#[wasm_bindgen]
pub struct JsForm {
    inner: Form,
}

#[wasm_bindgen]
impl JsForm {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String) -> Self { ... }
    
    #[wasm_bindgen(js_name = fromData)]
    pub fn from_data(js_form_data: web_sys::FormData) -> Result<JsForm, JsValue> {
        let form_data = FormData::try_from(js_form_data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(JsForm { inner: Form::from_data(form_data) })
    }
    
    #[wasm_bindgen(js_name = validate)]
    pub fn validate(&self, js_form_data: web_sys::FormData) -> Result<(), JsValue> {
        let form_data = FormData::try_from(js_form_data)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner.validate(&form_data)
            .map_err(|violations| {
                serde_wasm_bindgen::to_value(&violations).unwrap()
            })
    }
}

/// JavaScript-compatible wrapper for FormData
#[wasm_bindgen]
pub struct JsFormData {
    inner: FormData,
}

#[wasm_bindgen]
impl JsFormData {
    #[wasm_bindgen(js_name = tryFromJsFormData)]
    pub fn try_from_js_form_data(js_form_data: web_sys::FormData) -> Result<JsFormData, JsValue> {
        FormData::try_from(js_form_data)
            .map(|inner| JsFormData { inner })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    pub fn get(&self, path: &str) -> JsValue {
        self.inner.get(path)
            .map(|v| serde_wasm_bindgen::to_value(v).unwrap())
            .unwrap_or(JsValue::NULL)
    }
}

// Similar wrappers for JsElement, JsFieldset, etc.
```

#### 4.8 Build Script

**`walrs/form/ci-cd-wasm.sh`:**
```bash
#!/bin/bash
set -e

echo "Building walrs_form for WASM..."

# Build for web (browser ESM)
wasm-pack build --target web --features wasm --no-default-features

# Build for Node.js
wasm-pack build --target nodejs --features wasm --no-default-features --out-dir pkg-node

# Build for bundlers
wasm-pack build --target bundler --features wasm --no-default-features --out-dir pkg-bundler

echo "WASM build complete!"
```

---

### Step 5: Implement `walrs_form_serde` Crate with WASM Support

**Purpose:** Form serialization, loading, and code generation.

#### 5.1 `walrs/form_serde/Cargo.toml`
```toml
[package]
name = "walrs_form_serde"
version = "0.1.0"
edition = "2024"
authors = ["Ely De La Cruz <elycruz@elycruz.com>"]
description = "Serialization and code generation for walrs forms"
license = "MIT OR Apache-2.0"

[dependencies]
walrs_form = { path = "../form" }
walrs_form_core = { path = "../form_core" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
thiserror = "1.0"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
serde-wasm-bindgen = "0.6"

[features]
default = ["std"]
std = []
wasm = ["wasm-bindgen", "serde-wasm-bindgen"]
```

#### 5.2 Form Loader

**`walrs/form_serde/src/loader.rs`:**
```rust
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Unsupported format")]
    UnsupportedFormat,
}

pub trait FormLoader {
    fn load_yaml(yaml: &str) -> Result<Form, FormLoadError>;
    fn load_json(json: &str) -> Result<Form, FormLoadError>;
    fn load_file(path: &Path) -> Result<Form, FormLoadError>;
}

impl FormLoader for Form {
    fn load_yaml(yaml: &str) -> Result<Form, FormLoadError> {
        serde_yaml::from_str(yaml).map_err(FormLoadError::from)
    }
    
    fn load_json(json: &str) -> Result<Form, FormLoadError> {
        serde_json::from_str(json).map_err(FormLoadError::from)
    }
    
    fn load_file(path: &Path) -> Result<Form, FormLoadError> {
        let content = std::fs::read_to_string(path)?;
        match path.extension().and_then(|e| e.to_str()) {
            Some("yaml") | Some("yml") => Self::load_yaml(&content),
            Some("json") => Self::load_json(&content),
            _ => Err(FormLoadError::UnsupportedFormat),
        }
    }
}
```

#### 5.3 JSON Schema Generation

**`walrs/form_serde/src/schema.rs`:**
```rust
pub struct JsonSchemaGenerator;

impl JsonSchemaGenerator {
    pub fn generate(&self, form: &Form) -> serde_json::Value {
        // Traverse form structure
        // Map InputType to JSON Schema types:
        //   InputType::Email -> { "type": "string", "format": "email" }
        //   InputType::Number -> { "type": "number" }
        //   InputType::Date -> { "type": "string", "format": "date" }
        // Map Rule<T> to validation keywords:
        //   Rule::MinLength(5) -> { "minLength": 5 }
        //   Rule::Max(100) -> { "maximum": 100 }
        //   Rule::Pattern(regex) -> { "pattern": regex }
        // Generate "required" array from required fields
        // Generate "properties" object from elements
    }
}
```

#### 5.4 TypeScript Generation

**`walrs/form_serde/src/typescript.rs`:**
```rust
pub struct TypeScriptGenerator;

impl TypeScriptGenerator {
    pub fn generate(&self, form: &Form) -> String {
        // Generate TypeScript interface from form
        // Map InputType to TS types:
        //   InputType::Email -> string
        //   InputType::Number -> number
        //   InputType::Checkbox -> boolean
        //   SelectType::Multiple -> string[]
        // Include JSDoc comments for validation rules
        // Example output:
        // /**
        //  * User registration form
        //  */
        // interface UserRegistrationData {
        //   /** @minLength 3 @maxLength 20 */
        //   username: string;
        //   /** @format email */
        //   email: string;
        //   /** @minimum 18 */
        //   age: number;
        // }
    }
}
```

#### 5.5 WASM Bindings

**`walrs/form_serde/src/wasm.rs`:**
```rust
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct JsFormLoader;

#[wasm_bindgen]
impl JsFormLoader {
    #[wasm_bindgen(js_name = loadJson)]
    pub fn load_json(json: &str) -> Result<JsValue, JsValue> {
        Form::load_json(json)
            .map(|form| serde_wasm_bindgen::to_value(&form).unwrap())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[wasm_bindgen]
pub struct JsSchemaGenerator;

#[wasm_bindgen]
impl JsSchemaGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        JsSchemaGenerator
    }
    
    #[wasm_bindgen(js_name = generate)]
    pub fn generate(&self, form: JsValue) -> Result<JsValue, JsValue> {
        let form: Form = serde_wasm_bindgen::from_value(form)?;
        let generator = JsonSchemaGenerator;
        Ok(serde_wasm_bindgen::to_value(&generator.generate(&form)).unwrap())
    }
}

#[wasm_bindgen]
pub struct JsTypeScriptGenerator;

#[wasm_bindgen]
impl JsTypeScriptGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        JsTypeScriptGenerator
    }
    
    #[wasm_bindgen(js_name = generate)]
    pub fn generate(&self, form: JsValue) -> Result<String, JsValue> {
        let form: Form = serde_wasm_bindgen::from_value(form)?;
        let generator = TypeScriptGenerator;
        Ok(generator.generate(&form))
    }
}
```

#### 5.6 Build Script

**`walrs/form_serde/ci-cd-wasm.sh`:**
```bash
#!/bin/bash
set -e

echo "Building walrs_form_serde for WASM..."

wasm-pack build --target web --features wasm --no-default-features
wasm-pack build --target nodejs --features wasm --no-default-features --out-dir pkg-node
wasm-pack build --target bundler --features wasm --no-default-features --out-dir pkg-bundler

echo "WASM build complete!"
```

---

### Step 6: Create Integration Examples, Benchmarks, and Tests

#### 6.1 Examples

**`walrs/form/examples/actix_form.rs`:**
```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use walrs_form::{Form, FormData};

async fn handle_form(form_values: web::Form<HashMap<String, String>>) -> HttpResponse {
    // Convert to FormData
    let form_data = FormData::from(form_values.into_inner());
    
    // Load form definition
    let form = Form::load_file("forms/user_registration.yaml").unwrap();
    
    // Validate
    match form.validate(&form_data) {
        Ok(()) => HttpResponse::Ok().json({"status": "success"}),
        Err(violations) => HttpResponse::BadRequest().json(violations),
    }
}
```

**`walrs/form/examples/axum_form.rs`:**
```rust
use axum::{Form, Json, response::IntoResponse};
use walrs_form::FormData;

async fn handle_form(Form(data): Form<HashMap<String, String>>) -> impl IntoResponse {
    let form_data = FormData::from(data);
    // Similar to Actix example
}
```

**`walrs/form/examples/yaml_form.rs`:**
```rust
// Load form from YAML config
let form = Form::load_file("examples/user_form.yaml").unwrap();

// Bind data
let mut data = FormData::new();
data.set("username", "john_doe".into());
data.set("email", "john@example.com".into());

// Validate
match form.validate(&data) {
    Ok(()) => println!("Valid!"),
    Err(e) => println!("Errors: {:?}", e),
}
```

**`walrs/form/examples/wasm_form.html`:**
```html
<!DOCTYPE html>
<html>
<head>
    <title>WASM Form Validation</title>
</head>
<body>
    <form id="userForm">
        <input type="text" name="username" placeholder="Username">
        <input type="email" name="email" placeholder="Email">
        <input type="number" name="age" placeholder="Age">
        <select name="country">
            <option value="us">United States</option>
            <option value="uk">United Kingdom</option>
        </select>
        <textarea name="bio" placeholder="Bio"></textarea>
        <button type="submit">Submit</button>
    </form>
    <div id="errors"></div>
    
    <script type="module">
        import init, { JsForm } from './pkg/walrs_form.js';
        
        await init();
        
        const form = document.getElementById('userForm');
        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            
            const formData = new FormData(form);
            const jsForm = JsForm.fromData(formData);
            
            try {
                jsForm.validate(formData);
                alert('Form is valid!');
            } catch (violations) {
                document.getElementById('errors').innerHTML = 
                    JSON.stringify(violations, null, 2);
            }
        });
    </script>
</body>
</html>
```

#### 6.2 Benchmarks

**`walrs/inputfilter/benches/field_vs_input_benchmark.rs`:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use walrs_inputfilter::{Field, FieldBuilder, Rule, Input, InputBuilder};

fn bench_field_validation(c: &mut Criterion) {
    let field = FieldBuilder::default()
        .name("email".to_string())
        .rule(Rule::Required.and(Rule::Email).and(Rule::MinLength(5)))
        .build()
        .unwrap();
    
    c.bench_function("Field<String> validation", |b| {
        b.iter(|| {
            field.validate(black_box(&"test@example.com".to_string()))
        })
    });
}

fn bench_input_validation(c: &mut Criterion) {
    let email_validator = |s: &str| {
        if s.contains('@') { Ok(()) } 
        else { Err(Violation::new(TypeMismatch, "Invalid email")) }
    };
    
    let input = InputBuilder::default()
        .required(true)
        .validators(vec![&email_validator])
        .build()
        .unwrap();
    
    c.bench_function("Old Input validation", |b| {
        b.iter(|| {
            input.validate(black_box("test@example.com"))
        })
    });
}

criterion_group!(benches, bench_field_validation, bench_input_validation);
criterion_main!(benches);
```

#### 6.3 Tests

**`walrs/inputfilter/tests/field_test.rs`:**
```rust
#[test]
fn test_field_validation() {
    let field = FieldBuilder::default()
        .name("username".to_string())
        .rule(Rule::MinLength(3).and(Rule::MaxLength(20)))
        .build()
        .unwrap();
    
    assert!(field.validate(&"john".to_string()).is_ok());
    assert!(field.validate(&"jo".to_string()).is_err());
}

#[test]
fn test_field_with_filters() {
    let field = FieldBuilder::default()
        .name("email".to_string())
        .filters(vec![Filter::Trim, Filter::Lowercase])
        .build()
        .unwrap();
    
    let result = field.filter("  TEST@EXAMPLE.COM  ".to_string());
    assert_eq!(result, "test@example.com");
}
```

**`walrs/inputfilter/tests/field_filter_test.rs`:**
```rust
#[test]
fn test_one_of_required() {
    let mut field_filter = FieldFilter::new();
    field_filter.add_cross_field_rule(CrossFieldRule {
        name: "contact_required".to_string(),
        fields: vec!["email".to_string(), "phone".to_string()],
        rule: CrossFieldRuleType::OneOfRequired(vec![
            "email".to_string(),
            "phone".to_string(),
        ]),
    });
    
    let mut data = FormData::new();
    assert!(field_filter.validate(&data).is_err()); // Neither present
    
    data.set("email", "test@example.com".into());
    assert!(field_filter.validate(&data).is_ok()); // Email present
}

#[test]
fn test_mutually_exclusive() {
    // Test that only one of multiple fields can be filled
}

#[test]
fn test_dependent_required() {
    // Test conditional required fields
}
```

**`walrs/form/tests/form_data_test.rs`:**
```rust
#[test]
fn test_dot_notation() {
    let mut data = FormData::new();
    data.set("user.email", "test@example.com".into());
    
    assert_eq!(
        data.get("user.email").unwrap().as_str(),
        Some("test@example.com")
    );
}

#[test]
fn test_array_indexing() {
    let mut data = FormData::new();
    data.set("items[0].name", "Item 1".into());
    data.set("items[1].name", "Item 2".into());
    
    assert_eq!(
        data.get("items[0].name").unwrap().as_str(),
        Some("Item 1")
    );
    
    // Out of bounds
    assert!(data.get("items[99].name").is_none());
}
```

**`walrs/form/tests/element_pattern_matching_test.rs`:**
```rust
#[test]
fn test_input_type_matching() {
    let element = Element::Input(
        InputType::Email,
        InputElement { /* ... */ }
    );
    
    match element {
        Element::Input(InputType::Email, input) => {
            // Email-specific handling
            assert_eq!(input._type, InputType::Email);
        }
        _ => panic!("Should match email input"),
    }
}
```

**`walrs/form_serde/tests/serde_test.rs`:**
```rust
#[test]
fn test_form_yaml_roundtrip() {
    let form = Form { /* ... */ };
    
    let yaml = serde_yaml::to_string(&form).unwrap();
    let deserialized: Form = serde_yaml::from_str(&yaml).unwrap();
    
    assert_eq!(form.name, deserialized.name);
}

#[test]
fn test_input_type_serialization() {
    let input_type = InputType::Email;
    let json = serde_json::to_string(&input_type).unwrap();
    assert_eq!(json, r#""email""#);
}
```

**`walrs/form/tests-js/form.test.js`:**
```javascript
import { describe, it, expect } from 'vitest';
import init, { JsForm, JsFormData } from '../pkg/walrs_form.js';

describe('WASM Form Validation', () => {
    beforeAll(async () => {
        await init();
    });
    
    it('should validate FormData', () => {
        const formData = new FormData();
        formData.append('email', 'test@example.com');
        
        const jsFormData = JsFormData.tryFromJsFormData(formData);
        expect(jsFormData.get('email')).toBe('test@example.com');
    });
    
    it('should reject file inputs', () => {
        const formData = new FormData();
        const file = new File(['content'], 'test.txt');
        formData.append('file', file);
        
        expect(() => {
            JsFormData.tryFromJsFormData(formData);
        }).toThrow();
    });
});
```

---

## Further Considerations

### 1. Path Parser Implementation

**Location:** `walrs/form/src/path.rs`

The `FormData::get(path)` and `set(path, value)` methods need a robust path parser:

```rust
#[derive(Debug, PartialEq)]
pub enum PathSegment {
    Field(String),
    Index(usize),
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
    
    #[error("Invalid index: {0}")]
    InvalidIndex(String),
}

pub fn parse_path(path: &str) -> Result<Vec<PathSegment>, PathError> {
    // Parse patterns:
    // - "field" -> [Field("field")]
    // - "user.email" -> [Field("user"), Field("email")]
    // - "items[0]" -> [Field("items"), Index(0)]
    // - "items[0].name" -> [Field("items"), Index(0), Field("name")]
}
```

**Decision:** Create as separate module for testability and reuse.

### 2. FormViolations Structure

**Question:** Where should `FormViolations` live?

**Options:**
- `walrs_validator`: Shared infrastructure (alongside `Violations`)
- `walrs_inputfilter`: With `FieldFilter` 
- `walrs_form`: Form-specific

**Recommendation:** Place in `walrs_inputfilter` since it's used by `FieldFilter` and represents multi-field validation errors.

```rust
// walrs/inputfilter/src/violations.rs
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FormViolations {
    /// Per-field violations
    pub fields: HashMap<String, Violations>,
    
    /// Cross-field violations
    pub form: Violations,
}

impl FormViolations {
    pub fn is_empty(&self) -> bool {
        self.fields.values().all(|v| v.is_empty()) && self.form.is_empty()
    }
    
    pub fn for_field(&self, name: &str) -> Option<&Violations> {
        self.fields.get(name)
    }
}
```

### 3. Element Name Uniqueness

**Question:** Should `Form` enforce unique element names?

**Decision:** No enforcement at the struct level, but provide helper methods:
- `get_element(name)` returns first match (like HTML)
- `get_all_elements(name)` returns all matches (for radio groups)
- Document that radio buttons share names

```rust
impl Form {
    /// Get first element with name (most common case)
    pub fn get_element(&self, name: &str) -> Option<&Element> { ... }
    
    /// Get all elements with name (for radio groups)
    pub fn get_all_elements(&self, name: &str) -> Vec<&Element> { ... }
}
```

### 4. WASM Build Scripts

**Decision:** Yes, create build scripts following `walrs_acl` pattern.

Files to create:
- `walrs/form/ci-cd-wasm.sh`
- `walrs/form_serde/ci-cd-wasm.sh`

Include targets:
- `web` (browser ESM)
- `nodejs` 
- `bundler` (webpack, rollup)

### 5. Default Values for Elements

**Question:** Should elements support default values?

**Decision:** Yes, add `default_value: Option<Value>` field to element structs.

```rust
pub struct InputElement {
    // ...existing fields
    pub default_value: Option<Value>,
}

impl Form {
    pub fn bind_data(&mut self, data: FormData) {
        // Use element.default_value when data doesn't contain element.name
    }
}
```

**Use case:** Pre-populate forms with default values when FormData is empty.

---


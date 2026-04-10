# Walrs Form Ecosystem Design
**Date:** February 14, 2026  
**Status:** Proposal  
**Goal:** Design a holistic crate ecosystem for form handling, validation, filtering, and serialization that supports both frontend and backend web applications.
---
## Table of Contents

1. [Overview](#overview)
2. [Laminas Form Architecture Analysis](#laminas-form-architecture-analysis)
3. [Proposed Crate Ecosystem](#proposed-crate-ecosystem)
4. [Crate 1: walrs_form_core (Shared Foundation)](#crate-1-walrs_form_core-shared-foundation)
5. [Crate 2: walrs_validation (Validator Structs)](#crate-2-walrs_validation-validator-structs)
6. [Crate 3: walrs_filter (Filter Structs)](#crate-3-walrs_filter-filter-structs)
7. [Crate 4: walrs_inputfilter (Composition Layer)](#crate-4-walrs_inputfilter-composition-layer)
8. [Crate 5: walrs_form (Form Elements & Structure)](#crate-5-walrs_form-form-elements--structure)
9. [Crate 6: walrs_form_view (Rendering)](#crate-6-walrs_form_view-rendering)
10. [Crate 7: walrs_form_serde (Serialization)](#crate-7-walrs_form_serde-serialization)
11. [Cross-Cutting Concerns](#cross-cutting-concerns)
12. [Data Flow Architecture](#data-flow-architecture)
13. [Rust Idioms & Functional Patterns](#rust-idioms--functional-patterns)
14. [Frontend Integration Strategy](#frontend-integration-strategy)
15. [Implementation Roadmap](#implementation-roadmap)

---
## Overview
### Vision
A set of composable Rust crates that provide:
1. **Validation & Filtering** - Rule-based validation with tree composition
2. **Form Structure** - Type-safe HTML form element representation
3. **Rendering** - HTML output generation
4. **Serialization** - JSON/YAML form definitions for frontend/backend sharing
### Design Philosophy
- **Rust-first idioms** - Enums over inheritance, traits over interfaces
- **Functional composition** - Combinators, immutable transformations
- **Zero-cost abstractions** - Compile-time guarantees where possible
- **Serialization-friendly** - Forms as data, shareable across stack
- **Thread-safe by default** - \`Arc\`-based sharing for web servers
---
## Laminas Form Architecture Analysis
### Laminas Component Structure
\`\`\`
laminas-form/
├── Element/           # Individual form elements (Input, Select, etc.)
├── Fieldset/          # Groups of elements
├── Form/              # Top-level form container
├── View/              # Rendering helpers
│   └── Helper/        # HTML generation
├── InputFilterProviderInterface  # Links elements to validation
└── Hydrator/          # Data binding (object <-> form)
laminas-inputfilter/
├── Input/             # Single field validation
├── InputFilter/       # Collection of inputs
├── Factory/           # Build from config
└── Validator/         # Validation rules
laminas-validator/
├── AbstractValidator  # Base class
├── StringLength       # Length validation
├── Regex              # Pattern matching
├── Callback           # Custom validation
└── ValidatorChain     # Multiple validators
\`\`\`
### Key Laminas Patterns
1. **InputFilterProviderInterface** - Elements define their own validation
2. **Hydrators** - Bind form data to/from objects
3. **Factory pattern** - Build forms from arrays/config
4. **View helpers** - Render elements to HTML
5. **Fieldsets** - Nestable element groups
### What We'll Adapt for Rust

| Laminas Concept | Rust Adaptation |
|-----------------|-----------------|
| Class inheritance | Enum variants + traits |
| laminas-validator | `walrs_validation` crate (validator structs) |
| laminas-filter | `walrs_filter` crate (filter structs) |
| laminas-inputfilter | `walrs_inputfilter` crate (`Rule<T>` enum + `Input`/`InputFilter`) |
| Element classes | `Element` enum in `walrs_form` |
| Fieldset | `Fieldset` struct with `Vec<Element>` |
| Factory | `serde` deserialization + builders |
| View helpers | `Render` trait in `walrs_form_view` |
| Hydrators | `serde` + `From`/`Into` traits |

---

## Proposed Crate Ecosystem

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                              walrs_form                                       │
│         (Form, Fieldset, Element - structure & validation binding)           │
├──────────────────────────────────────────────────────────────────────────────┤
│              │                    │                    │                      │
│              ▼                    ▼                    ▼                      │
│  ┌───────────────────┐   ┌───────────────┐    ┌───────────────┐              │
│  │  walrs_inputfilter │   │walrs_form_    │    │walrs_form_    │              │
│  │  (Rule<T> enum,    │   │view           │    │serde          │              │
│  │   Input, InputFil- │   │(HTML render)  │    │(JSON/YAML)    │              │
│  │   ter composition) │   └───────────────┘    └───────────────┘              │
│  └─────────┬──────────┘                                                       │
│            │                                                                  │
│    ┌───────┴───────┐                                                          │
│    │               │                                                          │
│    ▼               ▼                                                          │
│  ┌─────────────┐ ┌─────────────┐                                              │
│  │walrs_       │ │walrs_       │                                              │
│  │validator    │ │filter       │                                              │
│  │(validator   │ │(filter      │                                              │
│  │ structs)    │ │ structs)    │                                              │
│  └──────┬──────┘ └──────┬──────┘                                              │
│         │               │                                                     │
│         └───────┬───────┘                                                     │
│                 ▼                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐     │
│  │                      walrs_form_core                                 │     │
│  │  (Common types, traits, Violation, Value, Attributes)                │     │
│  └─────────────────────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Crate Dependency Graph

```
walrs_form_core         (no dependencies - shared types, traits)
    │
    ├── walrs_validation      (validator structs - LengthValidator, PatternValidator, etc.)
    │
    ├── walrs_filter         (filter structs - TrimFilter, SlugFilter, etc.)
    │
    ├── walrs_inputfilter    (Rule<T> enum, Input<T>, InputFilter - depends on validator & filter)
    │
    ├── walrs_form           (Form, Fieldset, Element - depends on inputfilter)
    │       │
    │       ├── walrs_form_view   (HTML rendering)
    │       │
    │       └── walrs_form_serde  (serialization)
    │
    └── walrs_form_derive    (proc macros, optional)
```

### Crate Responsibilities

| Crate | Responsibility | Key Types |
|-------|----------------|-----------|
| `walrs_form_core` | Shared types, traits, error types | `Value`, `Violation`, `Violations`, `Validate` trait, `Filter` trait |
| `walrs_validation` | Individual validator implementations | `LengthValidator`, `PatternValidator`, `RangeValidator`, `EqualityValidator` |
| `walrs_filter` | Individual filter implementations | `TrimFilter`, `SlugFilter`, `StripTagsFilter`, `XmlEntitiesFilter` |
| `walrs_inputfilter` | Composition layer - rules as data | `Rule<T>` enum, `Input<T>`, `InputFilter`, combinators |
| `walrs_form` | Form structure and element types | `Form`, `Fieldset`, `Element` enum, `Attributes` |
| `walrs_form_view` | HTML rendering | `Render` trait, `Html`, theme support |
| `walrs_form_serde` | Serialization/deserialization | YAML/JSON loading, JSON Schema generation |
| `walrs_form_derive` | Proc macros for ergonomics | `#[derive(Validate)]`, `#[derive(FormElement)]` |

### How Rule<T> and Validators Work Together

The `Rule<T>` enum from the "Fresh Approach" serves as a **serializable data representation** 
of validation rules, while the validator structs in `walrs_validation` provide the 
**implementation with full customization**.

```rust
// walrs_validation - Full-featured validator struct
pub struct LengthValidator<'a, T: ?Sized> {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub too_short_msg: &'a dyn Fn(&Self, &T) -> String,
    pub too_long_msg: &'a dyn Fn(&Self, &T) -> String,
}

// walrs_inputfilter - Serializable rule enum
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Rule<T> {
    MinLength(usize),
    MaxLength(usize),
    Length { min: Option<usize>, max: Option<usize> },
    All(Vec<Rule<T>>),   // Tree composition
    Any(Vec<Rule<T>>),   // OR logic
    // ... other variants
}

// Conversion: Rule -> Validator (for execution)
impl Rule<String> {
    pub fn to_validator(&self) -> Box<dyn Validate<String>> {
        match self {
            Rule::MinLength(n) => Box::new(LengthValidator::new().min_length(*n)),
            Rule::MaxLength(n) => Box::new(LengthValidator::new().max_length(*n)),
            Rule::Length { min, max } => Box::new(
                LengthValidator::new()
                    .min_length(min.unwrap_or(0))
                    .max_length(max.unwrap_or(usize::MAX))
            ),
            // ...
        }
    }
}
```

### Benefits of This Hybrid Approach

| Aspect | `Rule<T>` Enum | Validator Structs |
|--------|----------------|-------------------|
| **Purpose** | Serializable rule representation | Full-featured implementation |
| **Serialization** | ✅ JSON/YAML friendly | ❌ Contains closures |
| **Customization** | Limited (data only) | ✅ Custom messages, callbacks |
| **Composition** | ✅ Tree structure (`All`, `Any`, `When`) | Manual chaining |
| **Use Case** | Config-driven forms | Programmatic validation |

This hybrid approach provides:

1. **Serialization** - `Rule<T>` can be loaded from JSON/YAML config
2. **Customization** - Validator structs support custom error messages, callbacks
3. **Type safety** - Both approaches are strongly typed
4. **Composition** - `Rule::All`, `Rule::Any`, `Rule::When` for tree structure
5. **Interoperability** - Can use validators directly OR via Rule enum
6. **Mirrors Laminas** - Separate validator/filter crates like the PHP framework

---

## Crate 1: walrs_form_core (Shared Foundation)

```rust
// ============================================================================
// Rule Enum - The heart of validation
// ============================================================================
/// A composable validation rule
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Rule<T> {
    // ---- Presence ----
    Required,
    // ---- String Rules ----
    MinLength(usize),
    MaxLength(usize),
    ExactLength(usize),
    Pattern(#[serde(with = "serde_regex")] Arc<Regex>),
    Email,
    Url,
    // ---- Numeric Rules ----
    Min(T),
    Max(T),
    Range { min: T, max: T },
    Step(T),
    // ---- Comparison ----
    Equals(T),
    OneOf(Vec<T>),
    // ---- Composite Rules (Tree Structure) ----
    All(Vec<Rule<T>>),           // AND - all must pass
    Any(Vec<Rule<T>>),           // OR - at least one
    Not(Box<Rule<T>>),           // Negation
    When {                        // Conditional
        condition: Condition<T>,
        then_rules: Vec<Rule<T>>,
        else_rules: Option<Vec<Rule<T>>>,
    },
    // ---- Custom ----
    #[serde(skip)]
    Custom(Arc<dyn Fn(&T) -> RuleResult + Send + Sync>),
    // ---- Reference to Named Rule ----
    Ref(String),  // For serialization - resolved at runtime
}
/// Conditions for When rules (serializable)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition<T> {
    IsEmpty,
    IsNotEmpty,
    Equals(T),
    GreaterThan(T),
    LessThan(T),
    Matches(String),  // Regex pattern
    #[serde(skip)]
    Custom(Arc<dyn Fn(&T) -> bool + Send + Sync>),
}
```
### Filter Types
```rust
/// A composable value transformer
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Filter<T> {
    // ---- String Filters ----
    Trim,
    Lowercase,
    Uppercase,
    StripTags,
    HtmlEntities,
    Slug { max_length: Option<usize> },
    // ---- Numeric Filters ----
    Clamp { min: T, max: T },
    Abs,
    Round { precision: u32 },
    // ---- Composite ----
    Chain(Vec<Filter<T>>),        // Apply in sequence
    // ---- Custom ----
    #[serde(skip)]
    Custom(Arc<dyn Fn(T) -> T + Send + Sync>),
}
```
### Input (Field-Level Validation)
```rust
/// Validation configuration for a single field
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input<T> {
    pub name: String,
    pub required: bool,
    pub rules: Vec<Rule<T>>,
    pub filters: Vec<Filter<T>>,
    pub break_on_failure: bool,
    #[serde(skip)]
    pub messages: Option<MessageProvider>,
}
impl<T> Input<T> {
    /// Validate a value
    pub fn validate(&self, value: &T) -> Result<(), Violations> { ... }
    /// Apply filters to transform a value
    pub fn filter(&self, value: T) -> T { ... }
    /// Filter then validate
    pub fn process(&self, value: T) -> Result<T, Violations> { ... }
}
```
### InputFilter (Multi-Field Validation)
```rust
/// Validation for multiple fields (e.g., a form)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputFilter {
    pub inputs: HashMap<String, Input<Value>>,
    pub cross_field_rules: Vec<CrossFieldRule>,
    #[serde(skip)]
    pub named_rules: HashMap<String, Rule<Value>>,  // For Rule::Ref resolution
}
/// Cross-field validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrossFieldRule {
    pub name: String,
    pub fields: Vec<String>,
    pub rule: CrossFieldRuleType,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CrossFieldRuleType {
    FieldsEqual { field_a: String, field_b: String },
    RequiredIf { field: String, condition: Condition<Value> },
    RequiredUnless { field: String, condition: Condition<Value> },
    #[serde(skip)]
    Custom(Arc<dyn Fn(&FormData) -> RuleResult + Send + Sync>),
}
```
---
## Crate 2: walrs_validation (Validator Structs)
### Validator Enum
```rust
/// All validation rules as enums
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "validator_type")]
pub enum Validator {
    // ---- Presence ----
    Required,
    // ---- String Rules ----
    MinLength(usize),
    MaxLength(usize),
    ExactLength(usize),
    Pattern(#[serde(with = "serde_regex")] Arc<Regex>),
    Email,
    Url,
    // ---- Numeric Rules ----
    Min(Value),
    Max(Value),
    Range { min: Value, max: Value },
    Step(Value),
    // ---- Comparison ----
    Equals(Value),
    OneOf(Vec<Value>),
    // ---- Composite Rules (Tree Structure) ----
    All(Vec<Validator>),           // AND - all must pass
    Any(Vec<Validator>),           // OR - at least one
    Not(Box<Validator>),           // Negation
    When {                        // Conditional
        condition: Condition<Value>,
        then_rules: Vec<Validator>,
        else_rules: Option<Vec<Validator>>,
    },
    // ---- Custom ----
    #[serde(skip)]
    Custom(Arc<dyn Fn(&Value) -> RuleResult + Send + Sync>),
    // ---- Reference to Named Rule ----
    Ref(String),  // For serialization - resolved at runtime
}
```
### Validator Struct
```rust
/// Validation configuration for a set of fields
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorConfig {
    pub name: String,
    pub required: bool,
    pub rules: Vec<Validator>,
    pub filters: Vec<Filter<Value>>,
    pub break_on_failure: bool,
    #[serde(skip)]
    pub messages: Option<MessageProvider>,
}
impl ValidatorConfig {
    /// Validate a value
    pub fn validate(&self, value: &Value) -> Result<(), Violations> { ... }
    /// Apply filters to transform a value
    pub fn filter(&self, value: Value) -> Value { ... }
    /// Filter then validate
    pub fn process(&self, value: Value) -> Result<Value, Violations> { ... }
}
```
---
## Crate 3: walrs_filter (Filter Structs)
### Filter Enum
```rust
/// All filter types as enums
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "filter_type")]
pub enum Filter {
    // ---- String Filters ----
    Trim,
    Lowercase,
    Uppercase,
    StripTags,
    HtmlEntities,
    Slug { max_length: Option<usize> },
    // ---- Numeric Filters ----
    Clamp { min: Value, max: Value },
    Abs,
    Round { precision: u32 },
    // ---- Composite ----
    Chain(Vec<Filter>),        // Apply in sequence
    // ---- Custom ----
    #[serde(skip)]
    Custom(Arc<dyn Fn(Value) -> Value + Send + Sync>),
}
```
### Filter Struct
```rust
/// Filter configuration for a set of fields
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterConfig {
    pub name: String,
    pub filters: Vec<Filter>,
}
impl FilterConfig {
    /// Apply filters to a value
    pub fn apply(&self, value: Value) -> Value {
        self.filters.iter().fold(value, |v, f| f.apply(v))
    }
}
```
---
## Crate 4: walrs_inputfilter (Composition Layer)
*Implements the Fresh Approach from [FRESH_APPROACH.md](./FRESH_APPROACH.md)*
### Core Types
```rust
// ============================================================================
// Rule Enum - The heart of validation
// ============================================================================
/// A composable validation rule
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Rule<T> {
    // ---- Presence ----
    Required,
    // ---- String Rules ----
    MinLength(usize),
    MaxLength(usize),
    ExactLength(usize),
    Pattern(#[serde(with = "serde_regex")] Arc<Regex>),
    Email,
    Url,
    // ---- Numeric Rules ----
    Min(T),
    Max(T),
    Range { min: T, max: T },
    Step(T),
    // ---- Comparison ----
    Equals(T),
    OneOf(Vec<T>),
    // ---- Composite Rules (Tree Structure) ----
    All(Vec<Rule<T>>),           // AND - all must pass
    Any(Vec<Rule<T>>),           // OR - at least one
    Not(Box<Rule<T>>),           // Negation
    When {                        // Conditional
        condition: Condition<T>,
        then_rules: Vec<Rule<T>>,
        else_rules: Option<Vec<Rule<T>>>,
    },
    // ---- Custom ----
    #[serde(skip)]
    Custom(Arc<dyn Fn(&T) -> RuleResult + Send + Sync>),
    // ---- Reference to Named Rule ----
    Ref(String),  // For serialization - resolved at runtime
}
/// Conditions for When rules (serializable)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition<T> {
    IsEmpty,
    IsNotEmpty,
    Equals(T),
    GreaterThan(T),
    LessThan(T),
    Matches(String),  // Regex pattern
    #[serde(skip)]
    Custom(Arc<dyn Fn(&T) -> bool + Send + Sync>),
}
```
### Filter Types
```rust
/// A composable value transformer
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Filter<T> {
    // ---- String Filters ----
    Trim,
    Lowercase,
    Uppercase,
    StripTags,
    HtmlEntities,
    Slug { max_length: Option<usize> },
    // ---- Numeric Filters ----
    Clamp { min: T, max: T },
    Abs,
    Round { precision: u32 },
    // ---- Composite ----
    Chain(Vec<Filter<T>>),        // Apply in sequence
    // ---- Custom ----
    #[serde(skip)]
    Custom(Arc<dyn Fn(T) -> T + Send + Sync>),
}
```
### Input (Field-Level Validation)
```rust
/// Validation configuration for a single field
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input<T> {
    pub name: String,
    pub required: bool,
    pub rules: Vec<Rule<T>>,
    pub filters: Vec<Filter<T>>,
    pub break_on_failure: bool,
    #[serde(skip)]
    pub messages: Option<MessageProvider>,
}
impl<T> Input<T> {
    /// Validate a value
    pub fn validate(&self, value: &T) -> Result<(), Violations> { ... }
    /// Apply filters to transform a value
    pub fn filter(&self, value: T) -> T { ... }
    /// Filter then validate
    pub fn process(&self, value: T) -> Result<T, Violations> { ... }
}
```
### InputFilter (Multi-Field Validation)
```rust
/// Validation for multiple fields (e.g., a form)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputFilter {
    pub inputs: HashMap<String, Input<Value>>,
    pub cross_field_rules: Vec<CrossFieldRule>,
    #[serde(skip)]
    pub named_rules: HashMap<String, Rule<Value>>,  // For Rule::Ref resolution
}
/// Cross-field validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrossFieldRule {
    pub name: String,
    pub fields: Vec<String>,
    pub rule: CrossFieldRuleType,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CrossFieldRuleType {
    FieldsEqual { field_a: String, field_b: String },
    RequiredIf { field: String, condition: Condition<Value> },
    RequiredUnless { field: String, condition: Condition<Value> },
    #[serde(skip)]
    Custom(Arc<dyn Fn(&FormData) -> RuleResult + Send + Sync>),
}
```
---
## Crate 5: walrs_form (Form Elements & Structure)
### Element Enum (Rust Idiomatic Approach)
```rust
/// All HTML form element types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "element_type")]
pub enum Element {
    // ---- Text Inputs ----
    Text(TextElement),
    Password(PasswordElement),
    Email(EmailElement),
    Url(UrlElement),
    Tel(TelElement),
    Search(SearchElement),
    // ---- Numeric Inputs ----
    Number(NumberElement),
    Range(RangeElement),
    // ---- Date/Time Inputs ----
    Date(DateElement),
    Time(TimeElement),
    DateTime(DateTimeElement),
    Month(MonthElement),
    Week(WeekElement),
    // ---- Choice Inputs ----
    Checkbox(CheckboxElement),
    Radio(RadioElement),
    Select(SelectElement),
    MultiSelect(MultiSelectElement),
    // ---- File Inputs ----
    File(FileElement),
    // ---- Text Areas ----
    Textarea(TextareaElement),
    // ---- Hidden/Special ----
    Hidden(HiddenElement),
    Color(ColorElement),
    // ---- Buttons ----
    Submit(SubmitElement),
    Reset(ResetElement),
    Button(ButtonElement),
    // ---- Grouping ----
    Fieldset(Fieldset),
    // ---- Custom ----
    Custom {
        element_type: String,
        attributes: Attributes,
        input: Option<Input<Value>>,
    },
}
```
### Common Element Traits
```rust
/// Common behavior for all elements
pub trait FormElement: Send + Sync {
    fn name(&self) -> &str;
    fn attributes(&self) -> &Attributes;
    fn attributes_mut(&mut self) -> &mut Attributes;
    fn input(&self) -> Option<&Input<Value>>;
    fn input_mut(&mut self) -> Option<&mut Input<Value>>;
    /// Get value as generic Value type
    fn value(&self) -> Option<&Value>;
    fn set_value(&mut self, value: Value);
    /// Validate the element's current value
    fn validate(&self) -> Result<(), Violations> {
        match (self.input(), self.value()) {
            (Some(input), Some(value)) => input.validate(value),
            (Some(input), None) if input.required => {
                Err(Violations::single(Violation::required(self.name())))
            }
            _ => Ok(()),
        }
    }
}
/// Elements that can provide their own input filter
pub trait InputFilterProvider {
    fn input_filter(&self) -> Input<Value>;
}
```
### Concrete Element Structs
```rust
/// Text input element
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextElement {
    pub name: String,
    pub attributes: Attributes,
    pub value: Option<String>,
    pub input: Option<Input<String>>,
    // Text-specific
    pub placeholder: Option<String>,
    pub maxlength: Option<usize>,
    pub minlength: Option<usize>,
    pub pattern: Option<String>,
}
/// Select element with options
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectElement {
    pub name: String,
    pub attributes: Attributes,
    pub value: Option<String>,
    pub input: Option<Input<String>>,
    // Select-specific
    pub options: Vec<SelectOption>,
    pub empty_option: Option<SelectOption>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
    pub selected: bool,
    pub group: Option<String>,
}
/// Checkbox element
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckboxElement {
    pub name: String,
    pub attributes: Attributes,
    pub checked: bool,
    pub input: Option<Input<bool>>,
    // Checkbox-specific
    pub checked_value: String,
    pub unchecked_value: Option<String>,
}
```
### Fieldset (Element Grouping)
```rust
/// A group of related form elements
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fieldset {
    pub name: String,
    pub legend: Option<String>,
    pub attributes: Attributes,
    pub elements: Vec<Element>,
    pub input_filter: Option<InputFilter>,
}
impl Fieldset {
    /// Iterate over all elements (including nested fieldsets)
    pub fn iter_elements(&self) -> impl Iterator<Item = &Element> {
        self.elements.iter().flat_map(|e| match e {
            Element::Fieldset(fs) => Box::new(fs.iter_elements()) as Box<dyn Iterator<Item = &Element>>,
            other => Box::new(std::iter::once(other)),
        })
    }
    /// Get element by name (dot notation for nested: "address.street")
    pub fn get(&self, name: &str) -> Option<&Element> { ... }
    /// Validate all elements
    pub fn validate(&self) -> Result<(), FormViolations> { ... }
}
```
### Form (Top-Level Container)
```rust
/// A complete HTML form
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Form {
    pub name: String,
    pub action: Option<String>,
    pub method: FormMethod,
    pub enctype: FormEnctype,
    pub attributes: Attributes,
    pub elements: Vec<Element>,
    pub input_filter: Option<InputFilter>,
    #[serde(skip)]
    pub data: Option<FormData>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormMethod {
    Get,
    Post,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormEnctype {
    UrlEncoded,
    Multipart,
    Plain,
}
impl Form {
    /// Bind data to form elements
    pub fn bind(&mut self, data: FormData) -> &mut Self { ... }
    /// Extract data from form elements
    pub fn extract(&self) -> FormData { ... }
    /// Validate all elements and cross-field rules
    pub fn validate(&self) -> Result<(), FormViolations> { ... }
    /// Check if form is valid
    pub fn is_valid(&self) -> bool { ... }
    /// Get all validation errors
    pub fn errors(&self) -> &FormViolations { ... }
}
```
### Attributes (HTML Attributes)
```rust
/// HTML element attributes
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Attributes {
    pub id: Option<String>,
    pub class: Vec<String>,
    pub style: Option<String>,
    pub disabled: bool,
    pub readonly: bool,
    pub required: bool,
    pub autofocus: bool,
    pub tabindex: Option<i32>,
    pub title: Option<String>,
    pub data: HashMap<String, String>,  // data-* attributes
    pub aria: HashMap<String, String>,  // aria-* attributes
    pub custom: HashMap<String, String>, // any other attributes
}
impl Attributes {
    /// Render as HTML attribute string
    pub fn to_html(&self) -> String { ... }
    /// Fluent builder
    pub fn class(mut self, class: impl Into<String>) -> Self { ... }
    pub fn id(mut self, id: impl Into<String>) -> Self { ... }
    pub fn data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self { ... }
}
```
---
## Crate 6: walrs_form_view (Rendering)
### Render Trait
```rust
/// Trait for rendering to HTML
pub trait Render {
    fn render(&self, config: &RenderConfig) -> Html;
    fn render_with_errors(&self, errors: &Violations, config: &RenderConfig) -> Html {
        // Default implementation wraps element with error display
        ...
    }
}
/// Rendered HTML output
#[derive(Clone, Debug)]
pub struct Html(String);
impl Html {
    pub fn as_str(&self) -> &str { &self.0 }
    pub fn into_string(self) -> String { self.0 }
}
impl std::fmt::Display for Html {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```
### Render Configuration
```rust
/// Configuration for HTML rendering
#[derive(Clone, Debug)]
pub struct RenderConfig {
    pub theme: Theme,
    pub label_position: LabelPosition,
    pub error_position: ErrorPosition,
    pub wrapper: Option<WrapperConfig>,
    pub escape_html: bool,
}
#[derive(Clone, Debug)]
pub enum Theme {
    None,
    Bootstrap5,
    Tailwind,
    Custom(Box<dyn ThemeProvider>),
}
#[derive(Clone, Debug)]
pub enum LabelPosition {
    Before,
    After,
    Wrap,
    None,
}
pub trait ThemeProvider: Send + Sync {
    fn input_class(&self, element: &Element, has_error: bool) -> String;
    fn label_class(&self, element: &Element) -> String;
    fn error_class(&self) -> String;
    fn wrapper_class(&self, element: &Element) -> String;
}
```
### Element Renderers
```rust
impl Render for Element {
    fn render(&self, config: &RenderConfig) -> Html {
        match self {
            Element::Text(e) => e.render(config),
            Element::Select(e) => e.render(config),
            Element::Checkbox(e) => e.render(config),
            Element::Fieldset(e) => e.render(config),
            // ... all variants
        }
    }
}
impl Render for TextElement {
    fn render(&self, config: &RenderConfig) -> Html {
        let mut attrs = self.attributes.clone();
        attrs.custom.insert("type".to_string(), "text".to_string());
        attrs.custom.insert("name".to_string(), self.name.clone());
        if let Some(ref value) = self.value {
            attrs.custom.insert("value".to_string(), 
                if config.escape_html { html_escape(value) } else { value.clone() }
            );
        }
        Html(format!("<input {}>", attrs.to_html()))
    }
}
impl Render for Form {
    fn render(&self, config: &RenderConfig) -> Html {
        let mut html = String::new();
        html.push_str(&format!(
            "<form name=\"{}\" method=\"{}\" action=\"{}\" {}>",
            self.name,
            self.method.as_str(),
            self.action.as_deref().unwrap_or(""),
            self.attributes.to_html()
        ));
        for element in &self.elements {
            html.push_str(&element.render(config).0);
        }
        html.push_str("</form>");
        Html(html)
    }
}
```
### View Helpers (Functional Approach)
```rust
/// Functional view helpers
pub mod helpers {
    /// Render a form element with label
    pub fn form_row(element: &Element, label: &str, config: &RenderConfig) -> Html { ... }
    /// Render just the label
    pub fn form_label(element: &Element, label: &str, config: &RenderConfig) -> Html { ... }
    /// Render just the input
    pub fn form_element(element: &Element, config: &RenderConfig) -> Html { ... }
    /// Render validation errors
    pub fn form_errors(violations: &Violations, config: &RenderConfig) -> Html { ... }
    /// Render entire form
    pub fn form(form: &Form, config: &RenderConfig) -> Html { ... }
}
```
---
## Crate 7: walrs_form_serde (Serialization)
### JSON Schema Generation
```rust
/// Generate JSON Schema for form definition
pub trait ToJsonSchema {
    fn to_json_schema(&self) -> serde_json::Value;
}
impl ToJsonSchema for Form {
    fn to_json_schema(&self) -> serde_json::Value {
        json!({
            "\$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": self.elements.iter()
                .filter_map(|e| e.to_json_schema_property())
                .collect::<serde_json::Map<String, Value>>(),
            "required": self.required_fields(),
        })
    }
}
impl ToJsonSchema for Rule<String> {
    fn to_json_schema(&self) -> serde_json::Value {
        match self {
            Rule::MinLength(n) => json!({ "minLength": n }),
            Rule::MaxLength(n) => json!({ "maxLength": n }),
            Rule::Pattern(re) => json!({ "pattern": re.as_str() }),
            Rule::Email => json!({ "format": "email" }),
            Rule::All(rules) => {
                json!({ "allOf": rules.iter().map(|r| r.to_json_schema()).collect::<Vec<_>>() })
            }
            // ... etc
        }
    }
}
```
### Form Definition Format
```yaml
# Example: user_registration_form.yaml
name: user_registration
method: post
action: /register
elements:
  - element_type: Text
    name: username
    attributes:
      placeholder: "Choose a username"
    input:
      required: true
      rules:
        - type: MinLength
          config: 3
        - type: MaxLength
          config: 20
        - type: Pattern
          config: "^[a-zA-Z0-9_]+\$"
      filters:
        - type: Trim
        - type: Lowercase
  - element_type: Email
    name: email
    input:
      required: true
      rules:
        - type: Email
  - element_type: Password
    name: password
    input:
      required: true
      rules:
        - type: MinLength
          config: 8
        - type: All
          config:
            - type: Pattern
              config: "[A-Z]"  # uppercase
            - type: Pattern
              config: "[0-9]"  # digit
  - element_type: Password
    name: password_confirm
    input:
      required: true
input_filter:
  cross_field_rules:
    - name: passwords_match
      fields: [password, password_confirm]
      rule:
        type: FieldsEqual
        field_a: password
        field_b: password_confirm
```
### Loading Forms
```rust
/// Load form from various sources
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
---
## Cross-Cutting Concerns
### Value Type (Dynamic Typing for Forms)
```rust
/// Dynamic value type for form data
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
impl Value {
    pub fn as_str(&self) -> Option<&str> { ... }
    pub fn as_bool(&self) -> Option<bool> { ... }
    pub fn as_i64(&self) -> Option<i64> { ... }
    pub fn is_empty(&self) -> bool { ... }
}
// Conversion traits
impl From<String> for Value { ... }
impl From<&str> for Value { ... }
impl From<i32> for Value { ... }
impl From<bool> for Value { ... }
// ... etc
```
### Form Data
```rust
/// Data bound to/from a form
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FormData(HashMap<String, Value>);
impl FormData {
    pub fn new() -> Self { Self(HashMap::new()) }
    pub fn get(&self, key: &str) -> Option<&Value> {
        // Support dot notation: "address.street"
        let parts: Vec<&str> = key.split('.').collect();
        // ... navigate nested structure
    }
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.0.insert(key.into(), value.into());
    }
    pub fn merge(&mut self, other: FormData) {
        self.0.extend(other.0);
    }
}
// Convert from web framework extractors
impl From<HashMap<String, String>> for FormData { ... }
impl From<serde_json::Value> for FormData { ... }
```
### Violations (Error Handling)
```rust
/// A single validation violation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Violation {
    pub code: ViolationCode,
    pub message: String,
    pub path: Option<String>,      // Field path (e.g., "address.street")
    pub params: HashMap<String, Value>, // For message interpolation
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ViolationCode {
    Required,
    MinLength,
    MaxLength,
    Pattern,
    Email,
    Range,
    Custom(String),
}
/// Collection of violations
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Violations(Vec<Violation>);
/// Form-level violations (by field)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FormViolations {
    pub fields: HashMap<String, Violations>,
    pub form: Violations,  // Cross-field errors
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
---
## Data Flow Architecture
### Backend Flow
```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   HTTP      │     │   Form      │     │  InputFil-  │     │   Business  │
│   Request   │────▶│   Binding   │────▶│    ter      │────▶│   Logic     │
│             │     │             │     │             │     │             │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
      │                   │                   │                   │
      │              FormData            Validated &          Domain
      │                                  Filtered Data        Objects
      │
      ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Form      │◀────│  Render     │◀────│  Violations │
│   HTML      │     │  Engine     │     │             │
└─────────────┘     └─────────────┘     └─────────────┘
```
### Frontend/Backend Shared Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                         BACKEND                                  │
│  ┌──────────────┐                                               │
│  │ Form Config  │──────┬──────────────────────────────────────┐ │
│  │ (YAML/JSON)  │      │                                      │ │
│  └──────────────┘      ▼                                      │ │
│                   ┌──────────┐     ┌──────────┐              │ │
│                   │  Form    │────▶│ Validate │              │ │
│                   │  Struct  │     │ on Submit│              │ │
│                   └──────────┘     └──────────┘              │ │
│                        │                                      │ │
│                        │ Serialize                            │ │
│                        ▼                                      │ │
│                   ┌──────────┐                               │ │
│                   │   JSON   │─────────────────────────────┐ │ │
│                   │  Schema  │                             │ │ │
│                   └──────────┘                             │ │ │
└────────────────────────│───────────────────────────────────┘ │
                         │                                     │
                         ▼                                     │
┌────────────────────────│─────────────────────────────────────┘
│                   FRONTEND                                    │
│                        │                                      │
│                        ▼                                      │
│                   ┌──────────┐     ┌──────────┐              │
│                   │   Form   │────▶│ Validate │              │
│                   │  Render  │     │ on Input │              │
│                   └──────────┘     └──────────┘              │
│                        │                                      │
│                        ▼                                      │
│                   ┌──────────┐                               │
│                   │   DOM    │                               │
│                   └──────────┘                               │
└───────────────────────────────────────────────────────────────┘
```
---
## Rust Idioms & Functional Patterns
### 1. Enum-Based Type Design
```rust
// Instead of class hierarchies, use enums
pub enum Element { ... }  // All element types
pub enum Rule<T> { ... }  // All validation rules
pub enum Filter<T> { ... } // All filters
// Pattern matching for behavior
impl Rule<T> {
    pub fn validate(&self, value: &T) -> RuleResult {
        match self {
            Rule::Required => { ... }
            Rule::MinLength(n) => { ... }
            Rule::All(rules) => rules.iter().try_for_each(|r| r.validate(value)),
            // exhaustive matching ensures all cases handled
        }
    }
}
```
### 2. Functional Combinators
```rust
// Rule combinators (from Fresh Approach)
impl<T> Rule<T> {
    pub fn and(self, other: Rule<T>) -> Rule<T> {
        match self {
            Rule::All(mut rules) => { rules.push(other); Rule::All(rules) }
            _ => Rule::All(vec![self, other]),
        }
    }
    pub fn or(self, other: Rule<T>) -> Rule<T> {
        match self {
            Rule::Any(mut rules) => { rules.push(other); Rule::Any(rules) }
            _ => Rule::Any(vec![self, other]),
        }
    }
    pub fn not(self) -> Rule<T> {
        Rule::Not(Box::new(self))
    }
    pub fn when(self, condition: Condition<T>) -> Rule<T> {
        Rule::When {
            condition,
            then_rules: vec![self],
            else_rules: None,
        }
    }
}
// Filter combinators
impl<T> Filter<T> {
    pub fn then(self, next: Filter<T>) -> Filter<T> {
        match self {
            Filter::Chain(mut filters) => { filters.push(next); Filter::Chain(filters) }
            _ => Filter::Chain(vec![self, next]),
        }
    }
}
```
### 3. Builder Pattern with Fluent API
```rust
impl Form {
    pub fn new(name: impl Into<String>) -> FormBuilder {
        FormBuilder::new(name)
    }
}
pub struct FormBuilder { ... }
impl FormBuilder {
    pub fn action(mut self, action: impl Into<String>) -> Self { ... }
    pub fn method(mut self, method: FormMethod) -> Self { ... }
    pub fn text(mut self, name: impl Into<String>) -> ElementBuilder<Self, TextElement> { ... }
    pub fn email(mut self, name: impl Into<String>) -> ElementBuilder<Self, EmailElement> { ... }
    pub fn select(mut self, name: impl Into<String>) -> ElementBuilder<Self, SelectElement> { ... }
    pub fn fieldset(mut self, name: impl Into<String>) -> FieldsetBuilder<Self> { ... }
    pub fn build(self) -> Form { ... }
}
// Usage
let form = Form::new("registration")
    .action("/register")
    .method(FormMethod::Post)
    .text("username")
        .required()
        .min_length(3)
        .max_length(20)
        .placeholder("Choose a username")
        .done()
    .email("email")
        .required()
        .done()
    .select("country")
        .options(countries)
        .empty_option("Select a country...")
        .done()
    .fieldset("address")
        .legend("Address")
        .text("street").required().done()
        .text("city").required().done()
        .done()
    .build();
```
### 4. Iterator & Combinator Chains
```rust
impl Form {
    /// Validate and collect all errors functionally
    pub fn validate(&self) -> Result<(), FormViolations> {
        let field_errors: HashMap<String, Violations> = self
            .iter_elements()
            .filter_map(|e| {
                e.validate().err().map(|v| (e.name().to_string(), v))
            })
            .collect();
        let form_errors: Violations = self
            .input_filter
            .as_ref()
            .map(|f| f.validate_cross_field(&self.extract()))
            .transpose()?
            .unwrap_or_default();
        if field_errors.is_empty() && form_errors.is_empty() {
            Ok(())
        } else {
            Err(FormViolations { fields: field_errors, form: form_errors })
        }
    }
}
```
### 5. Type-State Pattern for Form Lifecycle
```rust
/// Form states
pub struct Unbound;
pub struct Bound;
pub struct Validated;
pub struct Form<State = Unbound> {
    inner: FormInner,
    _state: PhantomData<State>,
}
impl Form<Unbound> {
    pub fn bind(self, data: FormData) -> Form<Bound> { ... }
}
impl Form<Bound> {
    pub fn validate(self) -> Result<Form<Validated>, (Form<Bound>, FormViolations)> { ... }
}
impl Form<Validated> {
    pub fn data(&self) -> &FormData { ... }  // Only available after validation
}
// Usage - compile-time enforcement of lifecycle
let form = Form::new("login");
let form = form.bind(request_data);  // Now Form<Bound>
let form = form.validate()?;          // Now Form<Validated>
let data = form.data();               // Safe to access
```
---
## Frontend Integration Strategy
### Strategy 1: JSON Schema for Native Validation
```rust
// Backend generates JSON Schema
let schema = form.to_json_schema();
// Frontend uses standard JSON Schema validators
// (ajv, joi, yup can consume JSON Schema)
```
### Strategy 2: WASM Compilation
```rust
// Compile walrs_inputfilter to WASM
#[wasm_bindgen]
pub fn validate_form(form_json: &str, data_json: &str) -> JsValue {
    let form: Form = serde_json::from_str(form_json).unwrap();
    let data: FormData = serde_json::from_str(data_json).unwrap();
    match form.validate_data(&data) {
        Ok(()) => JsValue::NULL,
        Err(violations) => serde_wasm_bindgen::to_value(&violations).unwrap(),
    }
}
```
### Strategy 3: Generate TypeScript Types
```rust
// Generate TypeScript interfaces from Form
pub trait ToTypeScript {
    fn to_typescript(&self) -> String;
}
impl ToTypeScript for Form {
    fn to_typescript(&self) -> String {
        let fields: Vec<String> = self.iter_elements()
            .map(|e| format!("  {}: {};", e.name(), e.typescript_type()))
            .collect();
        format!("interface {}Data {{\n{}\n}}", 
            to_pascal_case(&self.name), 
            fields.join("\n")
        )
    }
}
// Output:
// interface UserRegistrationData {
//   username: string;
//   email: string;
//   password: string;
//   password_confirm: string;
// }
```
---
## Implementation Roadmap
### Phase 1: Core Foundation (walrs_form_core + walrs_inputfilter v2)
- [ ] \`Value\` enum for dynamic typing
- [ ] \`Rule<T>\` enum with all variants
- [ ] \`Filter<T>\` enum with all variants
- [ ] \`Input<T>\` struct with validate/filter
- [ ] \`InputFilter\` for multi-field validation
- [ ] \`Violation\` and \`Violations\` types
- [ ] Serde support for all types
- [ ] Unit tests for all rules/filters
### Phase 2: Form Structure (walrs_form)
- [ ] \`Element\` enum with all HTML input types
- [ ] \`Attributes\` struct
- [ ] \`Fieldset\` struct
- [ ] \`Form\` struct
- [ ] Builder pattern for fluent construction
- [ ] Data binding (FormData)
- [ ] Element-level validation integration
### Phase 3: Rendering (walrs_form_view)
- [ ] \`Render\` trait
- [ ] \`Html\` output type
- [ ] Renderers for all elements
- [ ] Theme support (Bootstrap, Tailwind)
- [ ] Error rendering
- [ ] View helper functions
### Phase 4: Serialization (walrs_form_serde)
- [ ] YAML/JSON form definitions
- [ ] Form loading from files
- [ ] JSON Schema generation
- [ ] TypeScript type generation
### Phase 5: Integration & DX
- [ ] \`walrs_form_derive\` proc macro
- [ ] Actix-web integration example
- [ ] Axum integration example
- [ ] WASM compilation for frontend
- [ ] Documentation & examples
---
## Summary
This ecosystem provides:
1. **Separation of Concerns** - Each crate has a focused responsibility
2. **Rust Idioms** - Enums, traits, pattern matching, iterators
3. **Functional Patterns** - Combinators, immutable transformations, fluent builders
4. **Type Safety** - Compile-time guarantees where possible
5. **Serialization First** - Forms as data, shareable between frontend/backend
6. **Thread Safety** - Arc-based sharing for web servers
7. **Flexibility** - Dynamic rules, runtime configuration, custom extensions
The design allows for:
- Backend-only usage (Rust web servers)
- Full-stack usage (shared validation via JSON Schema or WASM)
- Configuration-driven forms (loaded from YAML/JSON)
- Programmatic form construction (builders)
- Integration with any web framework

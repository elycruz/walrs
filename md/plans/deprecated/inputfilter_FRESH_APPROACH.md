# Fresh Approach: Validation Rule Tree Architecture

**Date:** February 14, 2026  
**Status:** Proposal  
**Goal:** Provide structures that can be used to construct a tree of validation rules that can be applied to a value but are also thread safe.

---

## Table of Contents

1. [Core Problem with Current Design](#core-problem-with-current-design)
2. [Fresh Design Principles](#fresh-design-principles)
3. [Proposed Architecture](#proposed-architecture)
4. [Key Differences from Current Design](#key-differences-from-current-design)
5. [Comparison with Existing Rust Validation Crates](#comparison-with-existing-rust-validation-crates)
   - [Overview of Existing Crates](#overview-of-existing-crates)
   - [`validator` Crate Approach](#1-validator-crate-approach)
   - [`garde` Crate Approach](#2-garde-crate-approach)
   - [Feature Comparison Matrix](#3-feature-comparison-matrix)
   - [Key Differentiators](#4-key-differentiators-of-fresh-approach)
   - [When to Use Each](#5-when-to-use-each)
   - [Potential Hybrid Approach](#6-potential-hybrid-approach)
   - [Summary](#7-summary)
6. [Example Usage](#example-usage-proposed)
7. [Implementation Phases](#implementation-phases)
8. [Benefits](#benefits)
9. [Open Questions](#open-questions)
10. [Migration Path](#migration-path)
11. [Inspiration](#inspiration)
12. [References](#references)

---

## Core Problem with Current Design

The current design has accumulated complexity:

1. **Two separate structs** (`Input` / `RefInput`) for owned vs referenced types
2. **Closure references with lifetimes** (`&'a dyn Fn(...)`) make composition difficult
3. **Validators and filters mixed** in one struct
4. **Complex generic bounds** make the API hard to use
5. **Thread safety via `Send + Sync`** on closure refs is awkward

---

## Fresh Design Principles

1. **Validators as data, not closures** - Store validation rules as data structures, not function pointers
2. **Single unified `Rule` type** - No more `Input` vs `RefInput` split
3. **Tree structure via composition** - Rules can contain other rules
4. **Arc-based sharing** - Thread safety through `Arc<Rule>` not lifetime gymnastics
5. **Enum-based rule types** - Pattern matching instead of trait objects

---

## Proposed Architecture

### Core Types

```rust
/// A validation rule that can be composed into trees
#[derive(Clone, Debug)]
pub enum Rule<T> {
    // Leaf validators
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(Arc<Regex>),
    Range { min: Option<T>, max: Option<T> },
    Equals(T),
    Custom(Arc<dyn Fn(&T) -> RuleResult + Send + Sync>),
    
    // Composite rules
    All(Vec<Rule<T>>),           // AND - all must pass
    Any(Vec<Rule<T>>),           // OR - at least one must pass
    Not(Box<Rule<T>>, String),   // Negation with message
    When {                        // Conditional
        condition: Arc<dyn Fn(&T) -> bool + Send + Sync>,
        then_rule: Box<Rule<T>>,
    },
}

/// Result of applying a rule
pub type RuleResult = Result<(), Violation>;

/// A sharable, thread-safe rule
pub type SharedRule<T> = Arc<Rule<T>>;
```

### RuleSet for Field-Level Validation

```rust
/// A named set of rules for a single field
#[derive(Clone)]
pub struct FieldRule<T> {
    pub name: String,
    pub rules: Vec<Rule<T>>,
    pub required: bool,
    pub filters: Vec<Arc<dyn Fn(T) -> T + Send + Sync>>,
}

impl<T> FieldRule<T> {
    pub fn validate(&self, value: &T) -> Result<(), FieldViolations> { ... }
    pub fn filter(&self, value: T) -> T { ... }
    pub fn process(&self, value: T) -> Result<T, FieldViolations> { ... }
}
```

### FormRules for Multi-Field Validation

```rust
/// Rules for an entire form/struct
pub struct FormRules {
    fields: HashMap<String, Box<dyn AnyFieldRule>>,
    cross_field_rules: Vec<CrossFieldRule>,
}

impl FormRules {
    pub fn validate(&self, data: &impl FormData) -> Result<(), FormViolations> { ... }
}
```

---

## Key Differences from Current Design

| Aspect | Current | Proposed |
|--------|---------|----------|
| **Validator storage** | `&'a dyn Fn(T) -> Result` | `enum Rule<T>` variants |
| **Thread safety** | Lifetimes + `Send + Sync` bounds | `Arc<Rule<T>>` |
| **Composition** | `Vec<&'a ValidatorFn>` | `Rule::All(Vec<Rule<T>>)` |
| **Owned vs Ref** | `Input` / `RefInput` split | Single `Rule<T>` with generic `T` |
| **Custom validators** | All validators are closures | Most are data, `Custom()` for closures |
| **Serialization** | Not supported | `Rule` variants are serializable |
| **Builder pattern** | `derive_builder` | Fluent builder methods |

---

## Comparison with Existing Rust Validation Crates

### Overview of Existing Crates

| Crate | Approach | Primary Use Case |
|-------|----------|------------------|
| **validator** | Derive macro on structs | Struct field validation |
| **garde** | Derive macro with custom rules | Struct validation with better ergonomics |
| **validify** | Derive macro + modifiers | Validation + transformation |
| **nutype** | Newtype pattern | Single-value validation |

### 1. `validator` Crate Approach

```rust
use validator::{Validate, ValidationError};

#[derive(Validate)]
struct User {
    #[validate(length(min = 3, max = 20))]
    #[validate(regex = "USERNAME_REGEX")]
    username: String,
    
    #[validate(email)]
    email: String,
    
    #[validate(range(min = 18, max = 150))]
    age: u8,
    
    #[validate(custom = "validate_password")]
    password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    // custom logic
}

// Usage
let user = User { ... };
user.validate()?;
```

**Characteristics:**
- Derive macro based
- Rules defined via attributes
- Works on structs only
- Custom validators are functions
- Not easily composable at runtime

### 2. `garde` Crate Approach

```rust
use garde::Validate;

#[derive(Validate)]
struct User {
    #[garde(length(min = 3, max = 20), pattern(r"^[a-zA-Z0-9_]+$"))]
    username: String,
    
    #[garde(email)]
    email: String,
    
    #[garde(custom(validate_password))]
    password: String,
}

// Usage with context
let ctx = MyContext { ... };
user.validate(&ctx)?;
```

**Characteristics:**
- Similar to `validator` but with context support
- Better error messages
- Still derive-macro focused
- Rules are compile-time

### 3. Feature Comparison Matrix

| Aspect | `validator`/`garde` | Fresh Approach |
|--------|---------------------|----------------|
| **Rule Definition** | Compile-time attributes | Runtime `Rule<T>` enum |
| **Target** | Structs only | Any value type |
| **Composition** | Limited (nested structs) | Full tree composition |
| **Runtime Rules** | Not supported | First-class support |
| **Serialization** | No | Yes (rules are data) |
| **Thread Safety** | Implicit (structs are `Send`) | Explicit `Arc<Rule>` |
| **Custom Validators** | External functions | `Rule::Custom(Arc<Fn>)` |
| **Use Case** | Validate known structs | Dynamic rule trees |

### 4. Key Differentiators of Fresh Approach

#### Rules as Data vs Attributes

```rust
// validator/garde: Compile-time attributes
#[validate(length(min = 3))]
username: String,

// Fresh: Runtime data structure
let rule = Rule::MinLength(3);
let rule = Rule::All(vec![Rule::MinLength(3), Rule::MaxLength(20)]);
```

**Why it matters:** Rules can be loaded from config, modified at runtime, serialized/deserialized.

#### Value-Centric vs Struct-Centric

```rust
// validator: Must wrap in struct
#[derive(Validate)]
struct UsernameWrapper {
    #[validate(length(min = 3))]
    value: String,
}

// Fresh: Validate any value directly
let rule = Rule::<String>::MinLength(3);
rule.validate(&"ab".to_string())?;
```

**Why it matters:** Can validate individual values, function parameters, config values without wrapper structs.

#### Tree Composition

```rust
// validator: Flat list of rules per field
#[validate(length(min = 3), regex = "...")]

// Fresh: Nested tree structure
Rule::All(vec![
    Rule::MinLength(3),
    Rule::Any(vec![
        Rule::Pattern(email_regex),
        Rule::Pattern(phone_regex),
    ]),
    Rule::When {
        condition: Arc::new(|v| v.starts_with("+")),
        then_rule: Box::new(Rule::Pattern(intl_phone_regex)),
    },
])
```

**Why it matters:** Complex conditional logic, OR conditions, negation - all composable.

#### Thread-Safe Sharing

```rust
// validator: Validate owned struct, no sharing of rules
let user = User { ... };
user.validate()?;

// Fresh: Share rule trees across threads
let rules: Arc<Rule<String>> = Arc::new(Rule::MinLength(3));
let rules_clone = rules.clone();
tokio::spawn(async move {
    rules_clone.validate(&input)?;
});
```

**Why it matters:** Web servers can share validation rules across request handlers without cloning.

### 5. When to Use Each

| Use Case | Best Choice |
|----------|-------------|
| Validate known struct shapes | `validator` / `garde` |
| Compile-time rule checking | `validator` / `garde` |
| Dynamic rules from config | **Fresh Approach** |
| Validate individual values | **Fresh Approach** |
| Complex conditional logic | **Fresh Approach** |
| Share rules across threads | **Fresh Approach** |
| Serialize validation rules | **Fresh Approach** |
| Form builder with dynamic fields | **Fresh Approach** |

### 6. Potential Hybrid Approach

The approaches aren't mutually exclusive. We could support both:

```rust
// Derive macro for struct validation (like validator)
#[derive(Validate)]
struct User {
    #[validate(rules = "username_rules()")]
    username: String,
}

// But rules are runtime Rule<T> trees
fn username_rules() -> Rule<String> {
    Rule::All(vec![
        Rule::MinLength(3),
        Rule::MaxLength(20),
        Rule::Pattern(Arc::new(Regex::new(r"^[a-z_]+$").unwrap())),
    ])
}
```

This gives:
- Ergonomics of derive macros for struct validation
- Flexibility of runtime rule trees
- Reusable rules across fields/structs

### 7. Summary

The **Fresh Approach** fills a different niche than `validator`/`garde`:

- `validator`/`garde`: "I have a struct, validate its fields at compile-time"
- **Fresh Approach**: "I have validation rules as data, apply them to values dynamically"

Both are valid patterns - the Fresh Approach is better suited for:
- Dynamic/configurable validation
- Form builders
- API input validation with runtime rules
- Multi-tenant apps with per-tenant rules
- Validation rule editors/builders

---

## Example Usage (Proposed)

```rust
use walrs_inputfilter::{Rule, FieldRule, FormRules};

// Simple field validation
let username_rules = FieldRule::new("username")
    .required()
    .min_length(3)
    .max_length(20)
    .pattern(r"^[a-zA-Z0-9_]+$");

// Composable rules
let password_rules = FieldRule::new("password")
    .required()
    .add_rule(Rule::All(vec![
        Rule::MinLength(8),
        Rule::Custom(Arc::new(|p| {
            if p.chars().any(|c| c.is_uppercase()) {
                Ok(())
            } else {
                Err(Violation::new(TypeMismatch, "Must contain uppercase"))
            }
        })),
    ]));

// Form-level validation
let form = FormRules::new()
    .field(username_rules)
    .field(password_rules)
    .cross_validate(|data| {
        // password_confirm == password
    });

// Thread-safe sharing
let shared_form: Arc<FormRules> = Arc::new(form);

// Use in async handler
async fn handler(shared_form: Arc<FormRules>, input: FormInput) {
    match shared_form.validate(&input) {
        Ok(()) => { /* proceed */ },
        Err(violations) => { /* return errors */ },
    }
}
```

---

## Implementation Phases

### Phase 1: Core Rule Engine
1. Define `Rule<T>` enum with basic variants
2. Implement `validate()` for each variant
3. Add `RuleBuilder` for fluent construction

### Phase 2: Field Rules
1. Create `FieldRule<T>` struct
2. Add filter support
3. Add name/path for error context

### Phase 3: Form Rules
1. Create `FormRules` for multi-field
2. Add cross-field validation
3. Add nested form support

### Phase 4: Integration
1. Add serde support for `Rule` serialization
2. Add derive macro for struct validation
3. Migration guide from old API

---

## Benefits

1. **Simpler mental model** - Rules are data, not closures
2. **True tree structure** - Naturally composable via enum variants
3. **Thread-safe by default** - `Arc<Rule>` instead of lifetime juggling
4. **Serializable** - Rules can be loaded from config files
5. **Pattern matchable** - Can inspect/transform rules programmatically
6. **No lifetime complexity** - `'static` rules that can be stored anywhere

---

## Open Questions

1. **How to handle different value types in FormRules?** - Type erasure via `Box<dyn Any>` or separate validation per type?
2. **Should filters be part of Rule or separate?** - Current proposal keeps them in `FieldRule`
3. **Async validation support?** - Could add `AsyncRule` variant or separate trait
4. **Error message customization?** - Per-rule messages vs global message providers?

---

## Migration Path

To maintain backward compatibility:

1. Keep existing `Input`/`RefInput` as deprecated
2. Implement new `Rule`-based API alongside
3. Provide adapters: `Rule::from_legacy(Input)` 
4. Document migration in CHANGELOG
5. Remove deprecated API in next major version

---

## References

- Current design: [DESIGN.md](./DESIGN.md)
- Inspiration: [laminas-inputfilter](https://docs.laminas.dev/laminas-inputfilter/)
- Similar Rust crates: `validator`, `garde`, `validify`

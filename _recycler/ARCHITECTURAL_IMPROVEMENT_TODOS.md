# Possible Architectural Improvements for `walrs_inputfilter`

## Overview

This document outlines potential architectural improvements for the `walrs_inputfilter` crate, categorized by priority and complexity.

---

## 1. Trait Architecture Improvements

### 1.1 Unify `FilterForSized` and `FilterForUnsized` Traits (Medium Complexity)
**Current State:** Two separate traits with nearly identical method signatures.

**Improvement:** Consider using a single trait with GATs (Generic Associated Types) or associated type defaults:

```rust
pub trait InputFilter<T>: Display + Debug {
    type Output;
    
    fn validate_detailed(&self, value: T) -> ValidationResult;
    fn filter_detailed(&self, value: T) -> FilterResult<Self::Output>;
    // ... with default implementations for non-detailed versions
}
```

**Benefits:**
- Reduces trait duplication
- Simplifies the mental model for users
- Easier to extend in the future

**Trade-offs:**
- May require more complex generic bounds
- Potential breaking change for existing users

---

### 1.2 Separate Validation and Filtering Concerns (Low Complexity)
**Current State:** `FilterForSized`/`FilterForUnsized` combine validation AND filtering.

**Improvement:** Consider separate traits:
```rust
pub trait Validatable<T> {
    fn validate(&self, value: T) -> ValidationResult;
}

pub trait Filterable<T> {
    type Output;
    fn filter(&self, value: T) -> Self::Output;
}

pub trait InputProcessor<T>: Validatable<T> + Filterable<T> {
    fn process(&self, value: T) -> FilterResult<Self::Output>;
}
```

**Benefits:**
- Better separation of concerns
- Users can implement just validation or just filtering
- More composable

---

### 1.3 Add `ValidatorChain` and `FilterChain` Types (Medium Complexity)
**Current State:** Validators and filters are stored as `Vec<&'a dyn Fn(...)>`.

**Improvement:** Create dedicated chain types:
```rust
pub struct ValidatorChain<'a, T> {
    validators: Vec<&'a OwnedValidator<T>>,
    break_on_failure: bool,
}

impl<T> ValidatorChain<'_, T> {
    pub fn validate(&self, value: T) -> ValidationResult { ... }
    pub fn and(self, validator: &OwnedValidator<T>) -> Self { ... }
}
```

**Benefits:**
- Cleaner API for composing validators
- Better type safety
- Can add chain-specific optimizations

---

## 2. Struct Architecture Improvements

### 2.1 Consolidate `Input` and `RefInput` Further (High Complexity)
**Current State:** Two separate structs with ~80% identical code.

**Options:**

**Option A: Generic Input with Marker Traits**
```rust
pub struct Input<'a, T, FT, Mode = Owned> 
where
    Mode: InputMode<T, FT>,
{
    // shared fields...
    _mode: PhantomData<Mode>,
}

pub trait InputMode<T, FT> {
    type ValidatorType;
    fn validate(validator: &Self::ValidatorType, value: ???) -> ValidatorResult;
}
```

**Option B: Macro-based generation**
Use a macro to generate both `Input` and `RefInput` from a common template.

**Benefits:**
- Eliminates code duplication
- Single source of truth for logic

**Trade-offs:**
- More complex implementation
- May affect compile times

---

### 2.2 Builder Pattern Improvements (Low Complexity)
**Current State:**
- `Input` uses `derive_builder`
- `RefInput` has a manual builder

**Improvement:** Standardize on one approach, add fluent builder methods:
```rust
impl<'a, T, FT> Input<'a, T, FT> {
    pub fn builder() -> InputBuilder<'a, T, FT> { ... }
    
    // Fluent modifiers
    pub fn with_validator(mut self, v: &'a OwnedValidator<T>) -> Self { ... }
    pub fn with_filter(mut self, f: &'a FilterFn<FT>) -> Self { ... }
    pub fn required(mut self) -> Self { ... }
}
```

---

### 2.3 Add `InputGroup` / `InputSet` for Multi-Field Validation (Medium Complexity)
**Improvement:** Add a container for validating multiple related inputs:
```rust
pub struct InputGroup<'a> {
    inputs: HashMap<&'a str, Box<dyn AnyInput>>,
    cross_validators: Vec<Box<dyn Fn(&InputGroup) -> ValidationResult>>,
}

impl InputGroup<'_> {
    pub fn validate_all(&self, values: &HashMap<&str, Value>) -> Result<(), GroupViolations>;
}
```

**Benefits:**
- Supports form-level validation
- Cross-field validation (e.g., password confirmation)
- Better integration with web frameworks

---

## 3. Validator Architecture Improvements

### 3.1 Add Validator Combinators (Low Complexity)
**Improvement:** Add combinators for composing validators:
```rust
// AND combinator
let validator = length_validator.and(pattern_validator);

// OR combinator  
let validator = email_validator.or(phone_validator);

// NOT combinator
let validator = not(empty_validator);

// Conditional
let validator = when(|v| v.len() > 0, pattern_validator);
```

---

### 3.2 Unify Validator Traits (Low Complexity)
**Current State:** `Validate<T>` and `ValidateRef<T>` in `validators/traits.rs`.

**Improvement:** Consider using a single trait with blanket implementations:
```rust
pub trait Validator<T: ?Sized> {
    fn validate(&self, value: &T) -> ValidatorResult;
}

// Blanket impl for Copy types
impl<T: Copy, V: Validator<T>> ValidatorOwned<T> for V {
    fn validate_owned(&self, value: T) -> ValidatorResult {
        self.validate(&value)
    }
}
```

---

### 3.3 Add Async Validator Support (Medium Complexity)
**Improvement:** For validators that need async operations (e.g., database uniqueness checks):
```rust
#[async_trait]
pub trait AsyncValidator<T: ?Sized> {
    async fn validate(&self, value: &T) -> ValidatorResult;
}

pub struct AsyncInput<'a, T, FT> {
    // ... similar to Input but with async validators
}
```

---

## 4. Filter Architecture Improvements

### 4.1 Chain Filter with Different Output Types (Low Complexity)
**Current State:** `FilterFn<T>` returns `T`.

**Improvement:** Support type-changing filters:
```rust
pub trait TypedFilter<In, Out> {
    fn filter(&self, value: In) -> Out;
}

// Chainable
impl<A, B, C, F1, F2> TypedFilter<A, C> for Chain<F1, F2>
where
    F1: TypedFilter<A, B>,
    F2: TypedFilter<B, C>,
{
    fn filter(&self, value: A) -> C {
        self.1.filter(self.0.filter(value))
    }
}
```

---

### 4.2 Add Common Filter Presets (Low Complexity)
**Improvement:** Add commonly used filter combinations:
```rust
pub mod presets {
    pub fn trim_and_lowercase() -> impl Filter<String> { ... }
    pub fn sanitize_html() -> impl Filter<String> { ... }
    pub fn normalize_whitespace() -> impl Filter<String> { ... }
}
```

---

## 5. Error/Violation Architecture Improvements

### 5.1 Add Violation Context/Path (Medium Complexity)
**Improvement:** Add field path context to violations:
```rust
pub struct Violation {
    pub violation_type: ViolationType,
    pub message: String,
    pub path: Option<String>,      // e.g., "user.email"
    pub context: Option<Value>,    // Additional context
}
```

**Benefits:**
- Better error messages for nested structures
- Easier to map violations to form fields

---

### 5.2 Add Violation Codes (Low Complexity)
**Improvement:** Add machine-readable codes for i18n:
```rust
pub struct Violation {
    pub violation_type: ViolationType,
    pub code: &'static str,        // e.g., "validation.too_short"
    pub message: String,
    pub params: HashMap<String, Value>, // e.g., {"min": 5, "actual": 3}
}
```

---

## 6. API/Ergonomics Improvements

### 6.1 Add Derive Macro for Struct Validation (High Complexity)
**Improvement:** Add derive macro for automatic Input generation:
```rust
#[derive(Validate)]
pub struct UserInput {
    #[validate(required, length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(required, email)]
    pub email: String,
    
    #[validate(range(min = 0, max = 150))]
    pub age: Option<u8>,
}

// Auto-generates validation logic
let result = user_input.validate()?;
```

---

### 6.2 Add Serde Integration (Low Complexity)
**Improvement:** Add serde support for serializing validation rules:
```rust
#[derive(Serialize, Deserialize)]
pub struct SerializableValidator {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    // ...
}
```

**Benefits:**
- Can load validation rules from config files
- Share validation rules with frontend (JSON Schema compatibility)

---

### 6.3 Add JSON Schema Generation (Medium Complexity)
**Improvement:** Generate JSON Schema from validators:
```rust
impl ToJsonSchema for LengthValidator<str> {
    fn to_json_schema(&self) -> serde_json::Value {
        json!({
            "type": "string",
            "minLength": self.min_length,
            "maxLength": self.max_length
        })
    }
}
```

---

## 7. Performance Improvements

### 7.1 Add Lazy Validation Mode (Low Complexity)
**Improvement:** Only validate when explicitly requested:
```rust
pub struct LazyInput<'a, T, FT> {
    input: Input<'a, T, FT>,
    cached_result: OnceCell<ValidationResult>,
}
```

---

### 7.2 Add Parallel Validation (Medium Complexity)
**Improvement:** Run validators in parallel using Rayon:
```rust
impl Input<'_, T, FT> {
    pub fn validate_parallel(&self, value: T) -> ValidationResult {
        self.validators.par_iter()
            .filter_map(|v| v(value).err())
            .collect()
    }
}
```

---

## Priority Matrix

| Improvement | Complexity | Impact | Priority |
|------------|-----------|--------|----------|
| 6.1 Derive Macro | High | High | P1 |
| 2.3 InputGroup | Medium | High | P1 |
| 3.1 Validator Combinators | Low | Medium | P2 |
| 5.1 Violation Context | Medium | Medium | P2 |
| 4.2 Filter Presets | Low | Low | P3 |
| 1.1 Unify Traits | Medium | Medium | P3 |
| 3.3 Async Validators | Medium | Medium | P3 |
| 6.3 JSON Schema | Medium | Medium | P3 |

---

## Summary

The crate has a solid foundation. The highest-impact improvements would be:

1. **Derive macro for struct validation** - Would dramatically improve ergonomics
2. **InputGroup for multi-field validation** - Essential for real-world form validation
3. **Validator combinators** - Low effort, good DX improvement
4. **Violation context/path** - Better error handling for complex structures

Would you like me to implement any of these improvements?

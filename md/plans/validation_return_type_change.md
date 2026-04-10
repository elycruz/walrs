# Plan: Change Validation Return Types from `Result` to `Option`

## Problem Statement

Item 9 from `md/plans/random.md`:
> Consider changing all 'validation' results to `Option<Violation>` and `Option<Violations>` respectively.
> Sync validation is not fallible so it shouldn't return `Result<...>`, also since `Result<...>` creates more overhead in logic, it shouldn't be used for this use case.

An existing discussion document (`md/discussions/validate_return_type.md`) already recommends this change.

---

## Current State

### Type Aliases
| Alias | Definition | Location |
|-------|-----------|----------|
| `RuleResult` | `Result<(), Violation>` | `crates/validation/src/rule.rs:123` |
| `ValidatorResult` | `Result<(), Violation>` | `crates/validation/src/traits.rs:5` |

### Return Types by Layer
| Layer | Return Type | Crate |
|-------|------------|-------|
| Rule (internal, fail-fast) | `RuleResult` = `Result<(), Violation>` | `walrs_validation` |
| Rule `_all` (internal, fail-slow) | `Result<(), Violations>` | `walrs_validation` |
| Validate/ValidateRef traits (public) | `ValidatorResult` = `Result<(), Violation>` | `walrs_validation` |
| ValidateAsync/ValidateRefAsync traits | `ValidatorResult` | `walrs_validation` |
| Field validation | `Result<(), Violations>` | `walrs_inputfilter` |
| FieldFilter validation | `Result<(), FormViolations>` | `walrs_inputfilter` |
| Form validation | `Result<(), FormViolations>` | `walrs_form` |
| Rule::Custom closures | `Arc<dyn Fn(&T) -> RuleResult>` | `walrs_validation` |
| Rule::CustomAsync closures | `... -> Pin<Box<dyn Future<Output = RuleResult>>>` | `walrs_validation` |

### Scale of Change
- **~90** `RuleResult`/`ValidatorResult` references in production code
- **~344** `Ok(())`/`Err(Violation::...)` patterns in rule_impls
- **~14** `?` operator usages on validate calls in production code
- **~622** `is_ok()`/`is_err()`/`unwrap_err()` usages across all code (including tests)

### No True Fallibility (sync)
Every sync validation function produces either "valid" or "violation" — none can fail with I/O, parsing, or system errors. URL/IP/email parse errors are caught internally and converted to `Violation` values. This confirms `Result` is semantically incorrect for sync validators.

---

## Analysis: Pros & Cons

### Pros

1. **Semantic correctness** — Validation is a *query* ("is there a violation?"), not a *fallible operation*. A violation is an expected, normal outcome — not an unexpected error. `Option` conveys this accurately.

2. **Aligns with method renaming** — The planned rename of `validate*` → `check_validity*`/`check*` reads naturally with `Option`:
   ```rust
   fn check_validity(&self, value: &str) -> Option<Violation>  // "optionally get back a violation"
   ```

3. **Cleaner aggregation** — Collecting violations from multiple fields becomes idiomatic:
   ```rust
   // Option-based: natural with filter_map
   let violations: Vec<_> = fields.iter()
     .filter_map(|f| f.check(value))
     .collect();
   
   // vs. Result-based: awkward .err() calls
   let violations: Vec<_> = fields.iter()
     .filter_map(|f| f.validate(value).err())
     .collect();
   ```

4. **Reduced cognitive overhead** — `Result` suggests "something went wrong" to readers. But finding a violation isn't a program error — it's the expected purpose of the function. Using `Option` prevents this misinterpretation.

5. **`#[must_use]` already enforced** — Both `Violation` and `Violations` are marked `#[must_use]`, and `Option` is also `#[must_use]` by default. No loss of compile-time safety.

6. **`impl Error` becomes optional** — `Violation`/`Violations` implement `Error` today (violation.rs:219, 356). With `Option` returns, these impls are only needed for the `process()` path where violations end up inside `Result`. They can be kept but are no longer load-bearing for the primary API.

### Cons

1. **Massive refactor scope** — This is a cross-cutting change touching 3 crates, ~90 type alias usages, ~344 `Ok`/`Err` patterns, and extensive test rewrites. High risk of regressions.

2. **Loss of `?` operator for early return** — 14 production call-sites use `?` on validation results for short-circuit returns (in `Rule::All` chaining and `Field::process()`). These become:
   ```rust
   // Before:
   rule.validate_str_inner(value, locale)?;
   
   // After:
   if let Some(v) = rule.check_str_inner(value, locale) {
     return Some(v);
   }
   ```
   This is slightly more verbose but equally clear. A helper macro could restore conciseness if desired.

3. **`process()` methods require conversion** — `Field::process()` and `FieldFilter::process()` chain `try_filter()` (genuinely fallible, returns `Result`) with validation. The validation `Option` must be converted:
   ```rust
   // Field::process()
   let filtered = self.try_filter(filtered)?;          // stays Result
   if let Some(v) = self.check_ref(&filtered) {       // Option → Result bridge
     return Err(v.into());
   }
   Ok(filtered)
   ```

4. **Custom closure signature change** — `Rule::Custom(Arc<dyn Fn(&T) -> RuleResult>)` changes to `-> Option<Violation>`. This breaks all user-provided custom validators — a **semver-breaking** change.

5. **Test churn** — ~622 test assertions need updating: `is_ok()` → `is_none()`, `is_err()` → `is_some()`, `unwrap_err()` → `unwrap()`. Mechanical but time-consuming.

6. **External adoption friction** — Users familiar with Rust's `Result`-based validation (e.g., `validator` crate) may find `Option` unfamiliar initially. However, the semantic improvement is worth the adjustment.

### The Async Validator Question

This is the most important design consideration. Async validators (`Rule::CustomAsync`, `CrossFieldRuleType::CustomAsync`) can perform **genuinely fallible** operations: database lookups, API calls, file system checks, etc. These can fail with I/O errors, timeouts, or connection failures — not just produce violations.

**Current state** — async validators return `RuleResult = Result<(), Violation>`, the same as sync. This means:
- A database connection failure must be shoehorned into `Violation(CustomError, "database error")`
- Callers cannot distinguish "the input is invalid" from "I couldn't check the input"
- The current API **already has this conflation problem** — it's not something we'd introduce

**Three options:**

#### Option A: Split sync/async return types (recommended)

```rust
// Sync — pure query, cannot fail
type CheckResult = Option<Violation>;

// Async — can fail due to I/O, network, etc.
type AsyncCheckResult = Result<Option<Violation>, Box<dyn Error + Send + Sync>>;
// Ok(None)       → valid
// Ok(Some(v))    → invalid (found a violation)
// Err(e)         → check couldn't be performed (I/O error, timeout, etc.)
```

**Pros:**
- Most semantically correct — properly models all three states for async
- Actually *improves* on the current design (which conflates violations with I/O errors)
- `?` operator works naturally on async paths for I/O error propagation
- Sync paths get the clean `Option` semantics
- Users can now distinguish "invalid" from "check failed" in async callers

**Cons:**
- Two different return types for sync vs async paths
- Async `_inner` methods handle both sync and async rules in a single traversal, requiring conversion:
  ```rust
  // In async traversal, a sync rule returns Option<Violation>
  // Convert to async result:
  if let Some(v) = sync_rule.check_inner(value, locale) {
    return Ok(Some(v));
  }
  // An async rule returns Result<Option<Violation>, E>:
  if let Some(v) = async_rule_closure(value).await? {
    return Ok(Some(v));
  }
  ```

**User-facing async closure becomes clearer:**
```rust
Rule::custom_async(Arc::new(|value: &String| {
  Box::pin(async move {
    // I/O errors propagate naturally with ?
    let exists = db.check_username(value).await?;
    if exists {
      Ok(Some(Violation::new(CustomError, "Username taken.")))
    } else {
      Ok(None) // valid
    }
  })
}))
```

This is **clearer** than today's `Err(Violation::new(...))` pattern.

#### Option B: Use `Option<Violation>` for everything (sync + async)

Treat async validators the same as sync — `Option<Violation>`. I/O errors from async closures would need to be caught inside the closure and converted to `Some(Violation(CustomError, ...))`.

**Pros:**
- Simpler, consistent return type everywhere
- Minimal design overhead

**Cons:**
- Still conflates I/O errors with violations (same problem as today, just with `Some` instead of `Err`)
- Callers of async validation still can't distinguish "invalid input" from "check failed"

#### Option C: Wrap both sync and async in a unified result enum

```rust
enum CheckOutcome {
  Valid,
  Invalid(Violation),
  Failed(Box<dyn Error + Send + Sync>), // only possible for async
}
```

**Pros:**
- Single type for all paths
- Explicit about all three states

**Cons:**
- More complex than needed for sync (where `Failed` is impossible)
- Loses `Option`/`Result` ergonomics (no `?`, no `filter_map`, etc.)
- Invents a new type when Rust's existing `Result<Option<V>, E>` does the job

### Async Recommendation

**Option A (split sync/async)** is the most Rust-idiomatic and actually *fixes* a pre-existing deficiency in the async path. The sync/async split is natural because the two contexts have fundamentally different failure modes. The conversion cost in async traversal methods is minimal and localized.

This means:
- `Validate<T>` / `ValidateRef<T>` traits return `Option<Violation>`
- `ValidateAsync<T>` / `ValidateRefAsync<T>` traits return `Result<Option<Violation>, Box<dyn Error + Send + Sync>>`
- `Rule::Custom` closures: `Arc<dyn Fn(&T) -> Option<Violation>>`
- `Rule::CustomAsync` closures: `Arc<dyn Fn(&T) -> Pin<Box<dyn Future<Output = Result<Option<Violation>, Box<dyn Error + Send + Sync>>>>>>`

The async closure is verbose in raw form, but `Rule::custom_async()` already encapsulates construction. A type alias helps:
```rust
pub type AsyncCheckResult = Result<Option<Violation>, Box<dyn Error + Send + Sync>>;
```

### Neutral / Design Decisions

- **`process()` / `process_async()` return type stays `Result`** — Because `try_filter` is genuinely fallible, the `process()` pipeline must remain `Result`. For sync `process()`, the `Option` from validation is bridged with `if let Some(v)`. For `process_async()`, the nested `Result<Option<Violation>, E>` needs to be mapped into the pipeline's error type.
- **`impl Error for Violation`/`Violations`** — Keep these impls. They're still useful when violations end up inside `Result` (via `process()`) and for `Box<dyn Error>` interop.
- **Trait renaming** — This change is tightly coupled with the item 5 rename (`validate*` → `check*`). Doing both together makes sense to avoid two breaking changes.
- **`Violations` (plural) at field/form level** — `Field::check()` returns `Option<Violations>`, `FieldFilter::check()` returns `Option<FormViolations>`. The async versions wrap these in `Result<..., Box<dyn Error + Send + Sync>>`.

---

## Recommendation: Yes, This Change Makes Sense

**The semantic argument is strong and the codebase already agrees with itself** — the discussion doc (`md/discussions/validate_return_type.md`) concluded `Option<Violation>` is the better fit, and the rationale in `random.md` is sound.

The core insight is: **validation never fails — it always succeeds and either finds violations or doesn't.** Every `Err(Violation::range_overflow(...))` in the codebase is actually a successful discovery of invalid input, not a program error. Using `Result` for this is a semantic lie that:

- Misleads readers into thinking validation can fail for unexpected reasons
- Forces awkward `.err()` calls when collecting violations
- Conflates "I found a problem with your input" with "something went wrong in my code"

The main counter-argument is **cost vs. payoff**: ~344 `Ok`/`Err` flips and ~622 test assertion changes for what is functionally identical behavior. However:

1. **This is a library** — API precision matters more than in application code. Downstream users benefit from types that communicate intent.
2. **The crate appears pre-1.0** — breaking changes are expected now; they get more expensive later.
3. **It's tightly coupled with the `validate*` → `check*` rename** (item 5) — bundling both into one breaking change is efficient. Doing them separately would mean *two* breaking migrations for users.
4. **The `?` operator loss is minimal** — only 14 sites in production code, all easily converted to `if let Some(v)` patterns.
5. **The mechanical churn is large but low-risk** — `Ok(())` → `None` and `Err(v)` → `Some(v)` are 1:1 transforms with no logic changes. The compiler catches every missed site.

**One caveat**: We recommend doing items 5 and 9 together as a single breaking change. Renaming methods *and* changing return types at the same time gives users one migration instead of two, and the rename + return type change reinforce each other conceptually.

---

## Implementation Phases

### Phase 1: Core types (walrs_validation)

- Change `RuleResult` to `type RuleResult = Option<Violation>;`
- Add `type AsyncRuleResult = Result<Option<Violation>, Box<dyn Error + Send + Sync>>;`
- Change `ValidatorResult` to `type ValidatorResult = Option<Violation>;`
- Add `type AsyncValidatorResult = Result<Option<Violation>, Box<dyn Error + Send + Sync>>;`
- Update `Validate<T>`, `ValidateRef<T>` trait signatures → return `Option<Violation>`
- Update `ValidateAsync<T>`, `ValidateRefAsync<T>` → return `AsyncValidatorResult`
- Update all sync rule_impls: `Ok(())` → `None`, `Err(violation)` → `Some(violation)`
- Update async rule_impls: adapt to `Result<Option<Violation>, E>`, bridge sync→async
- Update `?` early-returns in sync `Rule::All`/`Rule::Any` to `if let Some(v)` pattern
- Update `_all` methods: `Result<(), Violations>` → `Option<Violations>`
- Update `Rule::Custom` closure signature: `Fn(&T) -> Option<Violation>`
- Update `Rule::CustomAsync` closure signature: `Fn(&T) -> Pin<Box<dyn Future<Output = AsyncRuleResult>>>`
- Update validation crate tests

### Phase 2: Input filter (walrs_inputfilter)

- Update `Field<String>` sync validation methods: `Result<(), Violations>` → `Option<Violations>`
- Update `Field<Value>` sync validation methods similarly
- Update `Field` async validation methods: → `Result<Option<Violations>, Box<dyn Error + Send + Sync>>`
- Update `Field::process()` to bridge sync `Option` → `Result`
- Update `Field::process_async()` to handle async result + bridge
- Update `FieldFilter::validate()`: `Result<(), FormViolations>` → `Option<FormViolations>`
- Update `FieldFilter::validate_async()`: → `Result<Option<FormViolations>, Box<dyn Error + Send + Sync>>`
- Update `FieldFilter::process()` / `process_async()` bridges
- Update `CrossFieldRule::evaluate()`: `RuleResult` → `Option<Violation>`
- Update `CrossFieldRuleType::CustomAsync` signature
- Update inputfilter crate tests and examples

### Phase 3: Form (walrs_form)

- Update `Form::validate()`: `Result<(), FormViolations>` → `Option<FormViolations>`
- Update element `validate_value()` methods: `Result<(), Violations)` → `Option<Violations>`
- Update form crate tests and examples

### Phase 4: Docs, benchmarks, README

- Update all doc comments reflecting new return types
- Update `md/discussions/validate_return_type.md` to mark as implemented
- Update examples in READMEs
- Update benchmarks
- Run full test suite + coverage

---

## Notes

- This is a **semver-breaking** change across all 3 crates.
- Tightly coupled with TODO item 5 (rename `validate*` → `check*`). Consider bundling them.
- A helper macro `check_bail!` could restore `?`-like ergonomics if the `if let Some` pattern proves too verbose.

# Plan: Move `Filter<T>` Enum to `walrs_filter` Crate

> Date: 2026-02-22 (updated: 2026-02-22)
> Related Issue: #85

## Summary

Move the `Filter<T>` enum from `walrs_inputfilter` into `walrs_filter`, co-locating it with the
filter structs (`SlugFilter`, `StripTagsFilter`, `XmlEntitiesFilter`). This mirrors how `Rule<T>`
lives in `walrs_validation` alongside the validation logic (in `rule_impls/`), with
`walrs_inputfilter` re-exporting it.

## Motivation — Following the `Rule<T>` Pattern

The `walrs_validation` crate recently completed a refactor where:

1. Legacy validator structs (`LengthValidator`, `PatternValidator`, etc.) were removed.
2. All validation logic was consolidated into the `Rule<T>` enum.
3. Type-specific impl blocks were organized under `rule_impls/` (`string.rs`, `length.rs`,
   `scalar.rs`, `steppable.rs`), registered as `pub(crate)` modules.
4. `walrs_inputfilter` re-exports `Rule<T>` via:
   `pub use walrs_validation::rule::{Condition, Rule, RuleResult};`

The filter side should follow the same pattern: **the `Filter<T>` enum should live in
`walrs_filter`** (alongside the filter structs), with type-specific `apply()` impls organized
similarly, and `walrs_inputfilter` re-exporting it.

## Current State

### Validation (`walrs_validation`) — the pattern to follow

```
crates/validation/src/
├── lib.rs              # pub use rule::{Rule, ...}; pub(crate) mod rule_impls;
├── rule.rs             # Rule<T> enum, Condition<T>, CompiledRule<T>, combinators
├── rule_impls/
│   ├── mod.rs          # pub(crate) mod string; pub(crate) mod length; ...
│   ├── string.rs       # impl Rule<String> { validate_str(), validate_str_all(), ... }
│   ├── length.rs       # impl<T: WithLength> Rule<T> { validate_len(), ... }
│   ├── scalar.rs       # impl<T: ScalarValue + IsEmpty> Rule<T> { validate_scalar(), ... }
│   ├── steppable.rs    # impl<T: SteppableValue + IsEmpty> Rule<T> { validate_step(), ... }
│   └── attributes.rs   # impl Rule<T> { to_attributes_list() }
├── traits.rs           # Validate, ValidateRef, IsEmpty, WithLength, ScalarValue, ...
├── violation.rs        # Violation, Violations, ViolationType
├── message.rs          # Message<T>, MessageContext, MessageParams
├── value.rs            # Value (re-export of serde_json::Value), ValueExt
└── attributes.rs       # Attributes type
```

Key design points:
- `Rule<T>` is the **single composable enum** (serializable, with `All`/`Any`/`Not`/`When`/`Custom`)
- Type-specific validation lives in `rule_impls/` as inherent `impl Rule<ConcreteType>` blocks
- `pub(crate)` modules keep impl organization internal — users only see `Rule<T>` methods
- No separate validator structs (they were removed in the recent refactor)

### Filters (`walrs_filter` + `walrs_inputfilter`) — what needs to change

```
crates/filter/src/
├── lib.rs              # Re-exports: Filter (trait), SlugFilter, StripTagsFilter, XmlEntitiesFilter
├── traits.rs           # pub trait Filter<T> { type Output; fn filter(&self, value: T) -> Self::Output; }
├── slug.rs             # SlugFilter struct, impl Filter<Cow<str>> for SlugFilter
├── strip_tags.rs       # StripTagsFilter struct, impl Filter<Cow<str>> for StripTagsFilter
└── xml_entities.rs     # XmlEntitiesFilter struct, impl Filter<Cow<str>> for XmlEntitiesFilter

crates/inputfilter/src/
├── filter_enum.rs      # ⬅ Filter<T> enum lives HERE (should be in walrs_filter)
│                       #   - enum definition (Trim, Lowercase, StripTags, Slug, Clamp, Chain, Custom)
│                       #   - impl Filter<String> { apply() }
│                       #   - impl Filter<Value> { apply() }
│                       #   - impl_numeric_filter! macro (i32, i64, f32, f64)
├── field.rs            # Field<T> { filters: Option<Vec<Filter<T>>>, ... }
│                       #   - uses `use crate::filter_enum::Filter;`
├── field_filter.rs     # FieldFilter — references `crate::filter_enum::Filter` in tests
├── filters/mod.rs      # pub use walrs_filter::*; (re-export)
└── lib.rs              # pub use filter_enum::Filter;
```

**Problems:**
- `Filter<T>` enum is in `walrs_inputfilter` but delegates to `walrs_filter` structs — it
  should live alongside them.
- **Naming collision**: `Filter` trait vs `Filter<T>` enum. Currently resolved via aliasing:
  `use walrs_filter::Filter as FilterTrait` in `filter_enum.rs`.
- The filter structs (`SlugFilter`, etc.) are **not** being removed (unlike legacy validators),
  because the enum delegates to them — they provide the actual transformation logic.

## Naming Decision

**Recommended: Rename the trait to `FilterFn`** — the enum keeps the name `Filter<T>`.

| Option | Enum Name | Trait Name | Pros | Cons |
|--------|-----------|------------|------|------|
| **A (recommended)** | `Filter<T>` | `FilterFn` | Enum is the primary public API; users write `Filter::Trim` | Trait rename touches filter structs |
| B | `FilterOp<T>` | `Filter` | Trait keeps its name | Less ergonomic: `FilterOp::Trim` everywhere |
| C | `Filter<T>` | `Filterable` | Clear meaning | Misleading — trait is on the filter, not the value |

**Rationale**: The enum is the primary public API (used in `Field<T>`, serialized in configs,
referenced in examples and tests). The trait is an implementation detail used internally by
three filter structs. `FilterFn` clearly communicates "a callable that transforms input" and
parallels Rust's `Fn`/`FnMut`/`FnOnce` naming convention.

## Structural Comparison: Before & After

### Before (current)

| Concept | Validation | Filters |
|---------|-----------|---------|
| Composable enum | `Rule<T>` in `walrs_validation` ✅ | `Filter<T>` in `walrs_inputfilter` ❌ |
| Impl structs | *(removed — logic inline in `Rule<T>`)* | `SlugFilter`, `StripTagsFilter`, `XmlEntitiesFilter` in `walrs_filter` |
| Trait | `Validate`/`ValidateRef` in `walrs_validation` | `Filter` in `walrs_filter` |
| Re-export layer | `walrs_inputfilter` re-exports `Rule<T>` | *(no re-export — enum defined in wrong crate)* |

### After (proposed)

| Concept | Validation | Filters |
|---------|-----------|---------|
| Composable enum | `Rule<T>` in `walrs_validation` | `Filter<T>` in `walrs_filter` |
| Impl structs | *(none)* | `SlugFilter`, `StripTagsFilter`, `XmlEntitiesFilter` in `walrs_filter` |
| Trait | `Validate`/`ValidateRef` in `walrs_validation` | `FilterFn` in `walrs_filter` |
| Type-specific impls | `rule_impls/` in `walrs_validation` | `filter_impls/` in `walrs_filter` |
| Re-export layer | `walrs_inputfilter` re-exports `Rule<T>` | `walrs_inputfilter` re-exports `Filter<T>` |

## Steps

### Step 1: Rename `Filter` trait → `FilterFn`

Rename the trait to free up the `Filter` name for the enum.

**Files to modify:**
- `crates/filter/src/traits.rs` — `pub trait Filter<T>` → `pub trait FilterFn<T>`; update doc examples
- `crates/filter/src/slug.rs` — `use crate::Filter;` → `use crate::FilterFn;`; `impl Filter<Cow<str>> for SlugFilter` → `impl FilterFn<Cow<str>> for SlugFilter`
- `crates/filter/src/strip_tags.rs` — same pattern
- `crates/filter/src/xml_entities.rs` — same pattern
- `crates/filter/src/lib.rs` — update re-exports and doc examples (`use walrs_filter::FilterFn`)
- `crates/filter/README.md` — update trait examples
- `crates/filter/examples/basic_filters.rs` — `use walrs_filter::{Filter, ...}` → `use walrs_filter::{FilterFn, ...}`
- `crates/filter/examples/filter_chain.rs` — same
- `crates/filter/benches/filter_benchmarks.rs` — `use walrs_filter::Filter` → `use walrs_filter::FilterFn`

### Step 2: Add `serde` and `serde_json` dependencies to `walrs_filter`

**File:** `crates/filter/Cargo.toml`

Add to `[dependencies]`:
```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

These are needed for `#[derive(Serialize, Deserialize)]` on the `Filter<T>` enum and for
the `impl Filter<serde_json::Value>` block. No dependency on `walrs_validation` is needed —
use `serde_json::Value` directly (it's the same type that `walrs_validation::Value` re-exports).

### Step 3: Move `Filter<T>` enum into `walrs_filter`

Create the enum module and (optionally) a `filter_impls/` directory for organized type-specific
impls, following the `rule_impls/` pattern.

**Create:** `crates/filter/src/filter.rs`

Move from `crates/inputfilter/src/filter_enum.rs`:
- The `Filter<T>` enum definition (with `#[derive(Clone, Serialize, Deserialize)]`)
- `Debug` impl for `Filter<T>`
- `PartialEq` impl for `Filter<T>`

**Create:** `crates/filter/src/filter_impls/mod.rs`
```rust
pub(crate) mod string;
pub(crate) mod numeric;
pub(crate) mod value;
```

**Create:** `crates/filter/src/filter_impls/string.rs`
```rust
// impl Filter<String> { pub fn apply() } — string filter logic
// Delegates to crate::SlugFilter, crate::StripTagsFilter, crate::XmlEntitiesFilter
// using crate::FilterFn trait
```

**Create:** `crates/filter/src/filter_impls/numeric.rs`
```rust
// impl_numeric_filter! macro + invocations (i32, i64, f32, f64)
```

**Create:** `crates/filter/src/filter_impls/value.rs`
```rust
// impl Filter<serde_json::Value> { pub fn apply() }
// Uses serde_json::Value directly (no walrs_validation dependency needed)
```

**Update:** `crates/filter/src/lib.rs`
```rust
pub mod filter;
pub(crate) mod filter_impls;
// ...existing modules...
pub use filter::Filter;
```

**Note:** Since `Value` is just `serde_json::Value`, the `Filter<Value>` impl can live directly
in `walrs_filter` without introducing a dependency on `walrs_validation`. This is cleaner than
the original plan which kept `Filter<Value>` in `walrs_inputfilter`.

### Step 4: Update `walrs_inputfilter` — remove `filter_enum.rs`, update re-exports

**Delete:** `crates/inputfilter/src/filter_enum.rs` (entire module — all logic has moved)

**Update:** `crates/inputfilter/src/lib.rs`
```rust
// Remove: pub mod filter_enum;
// Remove: pub use filter_enum::Filter;
// Add:    pub use walrs_filter::Filter;
```

This mirrors how `walrs_inputfilter` re-exports `Rule<T>`:
```rust
// Existing pattern (validation):
pub use walrs_validation::rule::{Condition, Rule, RuleResult};

// New pattern (filters):
pub use walrs_filter::Filter;
```

**Update:** `crates/inputfilter/src/field.rs`
```rust
// Remove: use crate::filter_enum::Filter;
// Add:    use walrs_filter::Filter;
```

**Note:** `crates/inputfilter/src/filters/mod.rs` already does `pub use walrs_filter::*;`, so
`Filter<T>` will automatically be available through that path as well.

### Step 5: Update doc examples and test references

**Files with `crate::filter_enum::Filter` or `walrs_inputfilter::filter_enum::Filter` references:**

- `crates/inputfilter/src/lib.rs` (lines 15–16) — update doc example import:
  `use walrs_inputfilter::Filter;` (no more `filter_enum::Filter as FilterEnum`)
- `crates/inputfilter/src/field.rs` (line 27) — update doc example:
  `use walrs_inputfilter::Filter;` or `use walrs_filter::Filter;`
- `crates/inputfilter/src/field_filter.rs` (lines 588–589, 608) — update test code:
  `crate::filter_enum::Filter::Trim` → `crate::Filter::Trim` (or `walrs_filter::Filter::Trim`)
- `crates/inputfilter/examples/filters.rs` (line 8) — update:
  `use walrs_inputfilter::filter_enum::Filter;` → `use walrs_inputfilter::Filter;`
- `crates/inputfilter/README.md` — update example imports

**New documentation to add:**
- `crates/filter/README.md` — add `Filter<T>` enum section with examples
- `crates/filter/examples/` — add example showing enum usage alongside filter structs

### Step 6: Move tests

Tests from `crates/inputfilter/src/filter_enum.rs` should be distributed:

| Test | Destination |
|------|-------------|
| `test_trim_string`, `test_lowercase_string`, etc. | `crates/filter/src/filter_impls/string.rs` |
| `test_clamp_i32` | `crates/filter/src/filter_impls/numeric.rs` |
| `test_trim_value`, `test_clamp_value_*`, `test_filter_preserves_non_matching_types` | `crates/filter/src/filter_impls/value.rs` |
| `test_filter_serialization` | `crates/filter/src/filter.rs` |

## File-Level Change Summary

| File | Action |
|------|--------|
| `crates/filter/src/traits.rs` | Rename `Filter` → `FilterFn` |
| `crates/filter/src/slug.rs` | Update trait references |
| `crates/filter/src/strip_tags.rs` | Update trait references |
| `crates/filter/src/xml_entities.rs` | Update trait references |
| `crates/filter/src/lib.rs` | Add `filter` + `filter_impls` modules; update re-exports |
| `crates/filter/src/filter.rs` | **Create** — `Filter<T>` enum definition |
| `crates/filter/src/filter_impls/mod.rs` | **Create** — module registry |
| `crates/filter/src/filter_impls/string.rs` | **Create** — `impl Filter<String>` |
| `crates/filter/src/filter_impls/numeric.rs` | **Create** — `impl_numeric_filter!` macro |
| `crates/filter/src/filter_impls/value.rs` | **Create** — `impl Filter<serde_json::Value>` |
| `crates/filter/Cargo.toml` | Add `serde`, `serde_json` deps |
| `crates/filter/README.md` | Update trait name; add enum docs |
| `crates/filter/examples/basic_filters.rs` | Update `Filter` → `FilterFn` |
| `crates/filter/examples/filter_chain.rs` | Update `Filter` → `FilterFn` |
| `crates/filter/benches/filter_benchmarks.rs` | Update `Filter` → `FilterFn` |
| `crates/inputfilter/src/filter_enum.rs` | **Delete** (logic moved to `walrs_filter`) |
| `crates/inputfilter/src/lib.rs` | Remove `filter_enum` module; add `pub use walrs_filter::Filter;` |
| `crates/inputfilter/src/field.rs` | Update import |
| `crates/inputfilter/src/field_filter.rs` | Update test references |
| `crates/inputfilter/examples/filters.rs` | Update import path |
| `crates/inputfilter/README.md` | Update examples |

## Dependency Graph (after changes)

```
walrs_filter
  ├── dependencies: derive_builder, regex, ammonia, serde, serde_json
  ├── exports: FilterFn (trait), SlugFilter, StripTagsFilter, XmlEntitiesFilter, Filter<T> (enum)
  └── NO dependency on walrs_validation
       ↑
walrs_inputfilter
  ├── dependencies: walrs_filter, walrs_validation, serde, serde_json, ...
  ├── re-exports: Filter<T> (from walrs_filter), Rule<T> (from walrs_validation)
  └── owns: Field<T>, FieldFilter, FormViolations
       ↑
walrs_form
  ├── dependencies: walrs_inputfilter, walrs_validation
  └── owns: Form, re-exports Field, FieldFilter, etc.
```

No circular dependencies. `walrs_filter` uses `serde_json::Value` directly, avoiding any
dependency on `walrs_validation`.

## Future Considerations

1. **Filter struct consolidation**: Unlike the validation side (where legacy validator structs
   were removed), the filter structs (`SlugFilter`, `StripTagsFilter`, `XmlEntitiesFilter`)
   should be **kept** — they hold configuration and provide the actual transformation logic
   that the enum delegates to. The `FilterFn` trait remains useful as their shared interface.

2. **Feature-gated serde**: Could gate serde support behind a `serde` feature flag in
   `walrs_filter`, but `walrs_validation` doesn't do this, so consistency favors always-on.

3. **`FilterFn` naming alternatives**: `ApplyFilter`, `FilterTransform` are alternatives
   if `FilterFn` feels too close to Rust's `Fn`/`FnMut`/`FnOnce` traits. However, the
   `fn_traits` feature in `walrs_filter` already implements `FnOnce`/`FnMut`/`Fn` for the
   filter structs, so `FilterFn` actually aligns well with that convention.

4. **New filter variants**: Once the enum lives in `walrs_filter`, adding new filter types
   (e.g., `Abs`, `Round { precision }`, `Replace { from, to }`) becomes straightforward —
   add the variant, add the impl in the appropriate `filter_impls/` file.


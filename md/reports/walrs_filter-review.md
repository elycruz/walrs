# walrs_filter — Review Findings

**Files Reviewed:** 15
**Test Results:** 135 unit tests passed, 12 doc-tests passed (0 failures)
**Clippy Warnings:** 0
**Doc Warnings:** 0

## Severity Summary

| Severity | Count |
|---|---|
| 🔴 Critical | 0 |
| 🟠 High | 1 |
| 🟡 Medium | 4 |
| 🔵 Low | 5 |
| ✅ Clean | 9 files |

## Findings

### 1. README.md:341 — License mismatch with Cargo.toml

**Severity:** 🟠 High

README states:
```
## License

MIT & Apache-2.0
```

But `Cargo.toml` declares:
```toml
license = "Elastic-2.0"
```

**Impact:** Users relying on README for licensing information may mistakenly believe the crate is MIT/Apache-2.0 dual-licensed when it is actually Elastic-2.0. This has legal implications.

**Suggested fix:** Update README.md line 341 to match Cargo.toml:
```markdown
## License

Elastic-2.0
```

---

### 2. strip_tags.rs:95-99 — Fast-path always allocates despite no-op

**Severity:** 🟡 Medium

```rust
fn filter(&self, input: Cow<'_, str>) -> Self::Output {
    // Fast path: if no '<' present, no HTML tags to process
    if !input.contains('<') {
      return Cow::Owned(input.into_owned()); // Always allocates!
    }
```

The output type is `Cow<'static, str>`, so the borrowed input (with a shorter lifetime) cannot be returned as-is. This means every no-op invocation still allocates.

**Impact:** Performance — every call without HTML tags incurs an allocation, negating the purpose of the fast-path for `Cow::Borrowed` inputs.

**Suggested fix:** This is an inherent limitation of the `Output = Cow<'static, str>` type. Consider changing the output type to tie the lifetime to the input:
```rust
impl<'a> Filter<Cow<'a, str>> for StripTagsFilter<'_> {
    type Output = Cow<'a, str>;
    // ...
}
```
However, this requires Ammonia's output to be wrapped differently. A simpler fix: accept `Cow::Owned` input without re-allocating if already owned:
```rust
if !input.contains('<') {
    return match input {
        Cow::Owned(s) => Cow::Owned(s),
        Cow::Borrowed(s) => Cow::Owned(s.to_string()),
    };
}
```
Wait — that's what `into_owned` already does. The real fix is the lifetime change, which would be a breaking API change. Document this as a known limitation for now.

---

### 3. slug.rs:12-18 — `get_slug_filter_regex` and `get_dash_filter_regex` are public API

**Severity:** 🟡 Medium

```rust
pub fn get_slug_filter_regex() -> &'static Regex { ... }
pub fn get_dash_filter_regex() -> &'static Regex { ... }
```

These expose internal implementation details (compiled regexes) that users should not depend on.

**Impact:** Public API surface includes implementation details, creating semver obligations for internal regex patterns.

**Suggested fix:** Change to `pub(crate)` or remove `pub` entirely — these are only used within this module.

---

### 4. filter_op.rs:286-303 — Chain `apply_ref` always returns `Cow::Owned` for 2+ filters

**Severity:** 🟡 Medium

```rust
FilterOp::Chain(filters) => {
    let flat = flatten_chain(filters);
    // ...
    let mut result = first_result.into_owned(); // Always allocates for 2+ filters
    for f in &flat[1..] {
        match f.apply_ref(&result) {
            Cow::Borrowed(_) => {} // No change, keep result as-is
            Cow::Owned(s) => result = s,
        }
    }
    Cow::Owned(result) // Always Owned
}
```

Even if all filters in the chain are no-ops, `into_owned()` forces an allocation on the first result, and the final return is always `Cow::Owned`.

**Impact:** Zero-copy optimization is lost for chains where all operations are no-ops.

**Suggested fix:** Track whether any filter mutated and return `Cow::Borrowed(value)` if none did:
```rust
let flat = flatten_chain(filters);
if flat.is_empty() {
    return Cow::Borrowed(value);
}
let first = flat[0].apply_ref(value);
if flat.len() == 1 {
    return first;
}
let mut any_owned = matches!(&first, Cow::Owned(_));
let mut result = first.into_owned();
for f in &flat[1..] {
    match f.apply_ref(&result) {
        Cow::Borrowed(_) => {}
        Cow::Owned(s) => { result = s; any_owned = true; }
    }
}
if any_owned {
    Cow::Owned(result)
} else {
    Cow::Borrowed(value)
}
```

---

### 5. xml_entities.rs:34-36 — `XmlEntitiesFilter` missing `Clone`, `Debug` derives

**Severity:** 🟡 Medium

```rust
#[must_use]
pub struct XmlEntitiesFilter<'a> {
    pub chars_assoc_map: &'a HashMap<char, &'static str>,
}
```

The struct holds an immutable reference, which is `Clone + Debug`, but the struct doesn't derive either.

**Impact:** Users cannot clone or debug-print `XmlEntitiesFilter`, limiting composability.

**Suggested fix:** Add derives:
```rust
#[must_use]
#[derive(Clone, Debug)]
pub struct XmlEntitiesFilter<'a> { ... }
```

---

### 6. filter_error.rs:26 — `FilterError` missing `Eq` derive

**Severity:** 🔵 Low

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct FilterError { ... }
```

All fields are `String` and `Option<String>`, both of which implement `Eq`.

**Impact:** Minor — prevents use in contexts requiring `Eq` (e.g., `HashMap` keys, some generic bounds).

**Suggested fix:** Add `Eq`:
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
```

---

### 7. slug.rs:7 — Redundant `(?i)` flag in slug regex

**Severity:** 🔵 Low

```rust
static SLUG_FILTER_REGEX_STR: &str = r"(?i)[^a-zA-Z0-9_\-]";
```

The character class `[^a-zA-Z0-9_\-]` already explicitly lists both lower and uppercase ranges. The `(?i)` case-insensitive flag is redundant.

**Impact:** No functional impact — purely cosmetic.

**Suggested fix:** Remove the flag:
```rust
static SLUG_FILTER_REGEX_STR: &str = r"[^a-zA-Z0-9_\-]";
```

---

### 8. Cargo.toml:22 — `derive_builder` dependency used only for `SlugFilter`

**Severity:** 🔵 Low

```toml
derive_builder = "0.13.0"
```

`derive_builder` is a proc-macro dependency that adds compile time, used only for `SlugFilterBuilder`. The struct has just 2 fields.

**Impact:** Increased compile time for minimal benefit.

**Suggested fix:** Consider replacing with a manual builder or removing if the builder pattern isn't essential for `SlugFilter`.

---

### 9. Cargo.toml:23 — `regex` dependency used only for slug

**Severity:** 🔵 Low

The `regex` crate is used only in `slug.rs` for two simple character-class patterns. These could be implemented with `char::is_ascii_alphanumeric()` and iterators, removing a heavy dependency.

**Impact:** Increased compile time and binary size for patterns achievable with std.

**Suggested fix:** Low priority — the current approach using `OnceLock` and compiled regex is performant at runtime. Consider replacing only if dependency reduction is a goal.

---

### 10. strip_tags.rs:35 — `StripTagsFilter` missing `Clone`, `Debug`

**Severity:** 🔵 Low

```rust
pub struct StripTagsFilter<'a> {
    pub ammonia: Option<ammonia::Builder<'a>>,
}
```

`ammonia::Builder` does not implement `Clone` or `Debug`, so these cannot be derived. This is an upstream limitation.

**Impact:** Users cannot clone or debug-print `StripTagsFilter`. This is a known limitation of wrapping `ammonia::Builder`.

**Suggested fix:** No fix possible without upstream changes. Document this limitation.

---

## ✅ Clean Files

- `src/lib.rs` — Well-structured module declarations, correct re-exports, good crate-level docs
- `src/traits.rs` — Clean trait definitions with good doc examples
- `src/filter_error.rs` — Well-designed error type with good test coverage (minor `Eq` note above)
- `src/filter_op.rs` — Comprehensive implementation with thorough tests (minor chain perf note above)
- `src/try_filter_op.rs` — Correct fallible filter implementation, good flatten_try_chain for deep nesting safety
- `benches/filter_benchmarks.rs` — Thorough benchmarks covering all filter types and edge cases
- `examples/basic_filters.rs` — Clean, demonstrative
- `examples/filter_chain.rs` — Clean, demonstrative
- `examples/filter_op_usage.rs` — Comprehensive examples covering all FilterOp variants
- `examples/try_filters.rs` — Good fallible filter examples

## Test Coverage Gaps

The test coverage is generally thorough. Minor gaps:

1. **`StripTagsFilter` with custom `ammonia::Builder`** — only tested in one threaded test. Could use more edge cases (e.g., allowed tags list, attribute filtering).
2. **`FilterOp::Truncate` with `max_length: 0`** — not explicitly tested (should return empty string).
3. **`FilterOp::Slug` with `max_length: Some(0)`** — not explicitly tested.
4. **`FilterOp<Value>::Custom`** — not tested in the Value context.
5. **`TryFilterOp` numeric types beyond `i32`/`i64`** — `f32`, `f64`, `u32`, `u64`, `usize` not tested with TryFilterOp.
6. **Empty string edge cases** — `to_slug(Cow::Borrowed(""))` behavior not tested for `FilterOp::Slug`.
7. **`FilterOp::Replace` with `from == to`** — not tested (should be effectively a no-op but still allocates).

## Recommendations (Priority Order)

1. **Fix README license** (Finding #1) — Legal correctness issue, immediate fix needed
2. **Make regex helper functions `pub(crate)`** (Finding #3) — API hygiene, prevents semver lock-in
3. **Add `Clone`/`Debug` to `XmlEntitiesFilter`** (Finding #5) — Easy win for API completeness
4. **Add `Eq` to `FilterError`** (Finding #6) — Trivial, improves ergonomics
5. **Optimize Chain `apply_ref` for all-noop case** (Finding #4) — Performance improvement
6. **Remove redundant `(?i)` regex flag** (Finding #7) — Trivial cleanup
7. **Consider removing `derive_builder`** (Finding #8) — Compile-time improvement, lower priority
8. **Document `StripTagsFilter` output lifetime limitation** (Finding #2) — Clarifies known behavior

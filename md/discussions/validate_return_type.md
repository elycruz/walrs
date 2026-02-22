# `Result<(), Violation>` vs `Option<Violation>` for `validate*` methods

## Summary

**`Option<Violation>` is the better fit for our use cases.**

## Semantics

- `Result<(), Violation>` implies an *operation* that can fail — the `Err` variant signals something went wrong unexpectedly.
- `Option<Violation>` implies a *query* — "is there a violation?" — which is exactly what validation does. A violation isn't an error in the program; it's an expected, normal outcome of checking input.

## Alignment with plans

Our `random.md` shows renaming `validate*` to `check_validity*`/`check*`. The "check" framing reinforces that this is a query, not a fallible operation. Returning `Option<Violation>` aligns naturally:

```rust
// Reads as: "check validity, and optionally get back a violation"
fn check_validity(&self, value: &str) -> Option<Violation>
```

## Ergonomics for aggregation

The planned verbose format collects violations into `HashMap<String, ConstraintViolation>`. Aggregating from `Option<Violation>` is cleaner:

```rust
// With Option<Violation> — natural with filter_map
let violations: HashMap<String, Violation> = elements
    .iter()
    .filter_map(|el| {
        el.check_validity(value).map(|v| (el.name().to_string(), v))
    })
    .collect();
```

With `Result<(), Violation>`, you'd need `.err()` calls everywhere, which is awkward and suggests misuse of the type.

## When `Result` *does* make sense

Keep `Result` for the `build()` methods on `Form::builder()` and `InputElement::builder()` — those are fallible operations where misconfiguration is a genuine error.

## Return type recommendations

| Method                    | Return type                                        | Rationale                        |
|---------------------------|----------------------------------------------------|----------------------------------|
| `check_validity`          | `Option<Violation>`                                | Query — violation is expected output |
| `check_validity_verbose`  | `Option<HashMap<String, ConstraintViolation>>`     | Aggregated query                 |
| `builder().build()`       | `Result<T, BuildError>`                            | Fallible construction            |


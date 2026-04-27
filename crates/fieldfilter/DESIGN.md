# `walrs_fieldfilter` Design

## Purpose

Simplify HTML form handling on the backend by combining serializable validation
rules (`Rule<T>` from `walrs_validation`) and filter operations (`FilterOp<T>`
from `walrs_filter`) into per-field and multi-field pipelines.

## Inspiration

- [laminas-fieldfilter](https://docs.laminas.dev/laminas-fieldfilter/)
- [fjl-validator](https://github.com/functional-jslib/fjl/tree/monorepo/packages/fjl-validator)
- [fjl-fieldfilter](https://github.com/functional-jslib/fjl/tree/monorepo/packages/fjl-fieldfilter)

Unlike laminas-fieldfilter, Rust web frameworks (e.g., actix-web) handle type
coercion automatically, so this crate focuses on value-level filtering and
validation rather than type conversion.

## Architecture

```
walrs_validation → walrs_filter      → walrs_fieldfilter ← walrs_fieldset_derive
(Rule<T>)          (Filter trait,      (Field<T>,           (#[derive(Fieldset)])
                    FilterOp<T>,        Fieldset trait)
                    TryFilterOp<T>)
```

### Core Types

- **`Field<T>`** — Single-field configuration combining optional `FilterOp<T>`
  filters, optional `TryFilterOp<T>` fallible filters, and an optional
  `Rule<T>` validation rule. Built via `FieldBuilder<T>` (derive_builder).

- **`Fieldset`** — Typed multi-field pipeline trait. Implemented by hand or
  via `#[derive(Fieldset)]` (the `derive` feature). Provides `filter()`,
  `validate()`, and `sanitize()` over a struct's named fields with
  compile-time-checked field names and types.

- **`FieldsetAsync`** — Async variant of `Fieldset` (behind the `async`
  feature). Same API surface returning `Future`s.

- **`FieldsetViolations`** — Aggregate error container mapping field names to
  `Violations`. Form-level violations use the empty-string key.

### Processing Pipeline

`Fieldset::sanitize(self)` runs:

1. **`filter(self)`** — applies each field's `FilterOp` filters in order.
2. **`validate(&self)`** — runs each field's `Rule<T>` plus any
   `#[cross_validate(...)]` rules; collects errors into
   `FieldsetViolations`.

### Design Decisions

- **`IndexMap` for deterministic ordering** — `FieldsetViolations` uses
  `IndexMap` so iteration follows insertion order, making validation error
  output predictable.

- **Serializable rules and filters** — Both `Rule<T>` and `FilterOp<T>`
  derive `Serialize`/`Deserialize`, enabling server-to-client rule
  transport for client-side pre-validation.

- **Stateless** — `Field<T>` is a configuration object that does not
  mutate internal state during processing.

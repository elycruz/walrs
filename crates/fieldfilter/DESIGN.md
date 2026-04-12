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
walrs_validation → walrs_filter      → walrs_fieldfilter → walrs_form
(Rule<T>, Value)   (Filter trait,      (Field<T>,          (Form, Elements,
                    FilterOp<T>,        FieldFilter,         FormData)
                    TryFilterOp<T>)     CrossFieldRule)
```

### Core Types

- **`Field<T>`** — Single-field configuration combining optional `FilterOp<T>`
  filters, optional `TryFilterOp<T>` fallible filters, and an optional
  `Rule<T>` validation rule. Built via `FieldBuilder<T>` (derive_builder).
  Specialised impls exist for `T = String` and `T = Value`.

- **`FieldFilter`** — Multi-field form-level pipeline. Holds an
  `IndexMap<String, Field<Value>>` plus a list of `CrossFieldRule`s.
  Provides `filter()`, `try_filter()`, `validate()`, and `process()`.

- **`CrossFieldRule` / `CrossFieldRuleType`** — Serializable cross-field
  validation (FieldsEqual, RequiredIf, RequiredUnless, OneOfRequired,
  MutuallyExclusive, DependentRequired) plus a non-serializable `Custom`
  variant for arbitrary logic.

- **`FormViolations`** — Aggregate error container with per-field
  `Violations` and form-level `Violation` lists.

### Processing Pipeline

`FieldFilter.process(data)` runs:

1. **`filter(data)`** — applies each field's `FilterOp` filters in order.
2. **`try_filter(data)`** — applies each field's `TryFilterOp` filters;
   collects errors into `FormViolations`.
3. **`validate(&data)`** — runs each field's `Rule<T>` plus all
   `CrossFieldRule`s; collects errors into `FormViolations`.

### Design Decisions

- **`Value` for dynamic form data** — `FieldFilter.fields` uses
  `Field<Value>` so that heterogeneous form payloads (strings, numbers,
  booleans, arrays) can be handled with a single `IndexMap<String, Value>`.

- **`IndexMap` for deterministic ordering** — field iteration follows
  insertion order, making validation error output predictable.

- **Serializable rules and filters** — Both `Rule<T>` and `FilterOp<T>`
  derive `Serialize`/`Deserialize`, enabling server-to-client rule
  transport for client-side pre-validation.

- **Stateless** — `Field<T>` and `FieldFilter` are configuration objects
  that do not mutate internal state during processing.
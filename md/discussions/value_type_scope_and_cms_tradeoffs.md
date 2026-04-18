# Discussion: `Value` Scope, Dropping `Value`, and the CMS Use Case

**Date:** 2026-04-18
**Related:** [`value_enum_impedance_mismatch.md`](./value_enum_impedance_mismatch.md),
[`value_feature_gating_and_isolation.md`](./value_feature_gating_and_isolation.md),
[`FORM_ECOSYSTEM_COMPARISON.md`](../comparisons/FORM_ECOSYSTEM_COMPARISON.md)

---

## Purpose

Record of an architectural Q&A between the maintainer and Claude exploring
three progressively narrowing questions:

1. What would the form ecosystem lose if the `Value` type were dropped?
2. What if WASM/config-driven functionality were *also* dropped as goals?
3. For a CMS use case (user-defined schemas, runtime-unknown types), could
   we still ship something optimal without `Value` / `serde_json::Value`?

The discussion is a scoping exercise, not a proposal. The companion
document [`value_feature_gating_and_isolation.md`](./value_feature_gating_and_isolation.md)
turns the conclusions into concrete options.

---

## Q1 — What does dropping `Value` cost?

### What `Value` uniquely enables

`Value` (`crates/validation/src/value.rs`) is the runtime-typed enum
(`Null`/`Bool`/`I64`/`U64`/`F64`/`Str`/`Array`/`Object`) that lets the
ecosystem erase types at the form/field boundary. Everything else —
`Rule<T>`, `FilterOp<T>`, `Field<T>` — is already generic. Dropping
`Value` really means dropping **runtime-typed heterogeneous form data**,
not validation or filtering themselves.

Four capabilities are load-bearing on it:

1. **Heterogeneous storage** — `FormData = IndexMap<String, Value>` and
   `FieldFilter::fields: IndexMap<String, Field<Value>>` hold a whole
   form in one map. Without `Value`, you either (a) generate a bespoke
   enum per form via derive, or (b) use
   `IndexMap<String, Box<dyn FieldOps>>` with dynamic dispatch.
2. **Config-driven rules/filters** — `Rule<Value>` and `FilterOp<Value>`
   are serde-serializable, so pipelines can come from JSON/YAML.
   `Rule<T>` is serializable too, but only over a single concrete `T`;
   you can't load a mixed pipeline unless the leaves speak a common
   runtime type.
3. **Cross-field rules on mixed types** — `FieldsEqual`, `RequiredIf`,
   `DependentRequired` compare two cells of the same map. Without
   `Value` the comparison needs either `serde_json::Value` or per-pair
   monomorphization.
4. **WASM/JSON boundary** — `From<serde_json::Value>` + `value!` macro
   give the `walrs_form` WASM story its pivot point. The bridge still
   works against `serde_json::Value`, but you lose the `I64`/`U64`/`F64`
   discrimination `Value` adds deliberately.

### How each crate changes if `Value` is removed

- **walrs_validation** — minimal change. `Rule<T>` is already the
  primary surface; delete `value.rs`, the `Rule<Value>` impls, and
  `ValueExt`. Loses the "serializable rule pipeline over arbitrary
  input" bullet in the comparison doc — rules remain serializable, but
  only pinned to a concrete `T`.
- **walrs_filter** — same story. `FilterOp<T>` / `TryFilterOp<T>`
  survive; the `FilterOp<Value>` / `TryFilterOp<Value>` impls go.
- **walrs_fieldfilter** — hit hardest. `FieldFilter` today is
  monomorphic over `Value`. Replacement options:
  - `FieldFilter<T>` — forces every field to the same `T`. Kills
    heterogeneous forms.
  - `FieldFilter` holding `IndexMap<String, Box<dyn FieldOps>>` —
    trait objects with erased input. Works, but cross-field rules get
    awkward and you lose `Serialize`/`Deserialize` on the whole filter.
  - Generate a form-specific struct (enum per form) via derive. Loses
    config-driven forms.
- **walrs_form** — `FormData` stops being an interchange format.
  Either it becomes `serde_json::Value`-backed (regression in numeric
  fidelity) or each form ships its own typed data struct (regression in
  dynamic form loading from config). The WASM side keeps working via
  `serde_json`.
- **walrs_fieldset_derive** — `gen_form_data.rs` currently emits
  `Value::I64(...)` etc. to build `FormData`. Without `Value` the macro
  either emits `serde_json::Value`, emits a per-struct typed mirror, or
  stops generating `into_form_data` entirely. Derive-time validation /
  filtering is unaffected.

### The tradeoff

Keep ~90% of the design (typed rules, filter pipelines, derive,
cross-field rules *within a known type*) and lose the two
differentiators the comparison doc actually leans on: config-loaded
pipelines over mixed fields, and a single serializable `FormData` usable
on both sides of the wire. If you're willing to fall back to
`serde_json::Value` at that boundary (accepting `Number` ambiguity),
most of the functionality survives with one less custom type.

---

## Q2 — What if WASM / config-driven goals are *also* dropped?

If WASM interchange and config-driven pipelines come off the table,
neither `Value` nor `serde_json::Value` is needed anywhere in the public
API. Everything else is already generic over `T`.

**Each crate becomes:**

- **walrs_validation** — drop `value.rs` and the `Rule<Value>` /
  `Condition<Value>` impls. `Rule<T>` + combinators + `ViolationType` +
  `ValidateRef` survive unchanged. Rules are still `Serialize`/
  `Deserialize` per concrete `T`.
- **walrs_filter** — drop `FilterOp<Value>` / `TryFilterOp<Value>`.
  `FilterOp<T>` / `TryFilterOp<T>` remain primary. No capability lost.
- **walrs_fieldfilter** — this crate's reason to exist shrinks.
  Options:
  - Fold into derive output: `#[derive(Fieldset)]` generates a typed
    `validate(&self)` directly, no runtime map. Cross-field rules
    become generated code or typed closures.
  - Keep `FieldFilter<T>` as a typed multi-field helper for homogeneous
    collections (less compelling).
  - Cross-field rule *library* (`FieldsEqual`, `RequiredIf`,
    `MutuallyExclusive`, …) remains valuable, expressed over typed
    field accessors rather than `Value` cells.
- **walrs_form** — largely loses its reason to exist. Without the
  WASM/JSON pivot, a "serializable HTML form model" duplicates
  `askama`/`leptos`/`dioxus`/framework extractors.
- **walrs_fieldset_derive** — simplifies. `gen_form_data.rs` goes away.
  Ends up looking closer to `garde`'s output, but richer.

### Ecosystem positioning change

The comparison doc's two headline differentiators ("serializable rules
over mixed fields" and "WASM-friendly form model") both weaken.
Remaining differentiators vs. `garde`:

- Combinator API (`.and()` / `.or()` / `.not()` / `.when()` /
  `.when_else()`).
- `ValidateRef<T: ?Sized>` for zero-copy `&str` validation.
- Composable filter pipeline as a first-class concern (not
  deserialization-bound).
- Unified validation + filter + cross-field in a single derive.
- Typed `ViolationType` enum rather than string codes.

A narrower but still defensible niche — "`garde` with combinators,
filters, and typed violations" — rather than the broader
"config-driven/WASM form platform."

---

## Q3 — The CMS case: user-defined schemas + CRUD forms

### Short answer

For a CMS where the user defines the schema at runtime, you cannot ship
an *optimal* solution without some runtime-typed value — and walrs's
`Value` is better suited here than `serde_json::Value`. This is
precisely the use case it exists to serve.

### Why fully-typed won't work

In a CMS the field shape is unknown at compile time. There is no
`struct Post` — the user created "Post" yesterday via the admin UI. The
only ways around that are:

- **Codegen + recompile** on every schema change — unacceptable for a
  live CMS.
- **EAV-style string-only storage** — crude, loses native types
  (dates, numbers, refs).
- **Per-field trait objects** (`Box<dyn FieldValidator>`) — works for
  validation, but the HTTP boundary still hands you JSON, and
  cross-field rules still need a common runtime representation. Ends
  up reinventing `Field<Value>` with more indirection and no
  serialization.

So *something* has to be a runtime-typed bag of values. The choice is
which one.

### Sketch of a CMS pipeline on current walrs APIs

```rust
struct FieldDef {
    name: String,
    kind: FieldKind,                 // Text, Int, Date, Ref(type_id), ...
    rules: Vec<Rule<Value>>,         // serde-deserialized from DB
    filters: Vec<FilterOp<Value>>,
}
struct TypeDef { name: String, fields: Vec<FieldDef> }
```

- **Storage:** JSONB column per row (`data: Value`), or narrow EAV.
- **Create/Update:** HTTP body → `Value` (via existing `From<serde_json::Value>`),
  look up `TypeDef`, build an ad-hoc `FieldFilter` from the stored
  `FieldDef`s, run filters → rules → cross-field rules, persist.
- **Authoring UX:** admin UI rule builder → JSON → `Rule<Value>` via
  serde; or TOML/YAML schema files; or a small DSL parsed to
  `Rule<Value>`. All three converge on "serialize a `Rule<Value>`
  pipeline; apply to a `Value` input."

### Why `walrs::Value` beats `serde_json::Value` for this

1. **Numeric discrimination.** `serde_json::Value::Number` blurs
   `i64`/`u64`/`f64`. CMS fields *do* distinguish them ("positive
   integer", "currency", "percentage") and rules like `Step` or `Max`
   need to know.
2. **Trait impls already exist.** `Rule<Value>` and `FilterOp<Value>`
   are implemented. `Rule<serde_json::Value>` isn't, and wouldn't be
   trivial because of the numeric ambiguity.
3. **Existing bridge.** The `From<serde_json::Value>` impl behind a
   feature flag (`serde_json_bridge`) means the HTTP boundary is trivial
   — accept JSON, convert once at the edge, work in `Value` internally.

### Recommendation from the CMS angle

For a CMS product, the earlier question flips: `Value` is the core of
what makes this use case tractable. Drop WASM and config-file forms as
narratives if you want a tighter scope, but keep `Value` — and consider
making "runtime-typed CMS forms" the flagship story in the comparison
doc. It's a genuine niche that `validator`/`garde` cannot address.

---

## Outcome

`Value` is load-bearing for *any* runtime-schema workload (CMS,
admin-authored forms, config-driven validation, WASM interchange). The
follow-on question becomes: **how do we isolate `Value` so typed-only
consumers don't pay its compile cost, without deleting it?** That's the
subject of [`value_feature_gating_and_isolation.md`](./value_feature_gating_and_isolation.md).

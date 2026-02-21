# `Rule<T>` — Dependencies

> **Crate:** `walrs_validation` · **File:** `crates/validation/src/rule.rs`
>
> Additional impl blocks in `crates/validation/src/impls/{string,steppable,scalar,length,attributes}.rs`

---

## External Crate Dependencies

| Crate | Usage |
|---|---|
| **`serde`** (`Serialize`, `Deserialize`) | Derive on `Rule<T>`, `Condition<T>`; used in `ToAttributesList` impl |
| **`serde_json`** | `ToAttributesList` impl (converts attributes to `serde_json::Value`) |
| **`regex`** | `Condition<String>::evaluate_str`, `Rule<String>::validate_str` (pattern/email/URL matching) |
| **`thiserror`** | Transitive — used by `Violation` |

## Standard Library Dependencies

| Item | Usage |
|---|---|
| `std::fmt::{self, Debug, Display}` | `Debug` impls for `Rule<T>`, `Condition<T>`, `CompiledRule<T>` |
| `std::sync::Arc` | `Custom` variants in both `Rule<T>` and `Condition<T>`; `Message::Provider` |
| `std::sync::OnceLock` | `CompiledRule<T>` — lazy cache for compiled validators |

## Crate-Internal Type Dependencies

| Type / Trait | Defined In | Used By |
|---|---|---|
| `Violation` / `ViolationType` | `violation.rs` | All validation methods (`validate_str`, `validate_step`, `validate_scalar`, `validate_len`) |
| `Violations` (`Vec<Violation>`) | `violation.rs` | `validate_all_*` methods in scalar/string impls |
| `Message<T>` / `MessageContext<T>` / `MessageParams` | `message.rs` | `WithMessage` variant, locale-aware error resolution |
| `IsEmpty` (trait) | `traits.rs` | `Condition::evaluate`, bounds on `Rule<T>::validate_step`, `Rule<T>::validate_scalar`, `Rule<T>::compile` |
| `SteppableValue` (trait) | `traits.rs` | Bound on `Rule<T>::validate_step`, `Rule<T>::compile` |
| `ScalarValue` (trait) | `traits.rs` | Bound on `Rule<T>::validate_scalar` |
| `WithLength` (trait) | `traits.rs` | Bound on `Rule<T>::validate_len` |
| `InputValue` (trait) | `traits.rs` | Supertrait of `ScalarValue` |
| `NumberValue` (trait) | `traits.rs` | Supertrait of `SteppableValue` |
| `Validate` (trait) | `traits.rs` | Implemented by `Rule<T>` for steppable/scalar types, `CompiledRule<T>` |
| `ValidateRef` (trait) | `traits.rs` | Implemented by `CompiledRule<String>` |
| `ToAttributesList` (trait) | `traits.rs` | Implemented for `Rule<T: Serialize>` |
| `CachedStringValidators` | `impls/string.rs` | `CompiledRule<T>` field (cached regex patterns) |

## Trait Bounds on `T` (by impl block)

| Impl / Method | Bounds on `T` |
|---|---|
| `Rule<T>` struct (derive) | `Clone + Serialize + Deserialize` |
| `Debug for Rule<T>` | `Debug` |
| `PartialEq for Rule<T>` | `PartialEq` |
| `Rule<T>` combinators (`and`, `or`, `not`, `when`, etc.) | *(none — unconstrained)* |
| `Rule<String>::validate_str` | `T = String` (concrete) |
| `Rule<String>::compile` | `T = String` (concrete) |
| `Rule<T>::validate_step` | `SteppableValue + IsEmpty` |
| `Rule<T>::validate_scalar` | `ScalarValue + IsEmpty` |
| `Rule<T>::validate_len` | `WithLength` |
| `Rule<T>::compile` (generic) | `SteppableValue + IsEmpty + Clone` |
| `Validate<T> for Rule<T>` | `SteppableValue + IsEmpty` |
| `ToAttributesList for Rule<T>` | `Serialize` |
| `Condition<T>::evaluate` | `PartialEq + PartialOrd + IsEmpty` |

## Trait Hierarchy

```
InputValue: Copy + Default + PartialEq + PartialOrd + Display + Serialize
  └── ScalarValue: InputValue
        └── NumberValue: ScalarValue + Add + Sub + Mul + Div
              └── SteppableValue: NumberValue + Rem<Output = Self>
```


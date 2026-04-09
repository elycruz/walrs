# `with_message` API Unification

**Date:** 2026-04-09
**Branch:** `copilot/update-with-message-methods` (abandoned)

## Context

The abandoned branch attempted to unify `with_message()` and `with_message_provider()` into a single `with_message()` method via a new `IntoMessage<T>` trait.

## Current API (main)

Two separate methods on `Rule<T>`:

```rust
// Static message
pub fn with_message(self, msg: impl Into<String>) -> Rule<T>

// Dynamic (closure-based) message with optional locale
pub fn with_message_provider<F>(self, f: F, locale: Option<&str>) -> Rule<T>
```

Usage:

```rust
Rule::<String>::MinLength(8)
    .with_message("Password too short.");

Rule::<i32>::Min(0)
    .with_message_provider(|ctx| format!("Got {}, expected >= 0.", ctx.value), None);
```

## Proposed API (branch)

A single method powered by an `IntoMessage<T>` trait:

```rust
pub trait IntoMessage<T: ?Sized> {
    fn into_message(self) -> Message<T>;
}

// Impls for &str, String → Message::Static
// Impl for Message<T> → passthrough

pub fn with_message(self, msg: impl IntoMessage<T>) -> Rule<T>
```

Usage:

```rust
Rule::<String>::MinLength(8)
    .with_message("Password too short.");

Rule::<i32>::Min(0)
    .with_message(Message::provider(|ctx| {
        format!("Got {}, expected >= 0.", ctx.value)
    }));
```

Locale is set separately via `.with_locale("es")` instead of being bundled into the provider call.

## Trade-offs

### Pros of the branch approach
- Single unified method — idiomatic Rust `Into`-style pattern
- Removes the awkward `None` second arg on every `with_message_provider` call
- `Message::provider(...)` is explicit about what's being constructed
- Locale decoupled from provider — arguably cleaner separation of concerns

### Cons of the branch approach
- Slightly more verbose for closures: `Message::provider(|ctx| ...)` vs `with_message_provider(|ctx| ...)`
- Requires `.with_locale()` chain when locale was previously bundled in

## Verdict

The branch's `IntoMessage` approach is cleaner API design. The current main API works fine but has `with_message_provider(closure, None)` noise in every call site. Worth revisiting when doing an API cleanup pass.

## Files affected (from branch diff)

- `crates/validation/src/message.rs` — `IntoMessage` trait + impls
- `crates/validation/src/rule.rs` — unified `with_message`, removed `with_message_provider`
- `crates/inputfilter/src/rule.rs` — re-export `IntoMessage`
- `crates/form/examples/localized_form.rs` — updated call sites
- `crates/inputfilter/examples/localized_messages.rs` — updated call sites

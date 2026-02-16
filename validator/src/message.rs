//! # Message Types for Validator Error Messages
//!
//! This module provides the `Message<T>` enum and supporting types for flexible,
//! serializable validation error messages. Messages can be either static strings
//! or dynamic providers that generate contextual messages at runtime.
//!
//! ## Example
//!
//! ```rust
//! use walrs_validator::{Message, MessageContext, MessageParams};
//!
//! // Static message
//! let msg: Message<String> = Message::from("Must be at least 8 characters");
//!
//! // Dynamic message with value context
//! let msg: Message<i32> = Message::provider(|ctx| {
//!     format!(
//!         "Value {} is out of range (min: {}, max: {})",
//!         ctx.value,
//!         ctx.params.min.as_deref().unwrap_or("?"),
//!         ctx.params.max.as_deref().unwrap_or("?")
//!     )
//! });
//! ```

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::sync::Arc;

// ============================================================================
// MessageParams
// ============================================================================

/// Parameters extracted from a rule/validator for message interpolation.
///
/// When validation fails, these parameters provide context about
/// the constraint that was violated, enabling dynamic error messages.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MessageParams {
  /// The name/type of the rule (e.g., "MinLength", "Range")
  pub rule_name: &'static str,

  // ---- Presence ----
  /// Whether the value is required (for `Required` rule).
  pub required: bool,

  // ---- Length constraints ----
  /// Minimum length constraint (for `MinLength` rule).
  pub min_length: Option<usize>,
  /// Maximum length constraint (for `MaxLength` rule).
  pub max_length: Option<usize>,
  /// Exact length constraint (for `ExactLength` rule).
  pub exact_length: Option<usize>,

  // ---- Numeric constraints ----
  /// Minimum value constraint (for `Min` or `Range` rules) converted to string.
  pub min: Option<String>,
  /// Maximum value constraint (for `Max` or `Range` rules).
  pub max: Option<String>,
  /// Step value (for `Step` rule).
  pub step: Option<String>,

  // ---- String constraints ----
  /// Pattern string (for `Pattern` rule).
  pub pattern: Option<String>,

  // ---- Comparison constraints ----
  /// Expected value (for `Equals` rule).
  pub expected: Option<String>,
  /// Allowed values (for `OneOf` rule).
  pub one_of: Option<Vec<String>>,
}

impl MessageParams {
  /// Creates empty params with a rule name.
  pub fn new(rule_name: &'static str) -> Self {
    Self {
      rule_name,
      ..Default::default()
    }
  }

  /// Sets the required flag.
  pub fn with_required(mut self, required: bool) -> Self {
    self.required = required;
    self
  }

  /// Sets the minimum length constraint.
  pub fn with_min_length(mut self, len: usize) -> Self {
    self.min_length = Some(len);
    self
  }

  /// Sets the maximum length constraint.
  pub fn with_max_length(mut self, len: usize) -> Self {
    self.max_length = Some(len);
    self
  }

  /// Sets the exact length constraint.
  pub fn with_exact_length(mut self, len: usize) -> Self {
    self.exact_length = Some(len);
    self
  }

  /// Sets the minimum value constraint.
  pub fn with_min(mut self, min: impl ToString) -> Self {
    self.min = Some(min.to_string());
    self
  }

  /// Sets the maximum value constraint.
  pub fn with_max(mut self, max: impl ToString) -> Self {
    self.max = Some(max.to_string());
    self
  }

  /// Sets the step value constraint.
  pub fn with_step(mut self, step: impl ToString) -> Self {
    self.step = Some(step.to_string());
    self
  }

  /// Sets the pattern string.
  pub fn with_pattern(mut self, pattern: impl ToString) -> Self {
    self.pattern = Some(pattern.to_string());
    self
  }

  /// Sets the expected value.
  pub fn with_expected(mut self, expected: impl ToString) -> Self {
    self.expected = Some(expected.to_string());
    self
  }

  /// Sets the allowed values for `OneOf` rule.
  pub fn with_one_of(mut self, values: Vec<String>) -> Self {
    self.one_of = Some(values);
    self
  }
}

// ============================================================================
// MessageContext
// ============================================================================

/// Context passed to message providers during validation.
///
/// Contains both the value being validated and parameters from the validator,
/// enabling rich, contextual error messages.
///
/// # Example
///
/// ```rust
/// use walrs_validator::{Message, MessageContext, MessageParams};
///
/// let msg: Message<String> = Message::provider(|ctx| {
///     format!(
///         "Value '{}' must have at least {} characters",
///         ctx.value,
///         ctx.params.min_length.map(|n| n.to_string()).unwrap_or("?".to_string())
///     )
/// });
/// ```
#[derive(Clone, Debug)]
pub struct MessageContext<'a, T: ?Sized> {
  /// The value being validated
  pub value: &'a T,
  /// Parameters extracted from the validator
  pub params: MessageParams,
}

impl<'a, T: ?Sized> MessageContext<'a, T> {
  /// Creates a new message context.
  pub fn new(value: &'a T, params: MessageParams) -> Self {
    Self { value, params }
  }
}

// ============================================================================
// Message Enum
// ============================================================================

/// A validation error message that can be either a static string or a dynamic provider.
///
/// This enum enables:
/// - **Static messages**: Simple strings, serializable to JSON/YAML
/// - **Dynamic messages**: Closures that generate messages with context (value, params)
///
/// # Serialization
///
/// Only `Static` variants serialize. `Provider` is skipped and will deserialize
/// as an empty string, which can be detected and replaced with a fallback message.
///
/// # Example
///
/// ```rust
/// use walrs_validator::Message;
///
/// // Static message
/// let msg: Message<String> = Message::from("Must be at least 8 characters");
///
/// // Dynamic message with value context
/// let msg: Message<i32> = Message::provider(|ctx| format!("Value {} is out of range", ctx.value));
/// ```
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message<T: ?Sized> {
  /// A static message string (serializable)
  Static(String),

  /// A dynamic message provider (not serializable)
  ///
  /// The closure receives a `MessageContext` containing the value being validated
  /// and parameters extracted from the validator, enabling rich interpolated messages.
  #[serde(skip)]
  Provider(Arc<dyn Fn(&MessageContext<T>) -> String + Send + Sync>),
}

// Manual Clone implementation for Message<T> where T: ?Sized
impl<T: ?Sized> Clone for Message<T> {
  fn clone(&self) -> Self {
    match self {
      Message::Static(s) => Message::Static(s.clone()),
      Message::Provider(f) => Message::Provider(Arc::clone(f)),
    }
  }
}

impl<T: ?Sized> Message<T> {
  /// Creates a static message.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::Message;
  ///
  /// let msg: Message<str> = Message::static_msg("Invalid value");
  /// assert_eq!(msg.resolve("test"), "Invalid value");
  /// ```
  pub fn static_msg(msg: impl Into<String>) -> Self {
    Message::Static(msg.into())
  }

  /// Creates a dynamic message provider.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::Message;
  ///
  /// let msg: Message<i32> = Message::provider(|ctx| format!("Got {}, expected positive", ctx.value));
  /// assert_eq!(msg.resolve(&-5), "Got -5, expected positive");
  /// ```
  pub fn provider(f: impl Fn(&MessageContext<T>) -> String + Send + Sync + 'static) -> Self {
    Message::Provider(Arc::new(f))
  }

  /// Resolves the message, using the value for dynamic providers.
  ///
  /// This creates a default `MessageParams` with no rule-specific data.
  /// Use `resolve_with_context` for full context.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::Message;
  ///
  /// let static_msg: Message<str> = Message::from("Error");
  /// assert_eq!(static_msg.resolve("any"), "Error");
  ///
  /// let dynamic_msg: Message<str> = Message::provider(|ctx| format!("Bad: {}", ctx.value));
  /// assert_eq!(dynamic_msg.resolve("input"), "Bad: input");
  /// ```
  pub fn resolve(&self, value: &T) -> String {
    match self {
      Message::Static(s) => s.clone(),
      Message::Provider(f) => {
        let ctx = MessageContext::new(value, MessageParams::default());
        f(&ctx)
      }
    }
  }

  /// Resolves the message with full context including rule parameters.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::{Message, MessageContext, MessageParams};
  ///
  /// let msg: Message<i32> = Message::provider(|ctx| {
  ///     format!(
  ///         "Value must be between {} and {}",
  ///         ctx.params.min.as_deref().unwrap_or("?"),
  ///         ctx.params.max.as_deref().unwrap_or("?")
  ///     )
  /// });
  ///
  /// let params = MessageParams::new("Range")
  ///     .with_min(0)
  ///     .with_max(100);
  /// let ctx = MessageContext::new(&50, params);
  /// assert_eq!(msg.resolve_with_context(&ctx), "Value must be between 0 and 100");
  /// ```
  pub fn resolve_with_context(&self, ctx: &MessageContext<T>) -> String {
    match self {
      Message::Static(s) => s.clone(),
      Message::Provider(f) => f(ctx),
    }
  }

  /// Resolves with a fallback if this is an empty static message.
  ///
  /// Useful for handling deserialized messages where `Provider` becomes
  /// an empty `Static` string.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::Message;
  ///
  /// let empty: Message<str> = Message::Static(String::new());
  /// assert_eq!(empty.resolve_or("x", "default"), "default");
  ///
  /// let filled: Message<str> = Message::from("custom");
  /// assert_eq!(filled.resolve_or("x", "default"), "custom");
  /// ```
  pub fn resolve_or(&self, value: &T, fallback: &str) -> String {
    match self {
      Message::Static(s) if s.is_empty() => fallback.to_string(),
      _ => self.resolve(value),
    }
  }

  /// Returns `true` if this is a static message.
  pub fn is_static(&self) -> bool {
    matches!(self, Message::Static(_))
  }

  /// Returns `true` if this is a provider (closure-based) message.
  pub fn is_provider(&self) -> bool {
    matches!(self, Message::Provider(_))
  }
}

impl<T: ?Sized> Debug for Message<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Static(s) => f.debug_tuple("Static").field(s).finish(),
      Self::Provider(_) => write!(f, "Provider(<fn>)"),
    }
  }
}

impl<T: ?Sized> PartialEq for Message<T> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Static(a), Self::Static(b)) => a == b,
      // Providers are never equal (can't compare closures)
      _ => false,
    }
  }
}

impl<T: ?Sized> Default for Message<T> {
  fn default() -> Self {
    Message::Static(String::new())
  }
}

// Convenience conversions
impl<T: ?Sized> From<&str> for Message<T> {
  fn from(s: &str) -> Self {
    Message::Static(s.to_string())
  }
}

impl<T: ?Sized> From<String> for Message<T> {
  fn from(s: String) -> Self {
    Message::Static(s)
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_message_static() {
    let msg: Message<str> = Message::static_msg("Error message");
    assert!(msg.is_static());
    assert!(!msg.is_provider());
    assert_eq!(msg.resolve("any"), "Error message");
  }

  #[test]
  fn test_message_provider() {
    let msg: Message<i32> = Message::provider(|ctx| format!("Value {} is invalid", ctx.value));
    assert!(msg.is_provider());
    assert!(!msg.is_static());
    assert_eq!(msg.resolve(&42), "Value 42 is invalid");
  }

  #[test]
  fn test_message_resolve_or_with_empty() {
    let empty: Message<str> = Message::Static(String::new());
    assert_eq!(empty.resolve_or("x", "fallback"), "fallback");
  }

  #[test]
  fn test_message_resolve_or_with_value() {
    let msg: Message<str> = Message::from("custom");
    assert_eq!(msg.resolve_or("x", "fallback"), "custom");
  }

  #[test]
  fn test_message_from_str() {
    let msg: Message<i32> = Message::from("test message");
    assert_eq!(msg.resolve(&0), "test message");
  }

  #[test]
  fn test_message_from_string() {
    let msg: Message<i32> = Message::from("owned string".to_string());
    assert_eq!(msg.resolve(&0), "owned string");
  }

  #[test]
  fn test_message_equality() {
    let a: Message<i32> = Message::from("same");
    let b: Message<i32> = Message::from("same");
    let c: Message<i32> = Message::from("different");

    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn test_message_provider_never_equal() {
    let a: Message<i32> = Message::provider(|_| "a".to_string());
    let b: Message<i32> = Message::provider(|_| "a".to_string());

    // Providers are never equal (can't compare closures)
    assert_ne!(a, b);
  }

  #[test]
  fn test_message_debug() {
    let static_msg: Message<i32> = Message::from("test");
    let debug_str = format!("{:?}", static_msg);
    assert!(debug_str.contains("Static"));
    assert!(debug_str.contains("test"));

    let provider_msg: Message<i32> = Message::provider(|_| "x".to_string());
    let debug_str = format!("{:?}", provider_msg);
    assert!(debug_str.contains("Provider"));
  }

  #[test]
  fn test_message_default() {
    let msg: Message<i32> = Message::default();
    assert_eq!(msg, Message::Static(String::new()));
  }

  #[test]
  fn test_message_serialization() {
    let msg: Message<i32> = Message::from("serialized");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("serialized"));

    let deserialized: Message<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(msg, deserialized);
  }

  #[test]
  fn test_message_params_new() {
    let params = MessageParams::new("MinLength");
    assert_eq!(params.rule_name, "MinLength");
    assert!(!params.required);
    assert!(params.min_length.is_none());
  }

  #[test]
  fn test_message_params_builder_pattern() {
    let params = MessageParams::new("Range").with_min(0).with_max(100);

    assert_eq!(params.rule_name, "Range");
    assert_eq!(params.min, Some("0".to_string()));
    assert_eq!(params.max, Some("100".to_string()));
  }

  #[test]
  fn test_message_context_new() {
    let value = "test";
    let params = MessageParams::new("MinLength").with_min_length(5);
    let ctx = MessageContext::new(value, params);

    assert_eq!(ctx.value, "test");
    assert_eq!(ctx.params.rule_name, "MinLength");
    assert_eq!(ctx.params.min_length, Some(5));
  }

  #[test]
  fn test_message_provider_resolve_with_context() {
    let msg: Message<str> = Message::provider(|ctx| {
      format!(
        "{} validation failed: expected min length {}, got '{}'",
        ctx.params.rule_name,
        ctx
          .params
          .min_length
          .map(|n| n.to_string())
          .unwrap_or("?".to_string()),
        ctx.value
      )
    });

    let params = MessageParams::new("MinLength").with_min_length(8);
    let value = "abc";
    let ctx = MessageContext::new(value, params);

    assert_eq!(
      msg.resolve_with_context(&ctx),
      "MinLength validation failed: expected min length 8, got 'abc'"
    );
  }

  #[test]
  fn test_message_with_unsized_str() {
    let msg: Message<str> = Message::provider(|ctx| format!("Value: {}", ctx.value));
    assert_eq!(msg.resolve("hello"), "Value: hello");
  }

  #[test]
  fn test_message_with_slice() {
    let msg: Message<[i32]> = Message::Provider(Arc::new(|ctx: &MessageContext<[i32]>| {
      format!("Length: {}", ctx.value.len())
    }));
    let arr = [1, 2, 3];
    assert_eq!(msg.resolve(&arr), "Length: 3");
  }

  #[test]
  fn test_message_params_length_fields() {
    let params = MessageParams::new("MinLength")
      .with_min_length(5)
      .with_max_length(100)
      .with_exact_length(50);

    assert_eq!(params.min_length, Some(5));
    assert_eq!(params.max_length, Some(100));
    assert_eq!(params.exact_length, Some(50));
  }

  #[test]
  fn test_message_params_required() {
    let params = MessageParams::new("Required").with_required(true);

    assert!(params.required);
  }

  #[test]
  fn test_message_params_one_of() {
    let params = MessageParams::new("OneOf").with_one_of(vec![
      "a".to_string(),
      "b".to_string(),
      "c".to_string(),
    ]);

    assert_eq!(
      params.one_of,
      Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
  }

  #[test]
  fn test_message_params_all_fields() {
    let params = MessageParams::new("Custom")
      .with_required(true)
      .with_min_length(1)
      .with_max_length(100)
      .with_exact_length(50)
      .with_min(0)
      .with_max(10)
      .with_step(2)
      .with_pattern(r"^\d+$")
      .with_expected("foo")
      .with_one_of(vec!["a".to_string(), "b".to_string()]);

    assert!(params.required);
    assert_eq!(params.min_length, Some(1));
    assert_eq!(params.max_length, Some(100));
    assert_eq!(params.exact_length, Some(50));
    assert_eq!(params.min, Some("0".to_string()));
    assert_eq!(params.max, Some("10".to_string()));
    assert_eq!(params.step, Some("2".to_string()));
    assert_eq!(params.pattern, Some(r"^\d+$".to_string()));
    assert_eq!(params.expected, Some("foo".to_string()));
    assert_eq!(params.one_of, Some(vec!["a".to_string(), "b".to_string()]));
  }

  #[test]
  fn test_message_provider_with_params() {
    let msg: Message<String> = Message::provider(|ctx| {
      format!(
        "Length must be at least {}",
        ctx
          .params
          .min_length
          .map(|n| n.to_string())
          .unwrap_or("?".to_string())
      )
    });

    assert!(msg.is_provider());
    assert!(!msg.is_static());
  }

  #[test]
  fn test_message_provider_resolve_without_context() {
    // When resolve() is called without context, default params are used
    let msg: Message<String> = Message::provider(|ctx| {
      format!(
        "min: {}, max: {}",
        ctx.params.min.as_deref().unwrap_or("none"),
        ctx.params.max.as_deref().unwrap_or("none")
      )
    });

    // resolve() creates a default MessageParams
    assert_eq!(msg.resolve(&"test".to_string()), "min: none, max: none");
  }

  #[test]
  fn test_message_static_resolve_with_context() {
    // Static messages ignore context
    let msg: Message<String> = Message::from("Static error");
    let params = MessageParams::new("MinLength").with_min(8);
    let value = "test".to_string();
    let ctx = MessageContext::new(&value, params);

    assert_eq!(msg.resolve_with_context(&ctx), "Static error");
  }

  #[test]
  fn test_message_provider_uses_context_value() {
    // Provider can access both value and params from context
    let msg: Message<i32> = Message::provider(|ctx| format!("Value: {}", ctx.value));
    let params = MessageParams::new("Min").with_min(0);
    let ctx = MessageContext::new(&42, params);

    assert_eq!(msg.resolve_with_context(&ctx), "Value: 42");
  }
}

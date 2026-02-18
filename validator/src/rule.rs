//! # Rule Enum - Composable Validation Rules
//!
//! This module provides a serializable, composable validation rule system based on
//! the `Rule<T>` enum. Rules can be combined using tree composition (`All`, `Any`, `Not`, `When`)
//! and support both built-in validation types and custom closures.
//!
//! ## Design Philosophy
//!
//! - **Serialization-friendly**: Most variants are JSON/YAML serializable via serde
//! - **Composable**: Rules can be combined with `and()`, `or()`, `not()`, `when()` combinators
//! - **Type-safe**: Strongly typed with generic parameter `T`
//! - **Interoperable**: Can work alongside existing validator structs
//!
//! ## Example
//!
//! ```rust
//! use walrs_validator::rule::{Rule, Condition};
//!
//! // Simple rules
//! let min_length = Rule::<String>::MinLength(3);
//! let max_length = Rule::<String>::MaxLength(50);
//!
//! // Composed rule using combinators
//! let length_rule = min_length.and(max_length);
//!
//! // Conditional rule
//! let conditional = Rule::<String>::When {
//!     condition: Condition::IsNotEmpty,
//!     then_rule: Box::new(Rule::MinLength(5)),
//!     else_rule: None,
//! };
//! ```

use serde::{Deserialize, Serialize};
use serde_json::value::to_value as to_json_value;
use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::ViolationType;
use crate::length::WithLength;
use crate::traits::ToAttributesList;
use crate::{Message, MessageContext, SteppableValue, Violation};

// ============================================================================
// Result Types
// ============================================================================

/// Result of applying a rule to a value.
pub type RuleResult = Result<(), Violation>;

// ============================================================================
// Condition Enum
// ============================================================================

/// Conditions for `When` rules.
///
/// Conditions determine whether the `then_rule` or `else_rule` of a `When` rule
/// should be applied. Most variants are serializable for config-driven validation.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Condition<T> {
  /// Value is empty (for strings: empty or whitespace-only)
  IsEmpty,

  /// Value is not empty
  IsNotEmpty,

  /// Value equals the specified value
  Equals(T),

  /// Value is greater than the specified value
  GreaterThan(T),

  /// Value is less than the specified value
  LessThan(T),

  /// Value matches a regex pattern (string representation)
  Matches(String),

  /// Custom condition function (not serializable)
  #[serde(skip)]
  Custom(Arc<dyn Fn(&T) -> bool + Send + Sync>),
}

impl<T: Debug> Debug for Condition<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::IsEmpty => write!(f, "IsEmpty"),
      Self::IsNotEmpty => write!(f, "IsNotEmpty"),
      Self::Equals(v) => f.debug_tuple("Equals").field(v).finish(),
      Self::GreaterThan(v) => f.debug_tuple("GreaterThan").field(v).finish(),
      Self::LessThan(v) => f.debug_tuple("LessThan").field(v).finish(),
      Self::Matches(p) => f.debug_tuple("Matches").field(p).finish(),
      Self::Custom(_) => write!(f, "Custom(<fn>)"),
    }
  }
}

impl<T: PartialEq> PartialEq for Condition<T> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::IsEmpty, Self::IsEmpty) => true,
      (Self::IsNotEmpty, Self::IsNotEmpty) => true,
      (Self::Equals(a), Self::Equals(b)) => a == b,
      (Self::GreaterThan(a), Self::GreaterThan(b)) => a == b,
      (Self::LessThan(a), Self::LessThan(b)) => a == b,
      (Self::Matches(a), Self::Matches(b)) => a == b,
      // Custom conditions are never equal (function pointer comparison is not meaningful)
      (Self::Custom(_), Self::Custom(_)) => false,
      _ => false,
    }
  }
}

// ============================================================================
// Condition Evaluation
// ============================================================================

/// Trait for checking if a value is "empty" for condition evaluation.
pub trait IsEmpty {
  /// Returns `true` if the value is considered empty.
  fn is_empty(&self) -> bool;
}

impl IsEmpty for String {
  fn is_empty(&self) -> bool {
    self.trim().is_empty()
  }
}

impl IsEmpty for str {
  fn is_empty(&self) -> bool {
    self.trim().is_empty()
  }
}

impl IsEmpty for &str {
  fn is_empty(&self) -> bool {
    self.trim().is_empty()
  }
}

impl<T> IsEmpty for Vec<T> {
  fn is_empty(&self) -> bool {
    self.is_empty()
  }
}

impl<T> IsEmpty for Option<T> {
  fn is_empty(&self) -> bool {
    self.is_none()
  }
}

// Numeric types are never "empty" in the traditional sense
macro_rules! impl_is_empty_numeric {
    ($($t:ty),*) => {
        $(
            impl IsEmpty for $t {
                fn is_empty(&self) -> bool {
                    false
                }
            }
        )*
    };
}

impl_is_empty_numeric!(
  i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64
);

impl<T: PartialEq + PartialOrd> Condition<T> {
  /// Evaluates the condition against a value.
  ///
  /// Returns `true` if the condition is satisfied.
  pub fn evaluate(&self, value: &T) -> bool
  where
    T: IsEmpty,
  {
    match self {
      Condition::IsEmpty => value.is_empty(),
      Condition::IsNotEmpty => !value.is_empty(),
      Condition::Equals(expected) => value == expected,
      Condition::GreaterThan(threshold) => value > threshold,
      Condition::LessThan(threshold) => value < threshold,
      Condition::Matches(_pattern) => {
        // For Matches, we need string conversion - handled in specialized impl
        false
      }
      Condition::Custom(f) => f(value),
    }
  }
}

impl Condition<String> {
  /// Evaluates the condition against a string value, with regex support for `Matches`.
  pub fn evaluate_str(&self, value: &str) -> bool {
    match self {
      Condition::IsEmpty => value.trim().is_empty(),
      Condition::IsNotEmpty => !value.trim().is_empty(),
      Condition::Equals(expected) => value == expected,
      Condition::GreaterThan(threshold) => value > threshold.as_str(),
      Condition::LessThan(threshold) => value < threshold.as_str(),
      Condition::Matches(pattern) => regex::Regex::new(pattern)
        .map(|re| re.is_match(value))
        .unwrap_or(false),
      Condition::Custom(f) => f(&value.to_string()),
    }
  }
}

// ============================================================================
// Violation Message Helpers
// ============================================================================

/// Creates a "value missing" violation for `Required` rule.
pub fn value_missing_violation() -> Violation {
  Violation::new(ViolationType::ValueMissing, "Value is required")
}

/// Creates a "too short" violation for `MinLength` rule.
pub fn too_short_violation(min: usize, actual: usize) -> Violation {
  Violation::new(
    ViolationType::TooShort,
    format!("Value must be at least {} characters (got {})", min, actual),
  )
}

/// Creates a "too long" violation for `MaxLength` rule.
pub fn too_long_violation(max: usize, actual: usize) -> Violation {
  Violation::new(
    ViolationType::TooLong,
    format!("Value must be at most {} characters (got {})", max, actual),
  )
}

/// Creates an "exact length" violation for `ExactLength` rule.
pub fn exact_length_violation(expected: usize, actual: usize) -> Violation {
  Violation::new(
    ViolationType::TooShort, // or TooLong depending on direction
    format!(
      "Value must be exactly {} characters (got {})",
      expected, actual
    ),
  )
}

/// Creates a "pattern mismatch" violation for `Pattern` rule.
pub fn pattern_mismatch_violation(pattern: &str) -> Violation {
  Violation::new(
    ViolationType::PatternMismatch,
    format!("Value does not match pattern: {}", pattern),
  )
}

/// Creates an "invalid email" violation for `Email` rule.
pub fn invalid_email_violation() -> Violation {
  Violation::new(ViolationType::TypeMismatch, "Invalid email address")
}

/// Creates an "invalid URL" violation for `Url` rule.
pub fn invalid_url_violation() -> Violation {
  Violation::new(ViolationType::TypeMismatch, "Invalid URL")
}

/// Creates a "range underflow" violation for `Min` rule.
pub fn range_underflow_violation<T: std::fmt::Display>(min: &T) -> Violation {
  Violation::new(
    ViolationType::RangeUnderflow,
    format!("Value must be at least {}", min),
  )
}

/// Creates a "range overflow" violation for `Max` rule.
pub fn range_overflow_violation<T: std::fmt::Display>(max: &T) -> Violation {
  Violation::new(
    ViolationType::RangeOverflow,
    format!("Value must be at most {}", max),
  )
}

/// Creates a "step mismatch" violation for `Step` rule.
pub fn step_mismatch_violation<T: std::fmt::Display>(step: &T) -> Violation {
  Violation::new(
    ViolationType::StepMismatch,
    format!("Value must be a multiple of {}", step),
  )
}

/// Creates a "not equal" violation for `Equals` rule.
pub fn not_equal_violation<T: std::fmt::Display>(expected: &T) -> Violation {
  Violation::new(
    ViolationType::NotEqual,
    format!("Value must equal {}", expected),
  )
}

/// Creates a "not one of" violation for `OneOf` rule.
pub fn not_one_of_violation() -> Violation {
  Violation::new(
    ViolationType::NotEqual,
    "Value must be one of the allowed values",
  )
}

/// Creates an "unresolved reference" violation for `Ref` rule.
pub fn unresolved_ref_violation(name: &str) -> Violation {
  Violation::new(
    ViolationType::CustomError,
    format!("Unresolved rule reference: {}", name),
  )
}

/// Creates a "negation failed" violation for `Not` rule.
pub fn negation_failed_violation() -> Violation {
  Violation::new(
    ViolationType::CustomError,
    "Value must not satisfy the negated rule",
  )
}

/// Creates a "too few items" violation for collection `MinLength` rule.
pub fn too_few_items_violation(min: usize, actual: usize) -> Violation {
  Violation::new(
    ViolationType::TooShort,
    format!(
      "Collection must have at least {} items (got {})",
      min, actual
    ),
  )
}

/// Creates a "too many items" violation for collection `MaxLength` rule.
pub fn too_many_items_violation(max: usize, actual: usize) -> Violation {
  Violation::new(
    ViolationType::TooLong,
    format!(
      "Collection must have at most {} items (got {})",
      max, actual
    ),
  )
}

/// Creates an "exact item count" violation for collection `ExactLength` rule.
pub fn exact_items_violation(expected: usize, actual: usize) -> Violation {
  Violation::new(
    ViolationType::TooShort, // or TooLong depending on direction
    format!(
      "Collection must have exactly {} items (got {})",
      expected, actual
    ),
  )
}

// ============================================================================
// Rule Enum
// ============================================================================

/// A composable validation rule.
///
/// `Rule<T>` represents validation logic as data, enabling:
/// - Serialization to/from JSON/YAML for config-driven validation
/// - Tree-based composition with `All`, `Any`, `Not`, `When`
/// - Custom validation via closures
///
/// # Type Parameter
///
/// - `T`: The type of value being validated
///
/// # Serialization
///
/// Most variants are serializable. The `Custom` and `Ref` variants are skipped
/// during serialization as they contain non-serializable data (closures or
/// runtime-resolved references).
///
/// # Relationship with Validator Structs
///
/// `Rule<T>` serves as a **serializable data representation** of validation rules,
/// while the validator structs (`LengthValidator`, `RangeValidator`, etc.) provide
/// **full-featured implementations** with custom error messages and callbacks.
///
/// Use `Rule<T>` for:
/// - Config-driven validation (JSON/YAML forms)
/// - Tree-based rule composition
/// - Simple validation scenarios
///
/// Use validator structs for:
/// - Custom error messages
/// - Programmatic validation with full control
/// - Integration with existing validation pipelines
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config", rename_all = "lowercase")]
pub enum Rule<T> {
  // ---- Presence ----
  /// Value must be present (non-empty)
  Required,

  // ---- Length Rules ----
  /// Minimum length constraint
  MinLength(usize),

  /// Maximum length constraint
  MaxLength(usize),

  /// Exact length constraint
  ExactLength(usize),

  // ---- String Rules ----
  /// Regex pattern match (stored as string for serialization)
  Pattern(String),

  /// Email format validation
  Email,

  /// URL format validation
  Url,

  // ---- Numeric Rules ----
  /// Minimum value constraint
  Min(T),

  /// Maximum value constraint
  Max(T),

  /// Range constraint (inclusive)
  Range {
    /// Minimum value (inclusive)
    min: T,
    /// Maximum value (inclusive)
    max: T,
  },

  /// Step constraint (value must be divisible by step)
  Step(T),

  // ---- Comparison ----
  /// Value must equal the specified value
  Equals(T),

  /// Value must be one of the specified values
  OneOf(Vec<T>),

  // ---- Composite Rules (Tree Structure) ----
  /// All rules must pass (AND logic)
  All(Vec<Rule<T>>),

  /// At least one rule must pass (OR logic)
  Any(Vec<Rule<T>>),

  /// Negation - rule must NOT pass
  Not(Box<Rule<T>>),

  /// Conditional validation
  When {
    /// Condition to evaluate
    condition: Condition<T>,
    /// Rule to apply if condition is true
    then_rule: Box<Rule<T>>,
    /// Rule to apply if condition is false (optional)
    else_rule: Option<Box<Rule<T>>>,
  },

  // ---- Custom ----
  /// Custom validation function (not serializable)
  #[serde(skip)]
  Custom(Arc<dyn Fn(&T) -> RuleResult + Send + Sync>),

  /// Reference to a named rule (resolved at runtime)
  #[serde(skip)]
  Ref(String),

  /// Wraps another rule with a custom error message.
  ///
  /// When the inner rule fails, the custom message is used instead of
  /// the default message.
  #[serde(skip)]
  WithMessage {
    /// The wrapped rule
    rule: Box<Rule<T>>,
    /// The custom message to use on failure
    message: Message<T>,
  },
}

impl<T: Debug> Debug for Rule<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Required => write!(f, "Required"),
      Self::MinLength(n) => f.debug_tuple("MinLength").field(n).finish(),
      Self::MaxLength(n) => f.debug_tuple("MaxLength").field(n).finish(),
      Self::ExactLength(n) => f.debug_tuple("ExactLength").field(n).finish(),
      Self::Pattern(p) => f.debug_tuple("Pattern").field(p).finish(),
      Self::Email => write!(f, "Email"),
      Self::Url => write!(f, "Url"),
      Self::Min(v) => f.debug_tuple("Min").field(v).finish(),
      Self::Max(v) => f.debug_tuple("Max").field(v).finish(),
      Self::Range { min, max } => f
        .debug_struct("Range")
        .field("min", min)
        .field("max", max)
        .finish(),
      Self::Step(v) => f.debug_tuple("Step").field(v).finish(),
      Self::Equals(v) => f.debug_tuple("Equals").field(v).finish(),
      Self::OneOf(vs) => f.debug_tuple("OneOf").field(vs).finish(),
      Self::All(rules) => f.debug_tuple("All").field(rules).finish(),
      Self::Any(rules) => f.debug_tuple("Any").field(rules).finish(),
      Self::Not(rule) => f.debug_tuple("Not").field(rule).finish(),
      Self::When {
        condition,
        then_rule,
        else_rule,
      } => f
        .debug_struct("When")
        .field("condition", condition)
        .field("then_rule", then_rule)
        .field("else_rule", else_rule)
        .finish(),
      Self::Custom(_) => write!(f, "Custom(<fn>)"),
      Self::Ref(name) => f.debug_tuple("Ref").field(name).finish(),
      Self::WithMessage { rule, message } => f
        .debug_struct("WithMessage")
        .field("rule", rule)
        .field("message", message)
        .finish(),
    }
  }
}

impl<T: PartialEq> PartialEq for Rule<T> {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Required, Self::Required) => true,
      (Self::MinLength(a), Self::MinLength(b)) => a == b,
      (Self::MaxLength(a), Self::MaxLength(b)) => a == b,
      (Self::ExactLength(a), Self::ExactLength(b)) => a == b,
      (Self::Pattern(a), Self::Pattern(b)) => a == b,
      (Self::Email, Self::Email) => true,
      (Self::Url, Self::Url) => true,
      (Self::Min(a), Self::Min(b)) => a == b,
      (Self::Max(a), Self::Max(b)) => a == b,
      (Self::Range { min: a1, max: a2 }, Self::Range { min: b1, max: b2 }) => a1 == b1 && a2 == b2,
      (Self::Step(a), Self::Step(b)) => a == b,
      (Self::Equals(a), Self::Equals(b)) => a == b,
      (Self::OneOf(a), Self::OneOf(b)) => a == b,
      (Self::All(a), Self::All(b)) => a == b,
      (Self::Any(a), Self::Any(b)) => a == b,
      (Self::Not(a), Self::Not(b)) => a == b,
      (
        Self::When {
          condition: c1,
          then_rule: t1,
          else_rule: e1,
        },
        Self::When {
          condition: c2,
          then_rule: t2,
          else_rule: e2,
        },
      ) => c1 == c2 && t1 == t2 && e1 == e2,
      (Self::Ref(a), Self::Ref(b)) => a == b,
      (
        Self::WithMessage {
          rule: r1,
          message: m1,
        },
        Self::WithMessage {
          rule: r2,
          message: m2,
        },
      ) => r1 == r2 && m1 == m2,
      // Custom rules are never equal
      (Self::Custom(_), Self::Custom(_)) => false,
      _ => false,
    }
  }
}

// ============================================================================
// Rule Combinators
// ============================================================================

impl<T> Rule<T> {
  /// Returns `true` if this rule requires a value to be present.
  ///
  /// This is used by `validate_option*` methods to determine if `None` should fail.
  /// Returns `true` for `Required` rules, or `All` rules containing `Required`.
  pub fn requires_value(&self) -> bool {
    match self {
      Rule::Required => true,
      Rule::All(rules) => rules.iter().any(|r| r.requires_value()),
      Rule::WithMessage { rule, .. } => rule.requires_value(),
      _ => false,
    }
  }

  /// Combines this rule with another using AND logic.
  ///
  /// Both rules must pass for the combined rule to pass.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
  /// ```
  pub fn and(self, other: Rule<T>) -> Rule<T> {
    match self {
      Rule::All(mut rules) => {
        rules.push(other);
        Rule::All(rules)
      }
      _ => Rule::All(vec![self, other]),
    }
  }

  /// Combines this rule with another using OR logic.
  ///
  /// At least one rule must pass for the combined rule to pass.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<String>::Email.or(Rule::Url);
  /// ```
  pub fn or(self, other: Rule<T>) -> Rule<T> {
    match self {
      Rule::Any(mut rules) => {
        rules.push(other);
        Rule::Any(rules)
      }
      _ => Rule::Any(vec![self, other]),
    }
  }

  /// Negates this rule.
  ///
  /// The negated rule passes when the original rule fails.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let not_empty = Rule::<String>::MinLength(1);
  /// let is_empty = not_empty.not();
  /// ```
  pub fn not(self) -> Rule<T> {
    Rule::Not(Box::new(self))
  }

  /// Creates a conditional rule.
  ///
  /// The rule is only applied when the condition is true.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::{Rule, Condition};
  ///
  /// let rule = Rule::<String>::MinLength(8)
  ///     .when(Condition::IsNotEmpty);
  /// ```
  pub fn when(self, condition: Condition<T>) -> Rule<T> {
    Rule::When {
      condition,
      then_rule: Box::new(self),
      else_rule: None,
    }
  }

  /// Creates a conditional rule with else branch.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::{Rule, Condition};
  ///
  /// let rule = Rule::<i32>::Min(0)
  ///     .when_else(
  ///         Condition::GreaterThan(0),
  ///         Rule::Max(100),  // else rule
  ///     );
  /// ```
  pub fn when_else(self, condition: Condition<T>, else_rule: Rule<T>) -> Rule<T> {
    Rule::When {
      condition,
      then_rule: Box::new(self),
      else_rule: Some(Box::new(else_rule)),
    }
  }

  /// Creates a custom rule from a closure.
  ///
  /// # Example
  ///
  /// ```rust
  /// use std::sync::Arc;
  /// use walrs_validator::rule::Rule;
  /// use walrs_validator::{Violation, ViolationType};
  ///
  /// let is_even = Rule::<i32>::custom(Arc::new(|value: &i32| {
  ///     if value % 2 == 0 {
  ///         Ok(())
  ///     } else {
  ///         Err(Violation::new(ViolationType::CustomError, "Value must be even"))
  ///     }
  /// }));
  /// ```
  pub fn custom(f: Arc<dyn Fn(&T) -> RuleResult + Send + Sync>) -> Rule<T> {
    Rule::Custom(f)
  }

  /// Creates a reference to a named rule.
  ///
  /// Named rules are resolved at runtime from a rule registry.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<String>::rule_ref("password_strength");
  /// ```
  pub fn rule_ref(name: impl Into<String>) -> Rule<T> {
    Rule::Ref(name.into())
  }

  /// Attaches a static custom error message to this rule.
  ///
  /// When validation fails, the custom message is used instead of
  /// the default message generated by the rule.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<String>::MinLength(8)
  ///     .with_message("Password must be at least 8 characters");
  /// ```
  pub fn with_message(self, msg: impl Into<String>) -> Rule<T> {
    Rule::WithMessage {
      rule: Box::new(self),
      message: Message::Static(msg.into()),
    }
  }

  /// Attaches a dynamic message provider to this rule.
  ///
  /// The closure receives a `MessageContext` containing the value being validated
  /// and rule parameters, enabling rich interpolated error messages.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<i32>::Min(0)
  ///     .with_message_provider(|ctx| format!("Value {} must be non-negative", ctx.value));
  /// ```
  pub fn with_message_provider<F>(self, f: F) -> Rule<T>
  where
    F: Fn(&MessageContext<T>) -> String + Send + Sync + 'static,
  {
    Rule::WithMessage {
      rule: Box::new(self),
      message: Message::Provider(Arc::new(f)),
    }
  }
}

// ============================================================================
// Rule Constructors (Convenience Methods)
// ============================================================================

impl<T> Rule<T> {
  /// Creates a `Required` rule.
  pub fn required() -> Rule<T> {
    Rule::Required
  }

  /// Creates a `MinLength` rule.
  pub fn min_length(len: usize) -> Rule<T> {
    Rule::MinLength(len)
  }

  /// Creates a `MaxLength` rule.
  pub fn max_length(len: usize) -> Rule<T> {
    Rule::MaxLength(len)
  }

  /// Creates an `ExactLength` rule.
  pub fn exact_length(len: usize) -> Rule<T> {
    Rule::ExactLength(len)
  }

  /// Creates a `Pattern` rule.
  pub fn pattern(pattern: impl Into<String>) -> Rule<T> {
    Rule::Pattern(pattern.into())
  }

  /// Creates an `Email` rule.
  pub fn email() -> Rule<T> {
    Rule::Email
  }

  /// Creates a `Url` rule.
  pub fn url() -> Rule<T> {
    Rule::Url
  }

  /// Creates a `Min` rule.
  pub fn min(value: T) -> Rule<T> {
    Rule::Min(value)
  }

  /// Creates a `Max` rule.
  pub fn max(value: T) -> Rule<T> {
    Rule::Max(value)
  }

  /// Creates a `Range` rule.
  pub fn range(min: T, max: T) -> Rule<T> {
    Rule::Range { min, max }
  }

  /// Creates a `Step` rule.
  pub fn step(value: T) -> Rule<T> {
    Rule::Step(value)
  }

  /// Creates an `Equals` rule.
  pub fn equals(value: T) -> Rule<T> {
    Rule::Equals(value)
  }

  /// Creates a `OneOf` rule.
  pub fn one_of(values: Vec<T>) -> Rule<T> {
    Rule::OneOf(values)
  }

  /// Creates an `All` rule (AND composition).
  pub fn all(rules: Vec<Rule<T>>) -> Rule<T> {
    Rule::All(rules)
  }

  /// Creates an `Any` rule (OR composition).
  pub fn any(rules: Vec<Rule<T>>) -> Rule<T> {
    Rule::Any(rules)
  }
}

// ============================================================================
// Rule Validation - String Types
// ============================================================================

// TODO: Check whether it is ok to implement this and what the
//    implications are if doing so.
impl Rule<&str> {}

impl Rule<String> {
  /// Validates a string value against this rule.
  ///
  /// Returns `Ok(())` if validation passes, or `Err(Violation)` on first failure.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
  /// assert!(rule.validate_ref("hello").is_ok());
  /// assert!(rule.validate_ref("hi").is_err());
  /// ```
  pub fn validate_ref(&self, value: &str) -> RuleResult {
    match self {
      Rule::Required => {
        if value.trim().is_empty() {
          Err(value_missing_violation())
        } else {
          Ok(())
        }
      }
      Rule::MinLength(min) => {
        let len = value.chars().count();
        if len < *min {
          Err(too_short_violation(*min, len))
        } else {
          Ok(())
        }
      }
      Rule::MaxLength(max) => {
        let len = value.chars().count();
        if len > *max {
          Err(too_long_violation(*max, len))
        } else {
          Ok(())
        }
      }
      Rule::ExactLength(expected) => {
        let len = value.chars().count();
        if len != *expected {
          Err(exact_length_violation(*expected, len))
        } else {
          Ok(())
        }
      }
      Rule::Pattern(pattern) => match regex::Regex::new(pattern) {
        Ok(re) => {
          if re.is_match(value) {
            Ok(())
          } else {
            Err(pattern_mismatch_violation(pattern))
          }
        }
        Err(_) => Err(pattern_mismatch_violation(pattern)),
      },
      Rule::Email => {
        // Simple email validation using regex
        let email_re =
          regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if email_re.is_match(value) {
          Ok(())
        } else {
          Err(invalid_email_violation())
        }
      }
      Rule::Url => {
        // Simple URL validation using regex
        let url_re = regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        if url_re.is_match(value) {
          Ok(())
        } else {
          Err(invalid_url_violation())
        }
      }
      Rule::Equals(expected) => {
        if value == expected {
          Ok(())
        } else {
          Err(not_equal_violation(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.iter().any(|v| v == value) {
          Ok(())
        } else {
          Err(not_one_of_violation())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate_ref(value)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_ref(value) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_ref(value) {
        Ok(()) => Err(negation_failed_violation()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate_str(value);
        if should_apply {
          then_rule.validate_ref(value)
        } else {
          match else_rule {
            Some(rule) => rule.validate_ref(value),
            None => Ok(()),
          }
        }
      }
      Rule::Custom(f) => f(&value.to_string()),
      Rule::Ref(name) => Err(unresolved_ref_violation(name)),
      Rule::WithMessage { rule, message } => match rule.validate_ref(value) {
        Ok(()) => Ok(()),
        Err(violation) => {
          let custom_msg = message.resolve(&value.to_string());
          Err(Violation::new(violation.violation_type(), custom_msg))
        }
      },
      // Numeric rules don't apply to strings - pass through
      Rule::Min(_) | Rule::Max(_) | Rule::Range { .. } | Rule::Step(_) => Ok(()),
    }
  }

  /// Validates a string value and collects all violations.
  ///
  /// Returns `Ok(())` if validation passes, or `Err(Violations)` with all failures.
  pub fn validate_ref_all(&self, value: &str) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_violations_ref(value, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional string value.
  ///
  /// - If `value` is `None` and `requires_value()` is true, returns `Err(ValueMissing)`.
  /// - If `value` is `None` and `requires_value()` is false, returns `Ok(())`.
  /// - If `value` is `Some(v)`, delegates to `validate_ref`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<String>::MinLength(3);
  /// assert!(rule.validate_ref_option(None).is_ok()); // None is OK for non-required
  /// assert!(rule.validate_ref_option(Some("hello")).is_ok());
  /// assert!(rule.validate_ref_option(Some("hi")).is_err());
  ///
  /// let required = Rule::<String>::Required;
  /// assert!(required.validate_ref_option(None).is_err()); // None fails Required
  /// assert!(required.validate_ref_option(Some("value")).is_ok());
  /// ```
  pub fn validate_ref_option(&self, value: Option<&str>) -> RuleResult {
    match value {
      Some(v) => self.validate_ref(v),
      None if self.requires_value() => Err(value_missing_violation()),
      None => Ok(()),
    }
  }

  /// Validates an optional string value and collects all violations.
  pub fn validate_ref_option_all(&self, value: Option<&str>) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_ref_all(v),
      None if self.requires_value() => Err(crate::Violations::from(value_missing_violation())),
      None => Ok(()),
    }
  }

  /// Helper to collect all violations recursively.
  fn collect_violations_ref(&self, value: &str, violations: &mut crate::Violations) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_violations_ref(value, violations);
        }
      }
      Rule::Any(rules) => {
        // For Any, we only add violations if ALL rules fail
        let mut any_violations = crate::Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = crate::Violations::default();
          rule.collect_violations_ref(value, &mut rule_violations);
          if rule_violations.is_empty() {
            any_passed = true;
            break;
          }
          any_violations.extend(rule_violations.into_iter());
        }
        if !any_passed && !rules.is_empty() {
          // Just add the last violation for Any
          if let Some(v) = any_violations.0.pop() {
            violations.push(v);
          }
        }
      }
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate_str(value);
        if should_apply {
          then_rule.collect_violations_ref(value, violations);
        } else if let Some(rule) = else_rule {
          rule.collect_violations_ref(value, violations);
        }
      }
      Rule::WithMessage { rule, message } => {
        let mut inner_violations = crate::Violations::default();
        rule.collect_violations_ref(value, &mut inner_violations);
        for violation in inner_violations {
          let custom_msg = message.resolve(&value.to_string());
          violations.push(Violation::new(violation.violation_type(), custom_msg));
        }
      }
      _ => {
        if let Err(v) = self.validate_ref(value) {
          violations.push(v);
        }
      }
    }
  }
}

// ============================================================================
// Rule Validation - Numeric Types
// ============================================================================

impl<T: SteppableValue + IsEmpty> Rule<T> {
  /// Validates a numeric value against this rule.
  ///
  /// Returns `Ok(())` if validation passes, or `Err(Violation)` on first failure.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
  /// assert!(rule.validate(50).is_ok());
  /// assert!(rule.validate(-5).is_err());
  /// ```
  pub fn validate(&self, value: T) -> RuleResult {
    match self {
      Rule::Required => {
        // Numeric values are always "present"
        Ok(())
      }
      Rule::Min(min) => {
        if value < *min {
          Err(range_underflow_violation(min))
        } else {
          Ok(())
        }
      }
      Rule::Max(max) => {
        if value > *max {
          Err(range_overflow_violation(max))
        } else {
          Ok(())
        }
      }
      Rule::Range { min, max } => {
        if value < *min {
          Err(range_underflow_violation(min))
        } else if value > *max {
          Err(range_overflow_violation(max))
        } else {
          Ok(())
        }
      }
      Rule::Step(step) => {
        if value.rem_check(*step) {
          Ok(())
        } else {
          Err(step_mismatch_violation(step))
        }
      }
      Rule::Equals(expected) => {
        if value == *expected {
          Ok(())
        } else {
          Err(not_equal_violation(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.contains(&value) {
          Ok(())
        } else {
          Err(not_one_of_violation())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate(value)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate(value) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate(value) {
        Ok(()) => Err(negation_failed_violation()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate(&value);
        if should_apply {
          then_rule.validate(value)
        } else {
          match else_rule {
            Some(rule) => rule.validate(value),
            None => Ok(()),
          }
        }
      }
      Rule::Custom(f) => f(&value),
      Rule::Ref(name) => Err(unresolved_ref_violation(name)),
      Rule::WithMessage { rule, message } => match rule.validate(value) {
        Ok(()) => Ok(()),
        Err(violation) => {
          let custom_msg = message.resolve(&value);
          Err(Violation::new(violation.violation_type(), custom_msg))
        }
      },
      // String rules don't apply to numbers - pass through
      Rule::MinLength(_)
      | Rule::MaxLength(_)
      | Rule::ExactLength(_)
      | Rule::Pattern(_)
      | Rule::Email
      | Rule::Url => Ok(()),
    }
  }

  /// Validates a numeric value and collects all violations.
  ///
  /// Returns `Ok(())` if validation passes, or `Err(Violations)` with all failures.
  pub fn validate_all(&self, value: T) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_violations(value, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional numeric value.
  ///
  /// - If `value` is `None`, returns `Err(ValueMissing)`.
  /// - If `value` is `Some(v)`, delegates to `validate`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
  /// assert!(rule.validate_option(None).is_err()); // None is an error
  /// assert!(rule.validate_option(Some(50)).is_ok());
  /// assert!(rule.validate_option(Some(-5)).is_err());
  /// ```
  pub fn validate_option(&self, value: Option<T>) -> RuleResult {
    match value {
      Some(v) => self.validate(v),
      None => Err(value_missing_violation()),
    }
  }

  /// Validates an optional numeric value and collects all violations.
  pub fn validate_option_all(&self, value: Option<T>) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_all(v),
      None => Err(crate::Violations::from(value_missing_violation())),
    }
  }

  /// Helper to collect all violations recursively.
  fn collect_violations(&self, value: T, violations: &mut crate::Violations) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_violations(value, violations);
        }
      }
      Rule::Any(rules) => {
        let mut any_violations = crate::Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = crate::Violations::default();
          rule.collect_violations(value, &mut rule_violations);
          if rule_violations.is_empty() {
            any_passed = true;
            break;
          }
          any_violations.extend(rule_violations.into_iter());
        }
        if !any_passed && !rules.is_empty() {
          if let Some(v) = any_violations.0.pop() {
            violations.push(v);
          }
        }
      }
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate(&value);
        if should_apply {
          then_rule.collect_violations(value, violations);
        } else if let Some(rule) = else_rule {
          rule.collect_violations(value, violations);
        }
      }
      Rule::WithMessage { rule, message } => {
        let mut inner_violations = crate::Violations::default();
        rule.collect_violations(value, &mut inner_violations);
        for violation in inner_violations {
          let custom_msg = message.resolve(&value);
          violations.push(Violation::new(violation.violation_type(), custom_msg));
        }
      }
      _ => {
        if let Err(v) = self.validate(value) {
          violations.push(v);
        }
      }
    }
  }
}

// ============================================================================
// CompiledRule - Cached Validator Wrapper
// ============================================================================

use std::sync::OnceLock;

/// Cached validators for a compiled rule.
struct CachedStringValidators {
  /// Cached regex for Pattern rules
  pattern_regex: Option<regex::Regex>,
  /// Cached email regex
  email_regex: Option<regex::Regex>,
  /// Cached URL regex
  url_regex: Option<regex::Regex>,
}

impl CachedStringValidators {
  fn new() -> Self {
    Self {
      pattern_regex: None,
      email_regex: None,
      url_regex: None,
    }
  }
}

/// A compiled rule with cached validators for better performance.
///
/// Use `CompiledRule` when you need to validate many values against the same rule.
/// The compiled form caches regex patterns and other validators to avoid
/// repeated compilation.
///
/// # Example
///
/// ```rust
/// use walrs_validator::rule::Rule;
///
/// // Define and compile rule once
/// let rule = Rule::<String>::MinLength(8)
///     .and(Rule::Pattern(r"[A-Z]".to_string()));
/// let compiled = rule.compile();
///
/// // Validate many times (reuses cached regex)
/// assert!(compiled.validate_ref("Password1").is_ok());
/// assert!(compiled.validate_ref("short").is_err());
/// ```
pub struct CompiledRule<T> {
  /// The underlying rule
  rule: Rule<T>,
  /// Cached string validators (lazily initialized)
  string_cache: OnceLock<CachedStringValidators>,
}

impl<T: Clone> CompiledRule<T> {
  /// Creates a new compiled rule from an existing rule.
  pub fn new(rule: Rule<T>) -> Self {
    Self {
      rule,
      string_cache: OnceLock::new(),
    }
  }

  /// Returns a reference to the underlying rule.
  pub fn rule(&self) -> &Rule<T> {
    &self.rule
  }

  /// Consumes the compiled rule and returns the underlying rule.
  pub fn into_rule(self) -> Rule<T> {
    self.rule
  }
}

impl<T: Clone> Clone for CompiledRule<T> {
  fn clone(&self) -> Self {
    Self {
      rule: self.rule.clone(),
      string_cache: OnceLock::new(), // Reset cache on clone
    }
  }
}

impl<T: Debug> Debug for CompiledRule<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CompiledRule")
      .field("rule", &self.rule)
      .finish()
  }
}

impl Rule<String> {
  /// Compiles this rule for efficient repeated validation.
  ///
  /// The compiled form caches regex patterns and other validators.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<String>::Pattern(r"^\d+$".to_string());
  /// let compiled = rule.compile();
  ///
  /// // Repeated calls reuse the cached regex
  /// assert!(compiled.validate_ref("123").is_ok());
  /// assert!(compiled.validate_ref("456").is_ok());
  /// ```
  pub fn compile(self) -> CompiledRule<String> {
    CompiledRule::new(self)
  }
}

impl<T: SteppableValue + IsEmpty + Clone> Rule<T> {
  /// Compiles this rule for efficient repeated validation.
  pub fn compile(self) -> CompiledRule<T> {
    CompiledRule::new(self)
  }
}

impl CompiledRule<String> {
  /// Gets or initializes the cached string validators.
  fn get_or_init_cache(&self) -> &CachedStringValidators {
    self.string_cache.get_or_init(|| {
      let mut cache = CachedStringValidators::new();

      // Pre-compile pattern regex if applicable
      if let Rule::Pattern(pattern) = &self.rule {
        cache.pattern_regex = regex::Regex::new(pattern).ok();
      }

      // Pre-compile email regex
      cache.email_regex =
        regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").ok();

      // Pre-compile URL regex
      cache.url_regex = regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").ok();

      cache
    })
  }

  /// Validates a string value using cached validators.
  ///
  /// This is more efficient than `Rule::validate_ref` when validating
  /// multiple values, as regex patterns are compiled once and reused.
  pub fn validate_ref(&self, value: &str) -> RuleResult {
    self.validate_ref_with_cache(value, self.get_or_init_cache())
  }

  fn validate_ref_with_cache(&self, value: &str, cache: &CachedStringValidators) -> RuleResult {
    match &self.rule {
      Rule::Required => {
        if value.trim().is_empty() {
          Err(value_missing_violation())
        } else {
          Ok(())
        }
      }
      Rule::MinLength(min) => {
        let len = value.chars().count();
        if len < *min {
          Err(too_short_violation(*min, len))
        } else {
          Ok(())
        }
      }
      Rule::MaxLength(max) => {
        let len = value.chars().count();
        if len > *max {
          Err(too_long_violation(*max, len))
        } else {
          Ok(())
        }
      }
      Rule::ExactLength(expected) => {
        let len = value.chars().count();
        if len != *expected {
          Err(exact_length_violation(*expected, len))
        } else {
          Ok(())
        }
      }
      Rule::Pattern(pattern) => {
        // Use cached regex if available
        let matches = cache
          .pattern_regex
          .as_ref()
          .map(|re| re.is_match(value))
          .unwrap_or_else(|| {
            regex::Regex::new(pattern)
              .map(|re| re.is_match(value))
              .unwrap_or(false)
          });
        if matches {
          Ok(())
        } else {
          Err(pattern_mismatch_violation(pattern))
        }
      }
      Rule::Email => {
        let matches = cache
          .email_regex
          .as_ref()
          .map(|re| re.is_match(value))
          .unwrap_or(false);
        if matches {
          Ok(())
        } else {
          Err(invalid_email_violation())
        }
      }
      Rule::Url => {
        let matches = cache
          .url_regex
          .as_ref()
          .map(|re| re.is_match(value))
          .unwrap_or(false);
        if matches {
          Ok(())
        } else {
          Err(invalid_url_violation())
        }
      }
      Rule::Equals(expected) => {
        if value == expected {
          Ok(())
        } else {
          Err(not_equal_violation(expected))
        }
      }
      Rule::OneOf(allowed) => {
        if allowed.iter().any(|v| v == value) {
          Ok(())
        } else {
          Err(not_one_of_violation())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          CompiledRule::new(rule.clone()).validate_ref(value)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match CompiledRule::new(rule.clone()).validate_ref(value) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match CompiledRule::new((**inner).clone()).validate_ref(value) {
        Ok(()) => Err(negation_failed_violation()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        let should_apply = condition.evaluate_str(value);
        if should_apply {
          CompiledRule::new((**then_rule).clone()).validate_ref(value)
        } else {
          match else_rule {
            Some(rule) => CompiledRule::new((**rule).clone()).validate_ref(value),
            None => Ok(()),
          }
        }
      }
      Rule::Custom(f) => f(&value.to_string()),
      Rule::Ref(name) => Err(unresolved_ref_violation(name)),
      Rule::WithMessage { rule, message } => {
        match CompiledRule::new((**rule).clone()).validate_ref(value) {
          Ok(()) => Ok(()),
          Err(violation) => {
            let custom_msg = message.resolve(&value.to_string());
            Err(Violation::new(violation.violation_type(), custom_msg))
          }
        }
      }
      Rule::Min(_) | Rule::Max(_) | Rule::Range { .. } | Rule::Step(_) => Ok(()),
    }
  }

  /// Validates a string value and collects all violations.
  pub fn validate_ref_all(&self, value: &str) -> Result<(), crate::Violations> {
    self.rule.validate_ref_all(value)
  }
}

impl<T: SteppableValue + IsEmpty + Clone> CompiledRule<T> {
  /// Validates a numeric value using the compiled rule.
  pub fn validate(&self, value: T) -> RuleResult {
    self.rule.validate(value)
  }

  /// Validates a numeric value and collects all violations.
  pub fn validate_all(&self, value: T) -> Result<(), crate::Violations> {
    self.rule.validate_all(value)
  }
}

// ============================================================================
// Trait Implementations for Rule and CompiledRule
// ============================================================================

use crate::traits::{Validate, ValidateRef};

impl ValidateRef<str> for Rule<String> {
  fn validate_ref(&self, value: &str) -> crate::ValidatorResult {
    Rule::validate_ref(self, value)
  }
}

impl ValidateRef<str> for CompiledRule<String> {
  fn validate_ref(&self, value: &str) -> crate::ValidatorResult {
    CompiledRule::validate_ref(self, value)
  }
}

impl<T: SteppableValue + IsEmpty + Clone> Validate<T> for Rule<T> {
  fn validate(&self, value: T) -> crate::ValidatorResult {
    Rule::validate(self, value)
  }
}

impl<T: SteppableValue + IsEmpty + Clone> Validate<T> for CompiledRule<T> {
  fn validate(&self, value: T) -> crate::ValidatorResult {
    CompiledRule::validate(self, value)
  }
}

// ============================================================================
// ToAttributesList Implementation for Rule
// ============================================================================

impl<T: Serialize> ToAttributesList for Rule<T> {
  /// Converts rule variants to HTML attribute key-value pairs.
  ///
  /// Returns a list of attribute name/value pairs suitable for HTML form elements.
  /// Composite rules (`All`/`Any`) flatten their children's attributes.
  /// Non-attribute-mappable variants return `None`.
  ///
  /// # HTML Attribute Mappings
  ///
  /// | Rule Variant | HTML Attribute(s) |
  /// |--------------|-------------------|
  /// | `Required` | `required=true` |
  /// | `MinLength(n)` | `minlength=n` |
  /// | `MaxLength(n)` | `maxlength=n` |
  /// | `ExactLength(n)` | `minlength=n`, `maxlength=n` |
  /// | `Pattern(p)` | `pattern=p` |
  /// | `Email` | `type=email` |
  /// | `Url` | `type=url` |
  /// | `Min(v)` | `min=v` |
  /// | `Max(v)` | `max=v` |
  /// | `Range { min, max }` | `min=min`, `max=max` |
  /// | `Step(v)` | `step=v` |
  /// | `All(rules)` | Flattened child attributes |
  /// | `Any(rules)` | Flattened child attributes |
  /// | Other variants | `None` |
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::{Rule, ToAttributesList};
  ///
  /// let rule = Rule::<String>::MinLength(3);
  /// let attrs = rule.to_attributes_list().unwrap();
  /// assert_eq!(attrs.len(), 1);
  /// assert_eq!(attrs[0].0, "minlength");
  /// assert_eq!(attrs[0].1, 3);
  ///
  /// // Composite rules flatten attributes
  /// let composite = Rule::<String>::MinLength(3).and(Rule::MaxLength(50));
  /// let attrs = composite.to_attributes_list().unwrap();
  /// assert_eq!(attrs.len(), 2);
  /// ```
  fn to_attributes_list(&self) -> Option<Vec<(String, serde_json::Value)>> {
    match self {
      // Presence
      Rule::Required => Some(vec![(
        "required".to_string(),
        serde_json::Value::Bool(true),
      )]),

      // Length Rules
      Rule::MinLength(n) => Some(vec![("minlength".to_string(), serde_json::Value::from(*n))]),
      Rule::MaxLength(n) => Some(vec![("maxlength".to_string(), serde_json::Value::from(*n))]),
      Rule::ExactLength(n) => Some(vec![
        ("minlength".to_string(), serde_json::Value::from(*n)),
        ("maxlength".to_string(), serde_json::Value::from(*n)),
      ]),

      // String Rules
      Rule::Pattern(p) => Some(vec![(
        "pattern".to_string(),
        serde_json::Value::from(p.clone()),
      )]),
      Rule::Email => Some(vec![("type".to_string(), serde_json::Value::from("email"))]),
      Rule::Url => Some(vec![("type".to_string(), serde_json::Value::from("url"))]),

      // Numeric Rules
      Rule::Min(v) => to_json_value(v)
        .ok()
        .map(|val| vec![("min".to_string(), val)]),
      Rule::Max(v) => to_json_value(v)
        .ok()
        .map(|val| vec![("max".to_string(), val)]),
      Rule::Range { min, max } => {
        let min_val = to_json_value(min).ok()?;
        let max_val = to_json_value(max).ok()?;
        Some(vec![
          ("min".to_string(), min_val),
          ("max".to_string(), max_val),
        ])
      }
      Rule::Step(v) => to_json_value(v)
        .ok()
        .map(|val| vec![("step".to_string(), val)]),

      // Comparison - Equals doesn't have a direct HTML attribute equivalent
      Rule::Equals(_) => None,
      Rule::OneOf(_) => None,

      // Composite Rules - flatten children
      Rule::All(rules) => {
        let mut attrs = Vec::new();
        for rule in rules {
          if let Some(child_attrs) = rule.to_attributes_list() {
            attrs.extend(child_attrs);
          }
        }
        if attrs.is_empty() { None } else { Some(attrs) }
      }
      Rule::Any(rules) => {
        let mut attrs = Vec::new();
        for rule in rules {
          if let Some(child_attrs) = rule.to_attributes_list() {
            attrs.extend(child_attrs);
          }
        }
        if attrs.is_empty() { None } else { Some(attrs) }
      }

      // Not - negation doesn't map to HTML attributes
      Rule::Not(_) => None,

      // Conditional - doesn't map to HTML attributes
      Rule::When { .. } => None,

      // Custom/Runtime variants - not attribute-mappable
      Rule::Custom(_) => None,
      Rule::Ref(_) => None,

      // WithMessage - delegate to inner rule
      Rule::WithMessage { rule, .. } => rule.to_attributes_list(),
    }
  }
}

// ============================================================================
// Rule Validation - Collection Types (WithLength)
// ============================================================================

impl<T: WithLength> Rule<T> {
  /// Validates a collection's length against this rule.
  ///
  /// Returns `Ok(())` if validation passes, or `Err(Violation)` on first failure.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<Vec<i32>>::MinLength(2).and(Rule::MaxLength(5));
  /// assert!(rule.validate_len_ref(&vec![1, 2, 3]).is_ok());
  /// assert!(rule.validate_len_ref(&vec![1]).is_err());
  /// assert!(rule.validate_len_ref(&vec![1, 2, 3, 4, 5, 6]).is_err());
  /// ```
  pub fn validate_len_ref(&self, value: &T) -> RuleResult {
    match self {
      Rule::Required => {
        if value.length() == 0 {
          Err(value_missing_violation())
        } else {
          Ok(())
        }
      }
      Rule::MinLength(min) => {
        let len = value.length();
        if len < *min {
          Err(too_few_items_violation(*min, len))
        } else {
          Ok(())
        }
      }
      Rule::MaxLength(max) => {
        let len = value.length();
        if len > *max {
          Err(too_many_items_violation(*max, len))
        } else {
          Ok(())
        }
      }
      Rule::ExactLength(expected) => {
        let len = value.length();
        if len != *expected {
          Err(exact_items_violation(*expected, len))
        } else {
          Ok(())
        }
      }
      Rule::All(rules) => {
        for rule in rules {
          rule.validate_len_ref(value)?;
        }
        Ok(())
      }
      Rule::Any(rules) => {
        if rules.is_empty() {
          return Ok(());
        }
        let mut last_err = None;
        for rule in rules {
          match rule.validate_len_ref(value) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
          }
        }
        Err(last_err.unwrap())
      }
      Rule::Not(inner) => match inner.validate_len_ref(value) {
        Ok(()) => Err(negation_failed_violation()),
        Err(_) => Ok(()),
      },
      Rule::When {
        condition: _,
        then_rule,
        else_rule: _,
      } => {
        // For collections, we only support simple condition evaluation based on emptiness
        // Full condition evaluation would require additional trait bounds
        // For now, always apply then_rule if value is not empty
        if value.length() > 0 {
          then_rule.validate_len_ref(value)?;
        }
        Ok(())
      }
      Rule::Custom(_) => {
        // Custom rules are not supported for generic WithLength validation
        // as they require the specific type T
        Ok(())
      }
      Rule::Ref(name) => Err(unresolved_ref_violation(name)),
      Rule::WithMessage { rule, message: _ } => {
        // For WithLength types, we can't easily resolve messages without more bounds
        // Just delegate to inner rule
        rule.validate_len_ref(value)
      }
      // Non-length rules don't apply to collections - pass through
      Rule::Pattern(_)
      | Rule::Email
      | Rule::Url
      | Rule::Min(_)
      | Rule::Max(_)
      | Rule::Range { .. }
      | Rule::Step(_)
      | Rule::Equals(_)
      | Rule::OneOf(_) => Ok(()),
    }
  }

  /// Validates a collection's length and collects all violations.
  ///
  /// Returns `Ok(())` if validation passes, or `Err(Violations)` with all failures.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<Vec<i32>>::MinLength(3).and(Rule::MaxLength(5));
  /// let result = rule.validate_len_ref_all(&vec![1]);
  /// assert!(result.is_err());
  /// ```
  pub fn validate_len_ref_all(&self, value: &T) -> Result<(), crate::Violations> {
    let mut violations = crate::Violations::default();
    self.collect_len_violations_ref(value, &mut violations);
    if violations.is_empty() {
      Ok(())
    } else {
      Err(violations)
    }
  }

  /// Validates an optional collection's length.
  ///
  /// - If `value` is `None` and `requires_value()` is true, returns `Err(ValueMissing)`.
  /// - If `value` is `None` and `requires_value()` is false, returns `Ok(())`.
  /// - If `value` is `Some(v)`, delegates to `validate_len_ref`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validator::rule::Rule;
  ///
  /// let rule = Rule::<Vec<i32>>::MinLength(2);
  /// assert!(rule.validate_len_ref_option(None).is_ok()); // None is OK for non-required
  /// assert!(rule.validate_len_ref_option(Some(&vec![1, 2, 3])).is_ok());
  /// assert!(rule.validate_len_ref_option(Some(&vec![1])).is_err());
  ///
  /// let required = Rule::<Vec<i32>>::Required;
  /// assert!(required.validate_len_ref_option(None).is_err()); // None fails Required
  /// ```
  pub fn validate_len_ref_option(&self, value: Option<&T>) -> RuleResult {
    match value {
      Some(v) => self.validate_len_ref(v),
      None if self.requires_value() => Err(value_missing_violation()),
      None => Ok(()),
    }
  }

  /// Validates an optional collection's length and collects all violations.
  pub fn validate_len_ref_option_all(&self, value: Option<&T>) -> Result<(), crate::Violations> {
    match value {
      Some(v) => self.validate_len_ref_all(v),
      None if self.requires_value() => Err(crate::Violations::from(value_missing_violation())),
      None => Ok(()),
    }
  }

  /// Helper to collect all length violations recursively.
  fn collect_len_violations_ref(&self, value: &T, violations: &mut crate::Violations) {
    match self {
      Rule::All(rules) => {
        for rule in rules {
          rule.collect_len_violations_ref(value, violations);
        }
      }
      Rule::Any(rules) => {
        // For Any, we only add violations if ALL rules fail
        let mut any_violations = crate::Violations::default();
        let mut any_passed = false;
        for rule in rules {
          let mut rule_violations = crate::Violations::default();
          rule.collect_len_violations_ref(value, &mut rule_violations);
          if rule_violations.is_empty() {
            any_passed = true;
            break;
          }
          any_violations.extend(rule_violations.into_iter());
        }
        if !any_passed && !rules.is_empty() {
          // Just add the last violation for Any
          if let Some(v) = any_violations.0.pop() {
            violations.push(v);
          }
        }
      }
      Rule::When {
        condition: _,
        then_rule,
        else_rule: _,
      } => {
        // For collections, apply then_rule if not empty
        if value.length() > 0 {
          then_rule.collect_len_violations_ref(value, violations);
        }
      }
      Rule::WithMessage { rule, message: _ } => {
        // Delegate to inner rule
        rule.collect_len_violations_ref(value, violations);
      }
      _ => {
        if let Err(v) = self.validate_len_ref(value) {
          violations.push(v);
        }
      }
    }
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rule_and_combinator() {
    let rule1 = Rule::<String>::MinLength(3);
    let rule2 = Rule::<String>::MaxLength(10);
    let combined = rule1.and(rule2);

    match combined {
      Rule::All(rules) => {
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0], Rule::MinLength(3));
        assert_eq!(rules[1], Rule::MaxLength(10));
      }
      _ => panic!("Expected Rule::All"),
    }
  }

  #[test]
  fn test_rule_and_combinator_flattens() {
    let rule1 = Rule::<String>::MinLength(3);
    let rule2 = Rule::<String>::MaxLength(10);
    let rule3 = Rule::<String>::Pattern(r"^\w+$".to_string());

    let combined = rule1.and(rule2).and(rule3);

    match combined {
      Rule::All(rules) => {
        assert_eq!(rules.len(), 3);
      }
      _ => panic!("Expected Rule::All"),
    }
  }

  #[test]
  fn test_rule_or_combinator() {
    let rule1 = Rule::<String>::Email;
    let rule2 = Rule::<String>::Url;
    let combined = rule1.or(rule2);

    match combined {
      Rule::Any(rules) => {
        assert_eq!(rules.len(), 2);
      }
      _ => panic!("Expected Rule::Any"),
    }
  }

  #[test]
  fn test_rule_not_combinator() {
    let rule = Rule::<String>::MinLength(1);
    let negated = rule.not();

    match negated {
      Rule::Not(inner) => {
        assert_eq!(*inner, Rule::MinLength(1));
      }
      _ => panic!("Expected Rule::Not"),
    }
  }

  #[test]
  fn test_rule_when_combinator() {
    let rule = Rule::<String>::MinLength(8).when(Condition::IsNotEmpty);

    match rule {
      Rule::When {
        condition,
        then_rule,
        else_rule,
      } => {
        assert_eq!(condition, Condition::IsNotEmpty);
        assert_eq!(*then_rule, Rule::MinLength(8));
        assert!(else_rule.is_none());
      }
      _ => panic!("Expected Rule::When"),
    }
  }

  #[test]
  fn test_rule_equality() {
    assert_eq!(Rule::<String>::Required, Rule::Required);
    assert_eq!(Rule::<String>::MinLength(5), Rule::MinLength(5));
    assert_ne!(Rule::<String>::MinLength(5), Rule::MinLength(10));
    assert_eq!(Rule::<i32>::Min(0), Rule::Min(0));
    assert_eq!(
      Rule::<i32>::Range { min: 0, max: 100 },
      Rule::Range { min: 0, max: 100 }
    );
  }

  #[test]
  fn test_rule_debug() {
    let rule = Rule::<String>::MinLength(5);
    let debug_str = format!("{:?}", rule);
    assert!(debug_str.contains("MinLength"));
    assert!(debug_str.contains("5"));
  }

  #[test]
  fn test_condition_equality() {
    assert_eq!(Condition::<String>::IsEmpty, Condition::IsEmpty);
    assert_eq!(Condition::<i32>::Equals(5), Condition::Equals(5));
    assert_ne!(Condition::<i32>::Equals(5), Condition::Equals(10));
  }

  #[test]
  fn test_rule_serialization() {
    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    let json = serde_json::to_string(&rule).unwrap();
    assert!(json.contains("range")); // lowercase due to rename_all
    assert!(json.contains("0"));
    assert!(json.contains("100"));

    let deserialized: Rule<i32> = serde_json::from_str(&json).unwrap();
    assert_eq!(rule, deserialized);
  }

  #[test]
  fn test_complex_rule_serialization() {
    let rule = Rule::<String>::All(vec![
      Rule::Required,
      Rule::MinLength(3),
      Rule::MaxLength(50),
    ]);

    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: Rule<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(rule, deserialized);
  }

  #[test]
  fn test_convenience_constructors() {
    assert_eq!(Rule::<String>::required(), Rule::Required);
    assert_eq!(Rule::<String>::min_length(5), Rule::MinLength(5));
    assert_eq!(Rule::<String>::max_length(10), Rule::MaxLength(10));
    assert_eq!(Rule::<i32>::min(0), Rule::Min(0));
    assert_eq!(Rule::<i32>::max(100), Rule::Max(100));
    assert_eq!(Rule::<i32>::range(0, 100), Rule::Range { min: 0, max: 100 });
  }

  // ========================================================================
  // WithMessage Tests
  // ========================================================================

  #[test]
  fn test_rule_with_message_static() {
    let rule = Rule::<String>::MinLength(8).with_message("Password too short");

    match rule {
      Rule::WithMessage {
        rule: inner,
        message,
      } => {
        assert_eq!(*inner, Rule::MinLength(8));
        assert_eq!(message, Message::from("Password too short"));
      }
      _ => panic!("Expected Rule::WithMessage"),
    }
  }

  #[test]
  fn test_rule_with_message_provider() {
    let rule =
      Rule::<i32>::Min(0).with_message_provider(|ctx| format!("Got {}, expected >= 0", ctx.value));

    match rule {
      Rule::WithMessage {
        rule: inner,
        message,
      } => {
        assert_eq!(*inner, Rule::Min(0));
        assert!(message.is_provider());
        assert_eq!(message.resolve(&-5), "Got -5, expected >= 0");
      }
      _ => panic!("Expected Rule::WithMessage"),
    }
  }

  #[test]
  fn test_rule_with_message_equality() {
    let a = Rule::<String>::MinLength(5).with_message("error");
    let b = Rule::<String>::MinLength(5).with_message("error");
    let c = Rule::<String>::MinLength(5).with_message("different");

    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn test_rule_with_message_debug() {
    let rule = Rule::<String>::Required.with_message("Field is required");
    let debug_str = format!("{:?}", rule);

    assert!(debug_str.contains("WithMessage"));
    assert!(debug_str.contains("Required"));
    assert!(debug_str.contains("Field is required"));
  }

  #[test]
  fn test_rule_with_message_chained() {
    // You can chain with_message after combinators
    let rule = Rule::<String>::MinLength(3)
      .and(Rule::MaxLength(10))
      .with_message("Length must be between 3 and 10");

    match rule {
      Rule::WithMessage {
        rule: inner,
        message,
      } => {
        match *inner {
          Rule::All(rules) => assert_eq!(rules.len(), 2),
          _ => panic!("Expected Rule::All inside WithMessage"),
        }
        assert_eq!(
          message.resolve(&"".to_string()),
          "Length must be between 3 and 10"
        );
      }
      _ => panic!("Expected Rule::WithMessage"),
    }
  }

  // ========================================================================
  // String Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_ref_required() {
    let rule = Rule::<String>::Required;
    assert!(rule.validate_ref("hello").is_ok());
    assert!(rule.validate_ref("").is_err());
    assert!(rule.validate_ref("   ").is_err());
  }

  #[test]
  fn test_validate_ref_min_length() {
    let rule = Rule::<String>::MinLength(3);
    assert!(rule.validate_ref("hello").is_ok());
    assert!(rule.validate_ref("abc").is_ok());
    assert!(rule.validate_ref("ab").is_err());
    assert!(rule.validate_ref("").is_err());
  }

  #[test]
  fn test_validate_ref_max_length() {
    let rule = Rule::<String>::MaxLength(5);
    assert!(rule.validate_ref("hello").is_ok());
    assert!(rule.validate_ref("hi").is_ok());
    assert!(rule.validate_ref("").is_ok());
    assert!(rule.validate_ref("hello!").is_err());
  }

  #[test]
  fn test_validate_ref_exact_length() {
    let rule = Rule::<String>::ExactLength(5);
    assert!(rule.validate_ref("hello").is_ok());
    assert!(rule.validate_ref("hi").is_err());
    assert!(rule.validate_ref("hello!").is_err());
  }

  #[test]
  fn test_validate_ref_pattern() {
    let rule = Rule::<String>::Pattern(r"^\d+$".to_string());
    assert!(rule.validate_ref("123").is_ok());
    assert!(rule.validate_ref("abc").is_err());
    assert!(rule.validate_ref("12a").is_err());
  }

  #[test]
  fn test_validate_ref_email() {
    let rule = Rule::<String>::Email;
    assert!(rule.validate_ref("user@example.com").is_ok());
    assert!(rule.validate_ref("user@sub.example.com").is_ok());
    assert!(rule.validate_ref("invalid").is_err());
    assert!(rule.validate_ref("@example.com").is_err());
  }

  #[test]
  fn test_validate_ref_url() {
    let rule = Rule::<String>::Url;
    assert!(rule.validate_ref("http://example.com").is_ok());
    assert!(rule.validate_ref("https://example.com/path").is_ok());
    assert!(rule.validate_ref("not-a-url").is_err());
    assert!(rule.validate_ref("ftp://example.com").is_err()); // Only http/https
  }

  #[test]
  fn test_validate_ref_equals() {
    let rule = Rule::<String>::Equals("secret".to_string());
    assert!(rule.validate_ref("secret").is_ok());
    assert!(rule.validate_ref("wrong").is_err());
  }

  #[test]
  fn test_validate_ref_one_of() {
    let rule = Rule::<String>::OneOf(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert!(rule.validate_ref("a").is_ok());
    assert!(rule.validate_ref("b").is_ok());
    assert!(rule.validate_ref("d").is_err());
  }

  #[test]
  fn test_validate_ref_all() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
    assert!(rule.validate_ref("hello").is_ok());
    assert!(rule.validate_ref("hi").is_err());
    assert!(rule.validate_ref("hello world!").is_err());
  }

  #[test]
  fn test_validate_ref_any() {
    let rule = Rule::<String>::Email.or(Rule::Url);
    assert!(rule.validate_ref("user@example.com").is_ok());
    assert!(rule.validate_ref("http://example.com").is_ok());
    assert!(rule.validate_ref("neither").is_err());
  }

  #[test]
  fn test_validate_ref_not() {
    let rule = Rule::<String>::MinLength(5).not();
    assert!(rule.validate_ref("hi").is_ok()); // Less than 5 chars, so NOT passes
    assert!(rule.validate_ref("hello").is_err()); // 5 chars, so NOT fails
  }

  #[test]
  fn test_validate_ref_when() {
    let rule = Rule::<String>::When {
      condition: Condition::IsNotEmpty,
      then_rule: Box::new(Rule::MinLength(5)),
      else_rule: None,
    };
    assert!(rule.validate_ref("").is_ok()); // Empty, condition false, no rule applied
    assert!(rule.validate_ref("hello").is_ok()); // Not empty, MinLength(5) passes
    assert!(rule.validate_ref("hi").is_err()); // Not empty, MinLength(5) fails
  }

  #[test]
  fn test_validate_ref_with_message() {
    let rule = Rule::<String>::MinLength(8).with_message("Password too short");

    let result = rule.validate_ref("hi");
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(violation.message(), "Password too short");
  }

  #[test]
  fn test_validate_ref_all_violations() {
    let rule = Rule::<String>::MinLength(3)
      .and(Rule::MaxLength(5))
      .and(Rule::Pattern(r"^\d+$".to_string()));

    // Valid
    assert!(rule.validate_ref_all("123").is_ok());

    // Multiple violations
    let result = rule.validate_ref_all("ab");
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert!(violations.len() >= 1); // At least TooShort
  }

  // ========================================================================
  // Numeric Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_min() {
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate(0).is_ok());
    assert!(rule.validate(100).is_ok());
    assert!(rule.validate(-1).is_err());
  }

  #[test]
  fn test_validate_max() {
    let rule = Rule::<i32>::Max(100);
    assert!(rule.validate(100).is_ok());
    assert!(rule.validate(0).is_ok());
    assert!(rule.validate(101).is_err());
  }

  #[test]
  fn test_validate_range() {
    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    assert!(rule.validate(0).is_ok());
    assert!(rule.validate(50).is_ok());
    assert!(rule.validate(100).is_ok());
    assert!(rule.validate(-1).is_err());
    assert!(rule.validate(101).is_err());
  }

  #[test]
  fn test_validate_step() {
    let rule = Rule::<i32>::Step(5);
    assert!(rule.validate(0).is_ok());
    assert!(rule.validate(5).is_ok());
    assert!(rule.validate(10).is_ok());
    assert!(rule.validate(3).is_err());
  }

  #[test]
  fn test_validate_step_float() {
    let rule = Rule::<f64>::Step(0.5);
    assert!(rule.validate(0.0).is_ok());
    assert!(rule.validate(0.5).is_ok());
    assert!(rule.validate(1.0).is_ok());
    assert!(rule.validate(0.3).is_err());
  }

  #[test]
  fn test_validate_equals_numeric() {
    let rule = Rule::<i32>::Equals(42);
    assert!(rule.validate(42).is_ok());
    assert!(rule.validate(0).is_err());
  }

  #[test]
  fn test_validate_one_of_numeric() {
    let rule = Rule::<i32>::OneOf(vec![1, 2, 3]);
    assert!(rule.validate(1).is_ok());
    assert!(rule.validate(2).is_ok());
    assert!(rule.validate(4).is_err());
  }

  #[test]
  fn test_validate_all_numeric() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100)).and(Rule::Step(10));
    assert!(rule.validate(50).is_ok());
    assert!(rule.validate(55).is_err()); // Not step of 10
    assert!(rule.validate(-10).is_err()); // Below min
  }

  #[test]
  fn test_validate_any_numeric() {
    let rule = Rule::<i32>::Equals(0).or(Rule::Equals(100));
    assert!(rule.validate(0).is_ok());
    assert!(rule.validate(100).is_ok());
    assert!(rule.validate(50).is_err());
  }

  #[test]
  fn test_validate_not_numeric() {
    let rule = Rule::<i32>::Min(0).not();
    assert!(rule.validate(-1).is_ok()); // Below 0, so NOT passes
    assert!(rule.validate(0).is_err()); // At 0, Min passes, so NOT fails
  }

  #[test]
  fn test_validate_with_message_numeric() {
    let rule = Rule::<i32>::Min(0).with_message("Must be non-negative");

    let result = rule.validate(-5);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(violation.message(), "Must be non-negative");
  }

  #[test]
  fn test_validate_all_numeric_multiple() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(10)).and(Rule::Step(3));

    // Valid
    assert!(rule.validate_all(6).is_ok());

    // Multiple violations: 15 is > 10 and not step of 3
    let result = rule.validate_all(15);
    assert!(result.is_err());
  }

  // ========================================================================
  // Condition Evaluation Tests
  // ========================================================================

  #[test]
  fn test_condition_is_empty() {
    let cond = Condition::<String>::IsEmpty;
    assert!(cond.evaluate_str(""));
    assert!(cond.evaluate_str("   "));
    assert!(!cond.evaluate_str("hello"));
  }

  #[test]
  fn test_condition_is_not_empty() {
    let cond = Condition::<String>::IsNotEmpty;
    assert!(!cond.evaluate_str(""));
    assert!(!cond.evaluate_str("   "));
    assert!(cond.evaluate_str("hello"));
  }

  #[test]
  fn test_condition_equals_str() {
    let cond = Condition::<String>::Equals("test".to_string());
    assert!(cond.evaluate_str("test"));
    assert!(!cond.evaluate_str("other"));
  }

  #[test]
  fn test_condition_matches() {
    let cond = Condition::<String>::Matches(r"^\d+$".to_string());
    assert!(cond.evaluate_str("123"));
    assert!(!cond.evaluate_str("abc"));
  }

  #[test]
  fn test_condition_evaluate_numeric() {
    let gt = Condition::<i32>::GreaterThan(10);
    assert!(gt.evaluate(&15));
    assert!(!gt.evaluate(&5));

    let lt = Condition::<i32>::LessThan(10);
    assert!(lt.evaluate(&5));
    assert!(!lt.evaluate(&15));

    let eq = Condition::<i32>::Equals(42);
    assert!(eq.evaluate(&42));
    assert!(!eq.evaluate(&0));
  }

  // ========================================================================
  // CompiledRule Tests
  // ========================================================================

  #[test]
  fn test_compiled_rule_string_basic() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
    let compiled = rule.compile();

    assert!(compiled.validate_ref("hello").is_ok());
    assert!(compiled.validate_ref("hi").is_err());
    assert!(compiled.validate_ref("hello world!").is_err());
  }

  #[test]
  fn test_compiled_rule_pattern_cached() {
    let rule = Rule::<String>::Pattern(r"^\d{3}-\d{4}$".to_string());
    let compiled = rule.compile();

    // Multiple calls reuse cached regex
    assert!(compiled.validate_ref("123-4567").is_ok());
    assert!(compiled.validate_ref("999-0000").is_ok());
    assert!(compiled.validate_ref("abc-defg").is_err());
    assert!(compiled.validate_ref("12-345").is_err());
  }

  #[test]
  fn test_compiled_rule_email() {
    let rule = Rule::<String>::Email;
    let compiled = rule.compile();

    assert!(compiled.validate_ref("user@example.com").is_ok());
    assert!(compiled.validate_ref("test@sub.domain.org").is_ok());
    assert!(compiled.validate_ref("invalid").is_err());
  }

  #[test]
  fn test_compiled_rule_url() {
    let rule = Rule::<String>::Url;
    let compiled = rule.compile();

    assert!(compiled.validate_ref("http://example.com").is_ok());
    assert!(
      compiled
        .validate_ref("https://example.com/path?query=1")
        .is_ok()
    );
    assert!(compiled.validate_ref("not-a-url").is_err());
  }

  #[test]
  fn test_compiled_rule_numeric() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    let compiled = rule.compile();

    assert!(compiled.validate(50).is_ok());
    assert!(compiled.validate(0).is_ok());
    assert!(compiled.validate(100).is_ok());
    assert!(compiled.validate(-1).is_err());
    assert!(compiled.validate(101).is_err());
  }

  #[test]
  fn test_compiled_rule_clone() {
    let rule = Rule::<String>::Pattern(r"^\w+$".to_string());
    let compiled = rule.compile();

    // Validate to initialize cache
    assert!(compiled.validate_ref("hello").is_ok());

    // Clone should work (cache is reset)
    let cloned = compiled.clone();
    assert!(cloned.validate_ref("world").is_ok());
  }

  #[test]
  fn test_compiled_rule_debug() {
    let rule = Rule::<String>::MinLength(5);
    let compiled = rule.compile();
    let debug_str = format!("{:?}", compiled);
    assert!(debug_str.contains("CompiledRule"));
    assert!(debug_str.contains("MinLength"));
  }

  #[test]
  fn test_compiled_rule_into_rule() {
    let rule = Rule::<String>::Required;
    let compiled = rule.clone().compile();
    let recovered = compiled.into_rule();
    assert_eq!(recovered, rule);
  }

  #[test]
  fn test_compiled_rule_with_trait() {
    use crate::ValidateRef;

    let rule = Rule::<String>::MinLength(3);
    let compiled = rule.compile();

    // Can use via trait
    let validator: &dyn ValidateRef<str> = &compiled;
    assert!(validator.validate_ref("hello").is_ok());
    assert!(validator.validate_ref("hi").is_err());
  }

  #[test]
  fn test_compiled_rule_validate_all() {
    let rule = Rule::<String>::MinLength(3)
      .and(Rule::MaxLength(5))
      .and(Rule::Pattern(r"^[a-z]+$".to_string()));
    let compiled = rule.compile();

    assert!(compiled.validate_ref_all("abc").is_ok());

    let result = compiled.validate_ref_all("AB");
    assert!(result.is_err());
  }

  // ========================================================================
  // Option Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_ref_option_none_non_required() {
    // None is OK for non-required rules
    let rule = Rule::<String>::MinLength(3);
    assert!(rule.validate_ref_option(None).is_ok());

    let rule = Rule::<String>::Pattern(r"^\d+$".to_string());
    assert!(rule.validate_ref_option(None).is_ok());

    let rule = Rule::<String>::Email;
    assert!(rule.validate_ref_option(None).is_ok());
  }

  #[test]
  fn test_validate_ref_option_none_required() {
    // None fails for Required rule
    let rule = Rule::<String>::Required;
    assert!(rule.validate_ref_option(None).is_err());

    let violation = rule.validate_ref_option(None).unwrap_err();
    assert_eq!(
      violation.violation_type(),
      crate::ViolationType::ValueMissing
    );
  }

  #[test]
  fn test_validate_ref_option_none_all_with_required() {
    // None fails if All contains Required
    let rule = Rule::<String>::Required.and(Rule::MinLength(3));
    assert!(rule.validate_ref_option(None).is_err());
  }

  #[test]
  fn test_validate_ref_option_none_all_without_required() {
    // None is OK if All doesn't contain Required
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(10));
    assert!(rule.validate_ref_option(None).is_ok());
  }

  #[test]
  fn test_validate_ref_option_some_valid() {
    let rule = Rule::<String>::MinLength(3);
    assert!(rule.validate_ref_option(Some("hello")).is_ok());
  }

  #[test]
  fn test_validate_ref_option_some_invalid() {
    let rule = Rule::<String>::MinLength(5);
    assert!(rule.validate_ref_option(Some("hi")).is_err());
  }

  #[test]
  fn test_validate_ref_option_all() {
    let rule = Rule::<String>::Required.and(Rule::MinLength(3));

    // None with Required in All
    let result = rule.validate_ref_option_all(None);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1);

    // Some valid
    assert!(rule.validate_ref_option_all(Some("hello")).is_ok());

    // Some invalid
    assert!(rule.validate_ref_option_all(Some("hi")).is_err());
  }

  #[test]
  fn test_validate_option_numeric_none() {
    // None is an error for numeric rules
    let rule = Rule::<i32>::Min(0);
    assert!(rule.validate_option(None).is_err());

    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    assert!(rule.validate_option(None).is_err());

    let rule = Rule::<f64>::Step(0.5);
    assert!(rule.validate_option(None).is_err());
  }

  #[test]
  fn test_validate_option_numeric_some_valid() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_option(Some(50)).is_ok());
  }

  #[test]
  fn test_validate_option_numeric_some_invalid() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100));
    assert!(rule.validate_option(Some(-5)).is_err());
    assert!(rule.validate_option(Some(150)).is_err());
  }

  #[test]
  fn test_validate_option_all_numeric() {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(100)).and(Rule::Step(10));

    // None is an error
    assert!(rule.validate_option_all(None).is_err());

    // Some valid
    assert!(rule.validate_option_all(Some(50)).is_ok());

    // Some invalid
    let result = rule.validate_option_all(Some(55));
    assert!(result.is_err());
  }

  // ========================================================================
  // Collection Length Validation Tests
  // ========================================================================

  #[test]
  fn test_validate_len_ref_min_length() {
    let rule = Rule::<Vec<i32>>::MinLength(2);
    assert!(rule.validate_len_ref(&vec![1, 2]).is_ok());
    assert!(rule.validate_len_ref(&vec![1, 2, 3]).is_ok());
    assert!(rule.validate_len_ref(&vec![1]).is_err());
    assert!(rule.validate_len_ref(&vec![]).is_err());
  }

  #[test]
  fn test_validate_len_ref_max_length() {
    let rule = Rule::<Vec<i32>>::MaxLength(3);
    assert!(rule.validate_len_ref(&vec![1]).is_ok());
    assert!(rule.validate_len_ref(&vec![1, 2, 3]).is_ok());
    assert!(rule.validate_len_ref(&vec![1, 2, 3, 4]).is_err());
  }

  #[test]
  fn test_validate_len_ref_exact_length() {
    let rule = Rule::<Vec<i32>>::ExactLength(3);
    assert!(rule.validate_len_ref(&vec![1, 2, 3]).is_ok());
    assert!(rule.validate_len_ref(&vec![1, 2]).is_err());
    assert!(rule.validate_len_ref(&vec![1, 2, 3, 4]).is_err());
  }

  #[test]
  fn test_validate_len_ref_required() {
    let rule = Rule::<Vec<i32>>::Required;
    assert!(rule.validate_len_ref(&vec![1]).is_ok());
    assert!(rule.validate_len_ref(&vec![]).is_err());
  }

  #[test]
  fn test_validate_len_ref_all_combinator() {
    let rule = Rule::<Vec<i32>>::MinLength(2).and(Rule::MaxLength(5));
    assert!(rule.validate_len_ref(&vec![1, 2]).is_ok());
    assert!(rule.validate_len_ref(&vec![1, 2, 3, 4, 5]).is_ok());
    assert!(rule.validate_len_ref(&vec![1]).is_err());
    assert!(rule.validate_len_ref(&vec![1, 2, 3, 4, 5, 6]).is_err());
  }

  #[test]
  fn test_validate_len_ref_any_combinator() {
    // Either exactly 2 items OR exactly 5 items
    let rule = Rule::<Vec<i32>>::ExactLength(2).or(Rule::ExactLength(5));
    assert!(rule.validate_len_ref(&vec![1, 2]).is_ok());
    assert!(rule.validate_len_ref(&vec![1, 2, 3, 4, 5]).is_ok());
    assert!(rule.validate_len_ref(&vec![1, 2, 3]).is_err());
  }

  #[test]
  fn test_validate_len_ref_not_combinator() {
    // NOT empty (must have at least 1 item)
    let rule = Rule::<Vec<i32>>::MaxLength(0).not();
    assert!(rule.validate_len_ref(&vec![1]).is_ok());
    assert!(rule.validate_len_ref(&vec![]).is_err());
  }

  // Note: Slice validation ([T]) is not supported because Rule<T> requires T: Sized.
  // Use Vec<T> or other sized collection types instead.
  // For slice validation, use LengthValidator<[T]> directly.

  #[test]
  fn test_validate_len_ref_hashmap() {
    use std::collections::HashMap;

    let rule = Rule::<HashMap<String, i32>>::MinLength(1).and(Rule::MaxLength(3));

    let mut map = HashMap::new();
    map.insert("a".to_string(), 1);
    assert!(rule.validate_len_ref(&map).is_ok());

    map.insert("b".to_string(), 2);
    map.insert("c".to_string(), 3);
    assert!(rule.validate_len_ref(&map).is_ok());

    map.insert("d".to_string(), 4);
    assert!(rule.validate_len_ref(&map).is_err());

    let empty_map: HashMap<String, i32> = HashMap::new();
    assert!(rule.validate_len_ref(&empty_map).is_err());
  }

  #[test]
  fn test_validate_len_ref_all_violations() {
    let rule = Rule::<Vec<i32>>::MinLength(3).and(Rule::MaxLength(2));
    // This is contradictory - will always fail

    let result = rule.validate_len_ref_all(&vec![1, 2]);
    assert!(result.is_err());
    let violations = result.unwrap_err();
    assert_eq!(violations.len(), 1); // MinLength fails
  }

  #[test]
  fn test_validate_len_ref_option_none_not_required() {
    let rule = Rule::<Vec<i32>>::MinLength(2);
    assert!(rule.validate_len_ref_option(None).is_ok());
  }

  #[test]
  fn test_validate_len_ref_option_none_required() {
    let rule = Rule::<Vec<i32>>::Required;
    assert!(rule.validate_len_ref_option(None).is_err());
  }

  #[test]
  fn test_validate_len_ref_option_some_valid() {
    let rule = Rule::<Vec<i32>>::MinLength(2);
    assert!(rule.validate_len_ref_option(Some(&vec![1, 2, 3])).is_ok());
  }

  #[test]
  fn test_validate_len_ref_option_some_invalid() {
    let rule = Rule::<Vec<i32>>::MinLength(2);
    assert!(rule.validate_len_ref_option(Some(&vec![1])).is_err());
  }

  #[test]
  fn test_validate_len_ref_option_all_with_required() {
    let rule = Rule::<Vec<i32>>::Required.and(Rule::MinLength(2));

    // None fails for Required
    assert!(rule.validate_len_ref_option(None).is_err());

    // Some valid
    assert!(rule.validate_len_ref_option(Some(&vec![1, 2])).is_ok());

    // Some invalid
    assert!(rule.validate_len_ref_option(Some(&vec![1])).is_err());
  }

  #[test]
  fn test_validate_len_ref_violation_messages() {
    let rule = Rule::<Vec<i32>>::MinLength(3);
    let result = rule.validate_len_ref(&vec![1]);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(
      violation.message(),
      "Collection must have at least 3 items (got 1)"
    );

    let rule = Rule::<Vec<i32>>::MaxLength(2);
    let result = rule.validate_len_ref(&vec![1, 2, 3, 4]);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(
      violation.message(),
      "Collection must have at most 2 items (got 4)"
    );

    let rule = Rule::<Vec<i32>>::ExactLength(3);
    let result = rule.validate_len_ref(&vec![1, 2]);
    assert!(result.is_err());
    let violation = result.unwrap_err();
    assert_eq!(
      violation.message(),
      "Collection must have exactly 3 items (got 2)"
    );
  }

  // ==========================================================================
  // ToAttributesList Tests
  // ==========================================================================

  #[test]
  fn test_to_attributes_list_required() {
    let rule = Rule::<String>::Required;
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "required");
    assert_eq!(attrs[0].1, serde_json::Value::Bool(true));
  }

  #[test]
  fn test_to_attributes_list_min_length() {
    let rule = Rule::<String>::MinLength(3);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "minlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(3));
  }

  #[test]
  fn test_to_attributes_list_max_length() {
    let rule = Rule::<String>::MaxLength(50);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "maxlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(50));
  }

  #[test]
  fn test_to_attributes_list_exact_length() {
    let rule = Rule::<String>::ExactLength(10);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].0, "minlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(10));
    assert_eq!(attrs[1].0, "maxlength");
    assert_eq!(attrs[1].1, serde_json::Value::from(10));
  }

  #[test]
  fn test_to_attributes_list_pattern() {
    let rule = Rule::<String>::Pattern(r"^\w+$".to_string());
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "pattern");
    assert_eq!(attrs[0].1, serde_json::Value::from(r"^\w+$"));
  }

  #[test]
  fn test_to_attributes_list_email() {
    let rule = Rule::<String>::Email;
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "type");
    assert_eq!(attrs[0].1, serde_json::Value::from("email"));
  }

  #[test]
  fn test_to_attributes_list_url() {
    let rule = Rule::<String>::Url;
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "type");
    assert_eq!(attrs[0].1, serde_json::Value::from("url"));
  }

  #[test]
  fn test_to_attributes_list_min() {
    let rule = Rule::<i32>::Min(0);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "min");
    assert_eq!(attrs[0].1, serde_json::Value::from(0));
  }

  #[test]
  fn test_to_attributes_list_max() {
    let rule = Rule::<i32>::Max(100);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "max");
    assert_eq!(attrs[0].1, serde_json::Value::from(100));
  }

  #[test]
  fn test_to_attributes_list_range() {
    let rule = Rule::<i32>::Range { min: 0, max: 100 };
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].0, "min");
    assert_eq!(attrs[0].1, serde_json::Value::from(0));
    assert_eq!(attrs[1].0, "max");
    assert_eq!(attrs[1].1, serde_json::Value::from(100));
  }

  #[test]
  fn test_to_attributes_list_step() {
    let rule = Rule::<i32>::Step(5);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "step");
    assert_eq!(attrs[0].1, serde_json::Value::from(5));
  }

  #[test]
  fn test_to_attributes_list_all_composite() {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(50));
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].0, "minlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(3));
    assert_eq!(attrs[1].0, "maxlength");
    assert_eq!(attrs[1].1, serde_json::Value::from(50));
  }

  #[test]
  fn test_to_attributes_list_any_composite() {
    let rule = Rule::<String>::Email.or(Rule::Url);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 2);
    // Both type attributes are collected (though HTML would only use one)
    assert!(attrs.iter().any(|(k, v)| k == "type" && v == "email"));
    assert!(attrs.iter().any(|(k, v)| k == "type" && v == "url"));
  }

  #[test]
  fn test_to_attributes_list_complex_composite() {
    let rule = Rule::<String>::Required
      .and(Rule::MinLength(5))
      .and(Rule::MaxLength(100))
      .and(Rule::Pattern(r"^\w+$".to_string()));
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 4);
    assert!(attrs.iter().any(|(k, _)| k == "required"));
    assert!(attrs.iter().any(|(k, _)| k == "minlength"));
    assert!(attrs.iter().any(|(k, _)| k == "maxlength"));
    assert!(attrs.iter().any(|(k, _)| k == "pattern"));
  }

  #[test]
  fn test_to_attributes_list_non_mappable_returns_none() {
    // Equals doesn't map to HTML attributes
    let rule = Rule::<String>::Equals("test".to_string());
    assert!(rule.to_attributes_list().is_none());

    // OneOf doesn't map to HTML attributes
    let rule = Rule::<String>::OneOf(vec!["a".to_string(), "b".to_string()]);
    assert!(rule.to_attributes_list().is_none());

    // Not doesn't map to HTML attributes
    let rule = Rule::<String>::MinLength(3).not();
    assert!(rule.to_attributes_list().is_none());

    // When doesn't map to HTML attributes
    let rule = Rule::<String>::When {
      condition: Condition::IsNotEmpty,
      then_rule: Box::new(Rule::MinLength(3)),
      else_rule: None,
    };
    assert!(rule.to_attributes_list().is_none());
  }

  #[test]
  fn test_to_attributes_list_with_message_delegates() {
    let inner_rule = Rule::<String>::MinLength(5);
    let rule = Rule::WithMessage {
      rule: Box::new(inner_rule),
      message: Message::Static("Custom message".to_string()),
    };
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].0, "minlength");
    assert_eq!(attrs[0].1, serde_json::Value::from(5));
  }

  #[test]
  fn test_to_attributes_list_empty_all_returns_none() {
    // All with only non-mappable rules
    let rule = Rule::<String>::All(vec![Rule::Equals("test".to_string())]);
    assert!(rule.to_attributes_list().is_none());
  }

  #[test]
  fn test_to_attributes_list_numeric_types() {
    // Test with f64
    let rule = Rule::<f64>::Min(0.5);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs[0].0, "min");
    assert_eq!(attrs[0].1, serde_json::Value::from(0.5));

    // Test Range with f64
    let rule = Rule::<f64>::Range { min: 0.0, max: 1.0 };
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs.len(), 2);
    assert_eq!(attrs[0].1, serde_json::Value::from(0.0));
    assert_eq!(attrs[1].1, serde_json::Value::from(1.0));

    // Test Step with f64
    let rule = Rule::<f64>::Step(0.1);
    let attrs = rule.to_attributes_list().unwrap();
    assert_eq!(attrs[0].0, "step");
    assert_eq!(attrs[0].1, serde_json::Value::from(0.1));
  }
}

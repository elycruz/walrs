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
//! use walrs_validation::rule::{Rule, Condition};
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
use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::{Message, MessageContext, SteppableValue, Violation};
use crate::traits::IsEmpty;

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

  /// Wraps another rule with a custom error message and optional locale.
  ///
  /// When the inner rule fails, the custom message is used instead of
  /// the default message. The optional locale is passed to the message
  /// provider for internationalization support.
  #[serde(skip)]
  WithMessage {
    /// The wrapped rule
    rule: Box<Rule<T>>,
    /// The custom message to use on failure
    message: Message<T>,
    /// Optional locale for i18n support (e.g., "es", "en-US", "fr", etc.)
    locale: Option<String>,
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
      Self::WithMessage { rule, message, locale } => f
        .debug_struct("WithMessage")
        .field("rule", rule)
        .field("message", message)
        .field("locale", locale)
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
          locale: l1,
        },
        Self::WithMessage {
          rule: r2,
          message: m2,
          locale: l2,
        },
      ) => r1 == r2 && m1 == m2 && l1 == l2,
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
  /// use walrs_validation::rule::Rule;
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
  /// use walrs_validation::rule::Rule;
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
  /// use walrs_validation::rule::Rule;
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
  /// use walrs_validation::rule::{Rule, Condition};
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
  /// use walrs_validation::rule::{Rule, Condition};
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
  /// use walrs_validation::rule::Rule;
  /// use walrs_validation::{Violation, ViolationType};
  ///
  /// let is_even = Rule::<i32>::custom(Arc::new(|value: &i32| {
  ///     if value % 2 == 0 {
  ///         Ok(())
  ///     } else {
  ///         Err(Violation::new(ViolationType::CustomError, "Value must be even."))
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
  /// use walrs_validation::rule::Rule;
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
  /// use walrs_validation::rule::Rule;
  ///
  /// let rule = Rule::<String>::MinLength(8)
  ///     .with_message("Password must be at least 8 characters.");
  /// ```
  pub fn with_message(self, msg: impl Into<String>) -> Rule<T> {
    Rule::WithMessage {
      rule: Box::new(self),
      message: Message::Static(msg.into()),
      locale: None,
    }
  }

  /// Attaches a dynamic message provider to this rule.
  ///
  /// The closure receives a `MessageContext` containing the value being validated
  /// and rule parameters, enabling rich interpolated error messages.
  /// An optional `locale` can be provided for internationalization support.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validation::rule::Rule;
  ///
  /// let rule = Rule::<i32>::Min(0)
  ///     .with_message_provider(|ctx| format!("Value {} must be non-negative.", ctx.value), None);
  ///
  /// let rule_es = Rule::<String>::MinLength(3)
  ///     .with_message_provider(|ctx| {
  ///         match ctx.locale {
  ///             Some("es") => format!("Mínimo 3 caracteres"),
  ///             _ => format!("Minimum 3 characters"),
  ///         }
  ///     }, Some("es"));
  /// ```
  pub fn with_message_provider<F>(self, f: F, locale: Option<&str>) -> Rule<T>
  where
    F: Fn(&MessageContext<T>) -> String + Send + Sync + 'static,
  {
    Rule::WithMessage {
      rule: Box::new(self),
      message: Message::Provider(Arc::new(f)),
      locale: locale.map(String::from),
    }
  }

  /// Attaches a locale to this rule for internationalized error messages.
  ///
  /// If this rule is already a `WithMessage` variant, updates its locale.
  /// Otherwise wraps the rule in a `WithMessage` with an empty static message
  /// (which will pass through the inner rule's violation message) and the given locale.
  ///
  /// # Example
  ///
  /// ```rust
  /// use walrs_validation::rule::Rule;
  ///
  /// let rule = Rule::<String>::MinLength(3)
  ///     .with_message_provider(|ctx| {
  ///         match ctx.locale {
  ///             Some("es") => format!("Mínimo 3 caracteres"),
  ///             _ => format!("Minimum 3 characters"),
  ///         }
  ///     }, None)
  ///     .with_locale("es");
  /// ```
  pub fn with_locale(self, locale: impl Into<String>) -> Rule<T> {
    let locale_str = locale.into();
    match self {
      Rule::WithMessage { rule, message, .. } => Rule::WithMessage {
        rule,
        message,
        locale: Some(locale_str),
      },
      other => Rule::WithMessage {
        rule: Box::new(other),
        message: Message::Static(String::new()),
        locale: Some(locale_str),
      },
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

// Rule<String> implementation moved to impls/string.rs

// Rule<Numeric> implementation moved to impls/step

// ============================================================================
// CompiledRule - Cached Validator Wrapper
// ============================================================================

use std::sync::OnceLock;
use crate::impls::string::CachedStringValidators;

/// A compiled rule with cached validators for better performance.
///
/// Use `CompiledRule` when you need to validate many values against the same rule.
/// The compiled form caches regex patterns and other validators to avoid
/// repeated construction.
///
/// # Example
///
/// ```rust
/// use walrs_validation::rule::Rule;
///
/// // Define and compile rule once
/// let rule = Rule::<String>::MinLength(8)
///     .and(Rule::Pattern(r"[A-Z]".to_string()));
/// let compiled = rule.compile();
///
/// // Validate many times (reuses cached regex)
/// assert!(compiled.validate_str("Password1").is_ok());
/// assert!(compiled.validate_str("short").is_err());
/// ```
pub struct CompiledRule<T> {
  /// The underlying rule
  pub(crate) rule: Rule<T>,
  /// Cached string validators (lazily initialized)
  pub(crate) string_cache: OnceLock<CachedStringValidators>,
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
  /// use walrs_validation::rule::Rule;
  ///
  /// let rule = Rule::<String>::Pattern(r"^\d+$".to_string());
  /// let compiled = rule.compile();
  ///
  /// // Repeated calls reuse the cached regex
  /// assert!(compiled.validate_str("123").is_ok());
  /// assert!(compiled.validate_str("456").is_ok());
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

// Trait implementations moved to impls modules

// ToAttributesList implementation moved to impls/attributes.rs

// Rule<WithLength> implementation moved to impls/length.rs

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
    let rule = Rule::<String>::MinLength(8).with_message("Password too short.");

    match rule {
      Rule::WithMessage {
        rule: inner,
        message,
        locale,
      } => {
        assert_eq!(*inner, Rule::MinLength(8));
        assert_eq!(message, Message::from("Password too short."));
        assert_eq!(locale, None);
      }
      _ => panic!("Expected Rule::WithMessage"),
    }
  }

  #[test]
  fn test_rule_with_message_provider() {
    let rule =
      Rule::<i32>::Min(0).with_message_provider(|ctx| format!("Got {}, expected >= 0.", ctx.value), None);

    match rule {
      Rule::WithMessage {
        rule: inner,
        message,
        locale,
      } => {
        assert_eq!(*inner, Rule::Min(0));
        assert!(message.is_provider());
        assert_eq!(message.resolve(&-5, None), "Got -5, expected >= 0.");
        assert_eq!(locale, None);
      }
      _ => panic!("Expected Rule::WithMessage"),
    }
  }

  #[test]
  fn test_rule_with_message_equality() {
    let a = Rule::<String>::MinLength(5).with_message("error.");
    let b = Rule::<String>::MinLength(5).with_message("error.");
    let c = Rule::<String>::MinLength(5).with_message("different.");

    assert_eq!(a, b);
    assert_ne!(a, c);
  }

  #[test]
  fn test_rule_with_message_debug() {
    let rule = Rule::<String>::Required.with_message("Field is required.");
    let debug_str = format!("{:?}", rule);

    assert!(debug_str.contains("WithMessage"));
    assert!(debug_str.contains("Required"));
    assert!(debug_str.contains("Field is required."));
  }

  #[test]
  fn test_rule_with_message_chained() {
    // You can chain with_message after combinators
    let rule = Rule::<String>::MinLength(3)
      .and(Rule::MaxLength(10))
      .with_message("Length must be between 3 and 10.");

    match rule {
      Rule::WithMessage {
        rule: inner,
        message,
        locale,
      } => {
        match *inner {
          Rule::All(rules) => assert_eq!(rules.len(), 2),
          _ => panic!("Expected Rule::All inside WithMessage"),
        }
        assert_eq!(
          message.resolve(&"".to_string(), None),
          "Length must be between 3 and 10."
        );
        assert_eq!(locale, None);
      }
      _ => panic!("Expected Rule::WithMessage"),
    }
  }

  // ========================================================================
  // String Validation Tests  →  see impls/string.rs
  // Numeric Validation Tests →  see impls/step
  // Collection Length Tests  →  see impls/length.rs
  // ToAttributesList Tests   →  see impls/attributes.rs
  // ========================================================================

  // ==========================================================================
  // e2e
  // ==========================================================================
  #[test]
  fn test_e2e_only_rule_and_validators() {
    let _slug = Rule::<String>::Pattern(r"(?i)^[\w\-]{1,108}$".to_string());
    let _screen_name = Rule::<String>::Pattern(r"(?i)^[a-z][\w\-]{7,55}$".to_string());
    let _numeric_id = Rule::<usize>::Range { min: 1, max: usize::MAX };

    // TODO: Complete this.
  }
}

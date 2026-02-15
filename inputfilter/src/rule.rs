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
//! use walrs_inputfilter::rule::{Rule, Condition};
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
//!     then_rules: vec![Rule::MinLength(5)],
//!     else_rules: None,
//! };
//! ```

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::sync::Arc;

use crate::Violation;

// ============================================================================
// Result Types
// ============================================================================

/// Result of applying a rule to a value.
pub type RuleResult = Result<(), Violation>;

// ============================================================================
// Message Enum
// ============================================================================

/// Parameters extracted from a rule for message interpolation.
///
/// When a rule fails validation, these parameters provide context about
/// the constraint that was violated, enabling dynamic error messages.
/// Each field corresponds to a variant or constraint from the `Rule` enum.
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
    /// Minimum value constraint (for `Min` or `Range` rules) converted to string
    /// to keep the API simple (`MessageParams` would require a generic otherwise).
    pub min: Option<String>,
    /// Maximum value constraint (for `Max` or `Range` rules) "".
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

/// Context passed to message providers during validation.
///
/// Contains both the value being validated and parameters from the rule,
/// enabling rich, contextual error messages.
///
/// # Example
///
/// ```rust
/// use walrs_inputfilter::rule::{Message, MessageContext, MessageParams};
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
pub struct MessageContext<'a, T> {
    /// The value being validated
    pub value: &'a T,
    /// Parameters extracted from the rule
    pub params: MessageParams,
}

impl<'a, T> MessageContext<'a, T> {
    /// Creates a new message context.
    pub fn new(value: &'a T, params: MessageParams) -> Self {
        Self { value, params }
    }
}

/// A validation error message that can be either a static string or a dynamic provider.
///
/// This enum enables:
/// - **Static messages**: Simple strings, serializable to JSON/YAML
/// - **Dynamic messages**: Closures that generate messages with context (value, rule params)
///
/// # Serialization
///
/// Only `Static` variants serialize. `Provider` is skipped and will deserialize
/// using the default (empty string), which can be detected and replaced with
/// a fallback message.
///
/// # Example
///
/// ```rust
/// use walrs_inputfilter::rule::Message;
///
/// // Static message
/// let msg: Message<String> = Message::from("Must be at least 8 characters");
///
/// // Dynamic message with value context
/// let msg: Message<i32> = Message::provider(|ctx| format!("Value {} is out of range", ctx.value));
/// ```
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message<T> {
    /// A static message string (serializable)
    Static(String),

    /// A dynamic message provider (not serializable)
    ///
    /// The closure receives a `MessageContext` containing the value being validated
    /// and parameters extracted from the rule, enabling rich interpolated messages.
    #[serde(skip)]
    Provider(Arc<dyn Fn(&MessageContext<T>) -> String + Send + Sync>),
}

impl<T> Message<T> {
    /// Creates a static message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use walrs_inputfilter::rule::Message;
    ///
    /// let msg: Message<String> = Message::static_msg("Invalid value");
    /// assert_eq!(msg.resolve(&"test".to_string()), "Invalid value");
    /// ```
    pub fn static_msg(msg: impl Into<String>) -> Self {
        Message::Static(msg.into())
    }

    /// Creates a dynamic message provider.
    ///
    /// # Example
    ///
    /// ```rust
    /// use walrs_inputfilter::rule::Message;
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
    /// use walrs_inputfilter::rule::Message;
    ///
    /// let static_msg: Message<String> = Message::from("Error");
    /// assert_eq!(static_msg.resolve(&"any".to_string()), "Error");
    ///
    /// let dynamic_msg: Message<String> = Message::provider(|ctx| format!("Bad: {}", ctx.value));
    /// assert_eq!(dynamic_msg.resolve(&"input".to_string()), "Bad: input");
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
    /// use walrs_inputfilter::rule::{Message, MessageContext, MessageParams};
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
    /// use walrs_inputfilter::rule::Message;
    ///
    /// let empty: Message<String> = Message::Static(String::new());
    /// assert_eq!(empty.resolve_or(&"x".to_string(), "default"), "default");
    ///
    /// let filled: Message<String> = Message::from("custom");
    /// assert_eq!(filled.resolve_or(&"x".to_string(), "default"), "custom");
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

impl<T> Debug for Message<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(s) => f.debug_tuple("Static").field(s).finish(),
            Self::Provider(_) => write!(f, "Provider(<fn>)"),
        }
    }
}

impl<T> PartialEq for Message<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(a), Self::Static(b)) => a == b,
            // Providers are never equal (can't compare closures)
            _ => false,
        }
    }
}

impl<T> Default for Message<T> {
    fn default() -> Self {
        Message::Static(String::new())
    }
}

// Convenience conversions
impl<T> From<&str> for Message<T> {
    fn from(s: &str) -> Self {
        Message::Static(s.to_string())
    }
}

impl<T> From<String> for Message<T> {
    fn from(s: String) -> Self {
        Message::Static(s)
    }
}

// ============================================================================
// Condition Enum
// ============================================================================

/// Conditions for `When` rules.
///
/// Conditions determine whether the `then_rules` or `else_rules` of a `When` rule
/// should be applied. Most variants are serializable for config-driven validation.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Rule<T> {
    // ---- Presence ----
    /// Value must be present (non-empty)
    Required,

    // ---- `WithLength` type ----
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
        /// Rules to apply if condition is true
        then_rules: Vec<Rule<T>>,
        /// Rules to apply if condition is false (optional)
        else_rules: Option<Vec<Rule<T>>>,
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
                then_rules,
                else_rules,
            } => f
                .debug_struct("When")
                .field("condition", condition)
                .field("then_rules", then_rules)
                .field("else_rules", else_rules)
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
            (
                Self::Range { min: a1, max: a2 },
                Self::Range { min: b1, max: b2 },
            ) => a1 == b1 && a2 == b2,
            (Self::Step(a), Self::Step(b)) => a == b,
            (Self::Equals(a), Self::Equals(b)) => a == b,
            (Self::OneOf(a), Self::OneOf(b)) => a == b,
            (Self::All(a), Self::All(b)) => a == b,
            (Self::Any(a), Self::Any(b)) => a == b,
            (Self::Not(a), Self::Not(b)) => a == b,
            (
                Self::When {
                    condition: c1,
                    then_rules: t1,
                    else_rules: e1,
                },
                Self::When {
                    condition: c2,
                    then_rules: t2,
                    else_rules: e2,
                },
            ) => c1 == c2 && t1 == t2 && e1 == e2,
            (Self::Ref(a), Self::Ref(b)) => a == b,
            (
                Self::WithMessage { rule: r1, message: m1 },
                Self::WithMessage { rule: r2, message: m2 },
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
    /// Combines this rule with another using AND logic.
    ///
    /// Both rules must pass for the combined rule to pass.
    ///
    /// # Example
    ///
    /// ```rust
    /// use walrs_inputfilter::rule::Rule;
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
    /// use walrs_inputfilter::rule::Rule;
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
    /// use walrs_inputfilter::rule::Rule;
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
    /// use walrs_inputfilter::rule::{Rule, Condition};
    ///
    /// let rule = Rule::<String>::MinLength(8)
    ///     .when(Condition::IsNotEmpty);
    /// ```
    pub fn when(self, condition: Condition<T>) -> Rule<T> {
        Rule::When {
            condition,
            then_rules: vec![self],
            else_rules: None,
        }
    }

    /// Creates a conditional rule with else branch.
    ///
    /// # Example
    ///
    /// ```rust
    /// use walrs_inputfilter::rule::{Rule, Condition};
    ///
    /// let rule = Rule::<i32>::Min(0)
    ///     .when_else(
    ///         Condition::GreaterThan(0),
    ///         vec![Rule::Max(100)],  // else rules
    ///     );
    /// ```
    pub fn when_else(self, condition: Condition<T>, else_rules: Vec<Rule<T>>) -> Rule<T> {
        Rule::When {
            condition,
            then_rules: vec![self],
            else_rules: Some(else_rules),
        }
    }

    /// Creates a custom rule from a closure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use walrs_inputfilter::rule::Rule;
    /// use walrs_inputfilter::{Violation, ViolationType};
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
    /// use walrs_inputfilter::rule::Rule;
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
    /// use walrs_inputfilter::rule::Rule;
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
    /// use walrs_inputfilter::rule::Rule;
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
                then_rules,
                else_rules,
            } => {
                assert_eq!(condition, Condition::IsNotEmpty);
                assert_eq!(then_rules.len(), 1);
                assert!(else_rules.is_none());
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
        assert!(json.contains("Range"));
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
        assert_eq!(
            Rule::<i32>::range(0, 100),
            Rule::Range { min: 0, max: 100 }
        );
    }

    // ========================================================================
    // Message Tests
    // ========================================================================

    #[test]
    fn test_message_static() {
        let msg: Message<String> = Message::static_msg("Error message");
        assert!(msg.is_static());
        assert!(!msg.is_provider());
        assert_eq!(msg.resolve(&"any".to_string()), "Error message");
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
        let empty: Message<String> = Message::Static(String::new());
        assert_eq!(empty.resolve_or(&"x".to_string(), "fallback"), "fallback");
    }

    #[test]
    fn test_message_resolve_or_with_value() {
        let msg: Message<String> = Message::from("custom");
        assert_eq!(msg.resolve_or(&"x".to_string(), "fallback"), "custom");
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

    // ========================================================================
    // WithMessage Tests
    // ========================================================================

    #[test]
    fn test_rule_with_message_static() {
        let rule = Rule::<String>::MinLength(8)
            .with_message("Password too short");

        match rule {
            Rule::WithMessage { rule: inner, message } => {
                assert_eq!(*inner, Rule::MinLength(8));
                assert_eq!(message, Message::from("Password too short"));
            }
            _ => panic!("Expected Rule::WithMessage"),
        }
    }

    #[test]
    fn test_rule_with_message_provider() {
        let rule = Rule::<i32>::Min(0)
            .with_message_provider(|ctx| format!("Got {}, expected >= 0", ctx.value));

        match rule {
            Rule::WithMessage { rule: inner, message } => {
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
            Rule::WithMessage { rule: inner, message } => {
                match *inner {
                    Rule::All(rules) => assert_eq!(rules.len(), 2),
                    _ => panic!("Expected Rule::All inside WithMessage"),
                }
                assert_eq!(message.resolve(&"".to_string()), "Length must be between 3 and 10");
            }
            _ => panic!("Expected Rule::WithMessage"),
        }
    }

    // ========================================================================
    // MessageParams Tests
    // ========================================================================

    #[test]
    fn test_message_params_new() {
        let params = MessageParams::new("MinLength");
        assert_eq!(params.rule_name, "MinLength");
        assert!(!params.required);
        assert!(params.min_length.is_none());
        assert!(params.max_length.is_none());
        assert!(params.exact_length.is_none());
        assert!(params.min.is_none());
        assert!(params.max.is_none());
    }

    #[test]
    fn test_message_params_builder_pattern() {
        let params = MessageParams::new("Range")
            .with_min(0)
            .with_max(100);

        assert_eq!(params.rule_name, "Range");
        assert_eq!(params.min, Some("0".to_string()));
        assert_eq!(params.max, Some("100".to_string()));
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
        let params = MessageParams::new("Required")
            .with_required(true);

        assert!(params.required);
    }

    #[test]
    fn test_message_params_one_of() {
        let params = MessageParams::new("OneOf")
            .with_one_of(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        assert_eq!(params.one_of, Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]));
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

    // ========================================================================
    // MessageContext Tests
    // ========================================================================

    #[test]
    fn test_message_context_new() {
        let value = "test".to_string();
        let params = MessageParams::new("MinLength").with_min(5);
        let ctx = MessageContext::new(&value, params);

        assert_eq!(ctx.value, &"test".to_string());
        assert_eq!(ctx.params.rule_name, "MinLength");
        assert_eq!(ctx.params.min, Some("5".to_string()));
    }

    #[test]
    fn test_message_provider_with_params() {
        let msg: Message<String> = Message::provider(|ctx| {
            format!(
                "Length must be at least {}",
                ctx.params.min_length.map(|n| n.to_string()).unwrap_or("?".to_string())
            )
        });

        assert!(msg.is_provider());
        assert!(!msg.is_static());
    }

    #[test]
    fn test_message_provider_resolve_with_context() {
        let msg: Message<String> = Message::provider(|ctx| {
            format!(
                "{} validation failed: expected min length {}, got '{}'",
                ctx.params.rule_name,
                ctx.params.min_length.map(|n| n.to_string()).unwrap_or("?".to_string()),
                ctx.value
            )
        });

        let params = MessageParams::new("MinLength")
            .with_min_length(8);
        let value = "abc".to_string();
        let ctx = MessageContext::new(&value, params);

        assert_eq!(
            msg.resolve_with_context(&ctx),
            "MinLength validation failed: expected min length 8, got 'abc'"
        );
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


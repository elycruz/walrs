//! Options structs for configurable validation rule variants.

use serde::{Deserialize, Serialize};

/// Options for URI validation (`Rule::Uri`).
///
/// Controls which URI forms are accepted and optionally restricts allowed schemes.
/// Modeled after laminas-validator's `Uri` validator options.
///
/// # Defaults
///
/// - `allow_absolute`: `true`
/// - `allow_relative`: `true`
/// - `allowed_schemes`: `None` (any scheme accepted)
///
/// # Example
///
/// ```rust
/// use walrs_validation::UriOptions;
///
/// // Only accept absolute HTTP/HTTPS URIs
/// let opts = UriOptions {
///   allow_absolute: true,
///   allow_relative: false,
///   allowed_schemes: Some(vec!["http".into(), "https".into()]),
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UriOptions {
  /// Allow absolute URIs (default: true).
  pub allow_absolute: bool,

  /// Allow relative URI references (default: true).
  pub allow_relative: bool,

  /// Restrict to specific schemes (case-insensitive).
  /// `None` means any scheme is accepted.
  pub allowed_schemes: Option<Vec<String>>,
}

impl Default for UriOptions {
  fn default() -> Self {
    Self {
      allow_absolute: true,
      allow_relative: true,
      allowed_schemes: None,
    }
  }
}

/// Options for URL validation (`Rule::Url`).
///
/// Controls which URL schemes are accepted. Uses the `url` crate's WHATWG URL Standard
/// parser (absolute URLs only). For relative URI support, use `Rule::Uri(UriOptions)`.
///
/// # Defaults
///
/// - `allowed_schemes`: `Some(["http", "https"])` (backward-compatible)
///
/// # Example
///
/// ```rust
/// use walrs_validation::UrlOptions;
///
/// // Accept any scheme
/// let opts = UrlOptions {
///   allowed_schemes: None,
/// };
///
/// // Only HTTPS
/// let opts = UrlOptions {
///   allowed_schemes: Some(vec!["https".into()]),
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UrlOptions {
  /// Restrict to specific schemes (case-insensitive).
  /// Defaults to `Some(["http", "https"])`.
  /// `None` means any scheme is accepted.
  pub allowed_schemes: Option<Vec<String>>,
}

impl Default for UrlOptions {
  fn default() -> Self {
    Self {
      allowed_schemes: Some(vec!["http".into(), "https".into()]),
    }
  }
}

/// Options for IP address validation (`Rule::Ip`).
///
/// Controls which IP address forms are accepted.
/// Modeled after laminas-validator's `Ip` validator options.
///
/// # Defaults
///
/// - `allow_ipv4`: `true`
/// - `allow_ipv6`: `true`
/// - `allow_ipvfuture`: `false`
/// - `allow_literal`: `true`
///
/// # Example
///
/// ```rust
/// use walrs_validation::IpOptions;
///
/// // Only accept IPv4 addresses
/// let opts = IpOptions {
///   allow_ipv4: true,
///   allow_ipv6: false,
///   ..Default::default()
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IpOptions {
  /// Allow IPv4 addresses (default: true).
  pub allow_ipv4: bool,

  /// Allow IPv6 addresses (default: true).
  pub allow_ipv6: bool,

  /// Allow IPvFuture addresses per RFC 3986 §3.2.2 (default: false).
  pub allow_ipvfuture: bool,

  /// Allow bracket-literal notation, e.g. `[::1]` (default: true).
  pub allow_literal: bool,
}

impl Default for IpOptions {
  fn default() -> Self {
    Self {
      allow_ipv4: true,
      allow_ipv6: true,
      allow_ipvfuture: false,
      allow_literal: true,
    }
  }
}

/// Options for hostname validation (`Rule::Hostname`).
///
/// Controls which hostname forms are accepted. Inspired by laminas-validator's
/// `Hostname` and `HostWithPublicIPv4Address` validators.
///
/// # Defaults
///
/// - `allow_dns`: `true`
/// - `allow_ip`: `true`
/// - `allow_local`: `false`
/// - `require_public_ipv4`: `false`
///
/// # Example
///
/// ```rust
/// use walrs_validation::HostnameOptions;
///
/// // Only accept DNS hostnames (no IPs, no local names)
/// let opts = HostnameOptions {
///   allow_dns: true,
///   allow_ip: false,
///   allow_local: false,
///   ..Default::default()
/// };
///
/// // Require that IP inputs are public IPv4 addresses
/// let opts = HostnameOptions {
///   require_public_ipv4: true,
///   ..Default::default()
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostnameOptions {
  /// Allow DNS hostnames, e.g. `example.com` (default: true).
  pub allow_dns: bool,

  /// Allow IP addresses as valid hostnames (default: true).
  pub allow_ip: bool,

  /// Allow local/reserved hostnames, e.g. `localhost`, single-label names
  /// without a TLD (default: false).
  pub allow_local: bool,

  /// When true, IP address inputs must be public (non-reserved) IPv4 addresses.
  /// Rejects RFC 1918 private, loopback, link-local, documentation, and other
  /// reserved ranges. Only applies when the input is an IP address.
  /// (default: false).
  pub require_public_ipv4: bool,
}

impl Default for HostnameOptions {
  fn default() -> Self {
    Self {
      allow_dns: true,
      allow_ip: true,
      allow_local: false,
      require_public_ipv4: false,
    }
  }
}

/// Options for email address validation (`Rule::Email`).
///
/// Controls which forms of email addresses are accepted. The domain part
/// is validated using hostname rules; the local part is checked for length
/// and allowed characters.
///
/// Inspired by laminas-validator's `EmailAddress` options, excluding
/// network-dependent options (`useMxCheck`, `useDeepMxCheck`).
///
/// # Defaults
///
/// - `allow_dns`: `true`
/// - `allow_ip`: `false`
/// - `allow_local`: `false`
/// - `check_domain`: `true`
/// - `min_local_part_length`: `1`
/// - `max_local_part_length`: `64`
///
/// # Example
///
/// ```rust
/// use walrs_validation::EmailOptions;
///
/// // Accept emails with IP-literal domains, e.g. `user@[192.168.0.1]`
/// let opts = EmailOptions {
///   allow_ip: true,
///   ..Default::default()
/// };
///
/// // Accept emails with local hostnames, e.g. `user@localhost`
/// let opts = EmailOptions {
///   allow_local: true,
///   ..Default::default()
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmailOptions {
  /// Allow DNS domain names in the email domain part, e.g. `user@example.com`
  /// (default: true).
  pub allow_dns: bool,

  /// Allow IP address literals in the email domain part, e.g. `user@[192.168.0.1]`
  /// (default: false).
  pub allow_ip: bool,

  /// Allow local/reserved hostnames in the email domain part, e.g. `user@localhost`
  /// (default: false).
  pub allow_local: bool,

  /// Whether to validate the domain part of the email address (default: true).
  /// When false, only the local (user) part is checked.
  pub check_domain: bool,

  /// Minimum length for the local part (default: 1).
  pub min_local_part_length: usize,

  /// Maximum length for the local part (default: 64, per RFC 5321).
  pub max_local_part_length: usize,
}

impl Default for EmailOptions {
  fn default() -> Self {
    Self {
      allow_dns: true,
      allow_ip: false,
      allow_local: false,
      check_domain: true,
      min_local_part_length: 1,
      max_local_part_length: 64,
    }
  }
}

/// Date format specification for date validation rules.
///
/// Controls how date strings are parsed. Can be one of several common presets
/// or a custom strftime-style format string.
///
/// # Defaults
///
/// - `Iso8601` (e.g., `2026-02-23` or `2026-02-23T18:00:00`)
///
/// # Example
///
/// ```rust
/// use walrs_validation::DateFormat;
///
/// let iso = DateFormat::Iso8601;
/// let us = DateFormat::UsDate;
/// let custom = DateFormat::Custom("%d %B %Y".into());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum DateFormat {
  /// ISO 8601 date: `2026-02-23` or datetime: `2026-02-23T18:00:00`
  Iso8601,
  /// US-style date: `02/23/2026`
  UsDate,
  /// European-style date: `23/02/2026`
  EuDate,
  /// RFC 2822 date: `Mon, 23 Feb 2026 18:00:00`
  Rfc2822,
  /// Custom strftime-style format string (e.g., `%d %B %Y`)
  Custom(String),
}

impl Default for DateFormat {
  fn default() -> Self {
    Self::Iso8601
  }
}

/// Options for date validation (`Rule::Date`).
///
/// Controls the expected date format and whether a time component is accepted.
///
/// # Defaults
///
/// - `format`: `DateFormat::Iso8601`
/// - `allow_time`: `false`
///
/// # Example
///
/// ```rust
/// use walrs_validation::{DateOptions, DateFormat};
///
/// // Accept ISO 8601 date-only strings
/// let opts = DateOptions::default();
///
/// // Accept US-style dates with time
/// let opts = DateOptions {
///   format: DateFormat::UsDate,
///   allow_time: true,
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DateOptions {
  /// Expected date format (default: ISO 8601).
  pub format: DateFormat,

  /// Whether to also accept a time component (default: false, date-only).
  pub allow_time: bool,
}

impl Default for DateOptions {
  fn default() -> Self {
    Self {
      format: DateFormat::Iso8601,
      allow_time: false,
    }
  }
}

/// Options for date range validation (`Rule::DateRange`).
///
/// Validates that a date string is parseable and falls within an optional
/// min/max range. The `min` and `max` bounds are stored as strings for
/// serialization and parsed at validation time using the configured
/// [`DateFormat`].
///
/// # Bound format and cross-format handling
///
/// Bounds are ideally provided in the same [`DateFormat`] and `allow_time`
/// setting as the input values. For the default `DateFormat::Iso8601`:
///
/// - When `allow_time` is `false`, bounds should be date-only values
///   (e.g., `"2020-01-01"`).
/// - When `allow_time` is `true`, bounds may be full date-time values
///   (e.g., `"2020-01-01T00:00:00"`) for precise time-based comparison.
///
/// When there is a format mismatch, the bound is still applied using the
/// available date component:
///
/// - A date-only bound with a datetime input (`allow_time = true`): the
///   date component of the input is compared against the bound (e.g., any
///   time on or after `"2020-01-01"` satisfies that min bound).
/// - A datetime bound with a date-only input (`allow_time = false`): the
///   date component of the bound is extracted for comparison.
///
/// If a bound string cannot be parsed as either a date or datetime in the
/// configured format, that side of the range is treated as if it were not
/// set and the corresponding min/max check is skipped.
///
/// # Defaults
///
/// - `format`: `DateFormat::Iso8601`
/// - `allow_time`: `false`
/// - `min`: `None`
/// - `max`: `None`
///
/// # Example
///
/// ```rust
/// use walrs_validation::{DateRangeOptions, DateFormat};
///
/// // Accept ISO dates between 2020-01-01 and 2030-12-31
/// let opts = DateRangeOptions {
///   format: DateFormat::Iso8601,
///   allow_time: false,
///   min: Some("2020-01-01".into()),
///   max: Some("2030-12-31".into()),
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DateRangeOptions {
  /// Expected date format for validated input values (default: ISO 8601).
  pub format: DateFormat,

  /// Whether to also accept a time component (default: false, date-only).
  pub allow_time: bool,

  /// Minimum date/datetime (inclusive), always specified in ISO 8601 format,
  /// regardless of the configured [`DateFormat`]. `None` means no lower bound.
  pub min: Option<String>,

  /// Maximum date/datetime (inclusive), always specified in ISO 8601 format,
  /// regardless of the configured [`DateFormat`]. `None` means no upper bound.
  pub max: Option<String>,
}

impl Default for DateRangeOptions {
  fn default() -> Self {
    Self {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: None,
      max: None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_uri_options_default() {
    let opts = UriOptions::default();
    assert!(opts.allow_absolute);
    assert!(opts.allow_relative);
    assert!(opts.allowed_schemes.is_none());
  }

  #[test]
  fn test_url_options_default() {
    let opts = UrlOptions::default();
    assert_eq!(
      opts.allowed_schemes,
      Some(vec!["http".to_string(), "https".to_string()])
    );
  }

  #[test]
  fn test_url_options_serialization() {
    let opts = UrlOptions {
      allowed_schemes: Some(vec!["https".into()]),
    };
    let json = serde_json::to_string(&opts).unwrap();
    let deserialized: UrlOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(opts, deserialized);
  }

  #[test]
  fn test_ip_options_default() {
    let opts = IpOptions::default();
    assert!(opts.allow_ipv4);
    assert!(opts.allow_ipv6);
    assert!(!opts.allow_ipvfuture);
    assert!(opts.allow_literal);
  }

  #[test]
  fn test_uri_options_serialization() {
    let opts = UriOptions {
      allow_absolute: true,
      allow_relative: false,
      allowed_schemes: Some(vec!["https".into()]),
    };
    let json = serde_json::to_string(&opts).unwrap();
    let deserialized: UriOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(opts, deserialized);
  }

  #[test]
  fn test_ip_options_serialization() {
    let opts = IpOptions {
      allow_ipv4: false,
      allow_ipv6: true,
      allow_ipvfuture: true,
      allow_literal: false,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let deserialized: IpOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(opts, deserialized);
  }

  #[test]
  fn test_hostname_options_default() {
    let opts = HostnameOptions::default();
    assert!(opts.allow_dns);
    assert!(opts.allow_ip);
    assert!(!opts.allow_local);
    assert!(!opts.require_public_ipv4);
  }

  #[test]
  fn test_hostname_options_serialization() {
    let opts = HostnameOptions {
      allow_dns: true,
      allow_ip: false,
      allow_local: true,
      require_public_ipv4: false,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let deserialized: HostnameOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(opts, deserialized);
  }

  #[test]
  fn test_email_options_default() {
    let opts = EmailOptions::default();
    assert!(opts.allow_dns);
    assert!(!opts.allow_ip);
    assert!(!opts.allow_local);
    assert!(opts.check_domain);
    assert_eq!(opts.min_local_part_length, 1);
    assert_eq!(opts.max_local_part_length, 64);
  }

  #[test]
  fn test_email_options_serialization() {
    let opts = EmailOptions {
      allow_dns: true,
      allow_ip: true,
      allow_local: false,
      check_domain: true,
      min_local_part_length: 2,
      max_local_part_length: 32,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let deserialized: EmailOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(opts, deserialized);
  }

  #[test]
  fn test_date_format_default() {
    assert_eq!(DateFormat::default(), DateFormat::Iso8601);
  }

  #[test]
  fn test_date_format_serialization() {
    let formats = vec![
      DateFormat::Iso8601,
      DateFormat::UsDate,
      DateFormat::EuDate,
      DateFormat::Rfc2822,
      DateFormat::Custom("%d %B %Y".into()),
    ];
    for fmt in formats {
      let json = serde_json::to_string(&fmt).unwrap();
      let deserialized: DateFormat = serde_json::from_str(&json).unwrap();
      assert_eq!(fmt, deserialized);
    }
  }

  #[test]
  fn test_date_options_default() {
    let opts = DateOptions::default();
    assert_eq!(opts.format, DateFormat::Iso8601);
    assert!(!opts.allow_time);
  }

  #[test]
  fn test_date_options_serialization() {
    let opts = DateOptions {
      format: DateFormat::UsDate,
      allow_time: true,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let deserialized: DateOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(opts, deserialized);
  }

  #[test]
  fn test_date_range_options_default() {
    let opts = DateRangeOptions::default();
    assert_eq!(opts.format, DateFormat::Iso8601);
    assert!(!opts.allow_time);
    assert!(opts.min.is_none());
    assert!(opts.max.is_none());
  }

  #[test]
  fn test_date_range_options_serialization() {
    let opts = DateRangeOptions {
      format: DateFormat::Iso8601,
      allow_time: false,
      min: Some("2020-01-01".into()),
      max: Some("2030-12-31".into()),
    };
    let json = serde_json::to_string(&opts).unwrap();
    let deserialized: DateRangeOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(opts, deserialized);
  }
}

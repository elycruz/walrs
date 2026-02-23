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
}

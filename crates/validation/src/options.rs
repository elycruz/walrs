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
}

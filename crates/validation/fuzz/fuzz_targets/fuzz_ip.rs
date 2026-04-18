#![no_main]

use libfuzzer_sys::fuzz_target;
use walrs_validation::traits::ValidateRef;
use walrs_validation::{IpOptions, Rule};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Default (IPv4 + IPv6, with literal)
        let rule = Rule::<String>::Ip(IpOptions::default());
        let _ = rule.validate_ref(s);

        // IPv4 only
        let opts = IpOptions {
            allow_ipv4: true,
            allow_ipv6: false,
            allow_ipvfuture: false,
            allow_literal: false,
        };
        let rule = Rule::<String>::Ip(opts);
        let _ = rule.validate_ref(s);

        // IPv6 only, with literal notation
        let opts = IpOptions {
            allow_ipv4: false,
            allow_ipv6: true,
            allow_ipvfuture: false,
            allow_literal: true,
        };
        let rule = Rule::<String>::Ip(opts);
        let _ = rule.validate_ref(s);

        // IPvFuture enabled
        let opts = IpOptions {
            allow_ipv4: true,
            allow_ipv6: true,
            allow_ipvfuture: true,
            allow_literal: true,
        };
        let rule = Rule::<String>::Ip(opts);
        let _ = rule.validate_ref(s);
    }
});

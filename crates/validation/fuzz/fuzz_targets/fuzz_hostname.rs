#![no_main]

use libfuzzer_sys::fuzz_target;
use walrs_validation::traits::ValidateRef;
use walrs_validation::{HostnameOptions, Rule};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Default options
        let rule = Rule::<String>::Hostname(HostnameOptions::default());
        let _ = rule.validate_ref(s);

        // DNS + IP + local allowed
        let opts = HostnameOptions {
            allow_dns: true,
            allow_ip: true,
            allow_local: true,
            require_public_ipv4: false,
        };
        let rule = Rule::<String>::Hostname(opts);
        let _ = rule.validate_ref(s);

        // DNS only
        let opts = HostnameOptions {
            allow_dns: true,
            allow_ip: false,
            allow_local: false,
            require_public_ipv4: false,
        };
        let rule = Rule::<String>::Hostname(opts);
        let _ = rule.validate_ref(s);

        // Require public IPv4
        let opts = HostnameOptions {
            allow_dns: true,
            allow_ip: true,
            allow_local: false,
            require_public_ipv4: true,
        };
        let rule = Rule::<String>::Hostname(opts);
        let _ = rule.validate_ref(s);
    }
});

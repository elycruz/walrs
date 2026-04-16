#![no_main]

use libfuzzer_sys::fuzz_target;
use walrs_validation::traits::ValidateRef;
use walrs_validation::{EmailOptions, Rule};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Default options
        let rule = Rule::<String>::Email(EmailOptions::default());
        let _ = rule.validate_ref(s);

        // With IP domain and local domain allowed
        let opts = EmailOptions {
            allow_dns: true,
            allow_ip: true,
            allow_local: true,
            check_domain: true,
            min_local_part_length: 1,
            max_local_part_length: 64,
        };
        let rule = Rule::<String>::Email(opts);
        let _ = rule.validate_ref(s);

        // Domain check disabled
        let opts = EmailOptions {
            check_domain: false,
            ..Default::default()
        };
        let rule = Rule::<String>::Email(opts);
        let _ = rule.validate_ref(s);
    }
});

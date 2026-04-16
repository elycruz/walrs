#![no_main]

use libfuzzer_sys::fuzz_target;
use walrs_validation::traits::ValidateRef;
use walrs_validation::{DateFormat, DateOptions, DateRangeOptions, Rule};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // ISO 8601 date only
        let opts = DateOptions {
            format: DateFormat::Iso8601,
            allow_time: false,
        };
        let rule = Rule::<String>::Date(opts);
        let _ = rule.validate_ref(s);

        // ISO 8601 with time
        let opts = DateOptions {
            format: DateFormat::Iso8601,
            allow_time: true,
        };
        let rule = Rule::<String>::Date(opts);
        let _ = rule.validate_ref(s);

        // US date format
        let opts = DateOptions {
            format: DateFormat::UsDate,
            allow_time: false,
        };
        let rule = Rule::<String>::Date(opts);
        let _ = rule.validate_ref(s);

        // EU date format
        let opts = DateOptions {
            format: DateFormat::EuDate,
            allow_time: false,
        };
        let rule = Rule::<String>::Date(opts);
        let _ = rule.validate_ref(s);

        // Date range validation
        let opts = DateRangeOptions {
            format: DateFormat::Iso8601,
            allow_time: false,
            min: Some("2000-01-01".into()),
            max: Some("2099-12-31".into()),
        };
        let rule = Rule::<String>::DateRange(opts);
        let _ = rule.validate_ref(s);
    }
});

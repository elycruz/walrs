#![no_main]

use libfuzzer_sys::fuzz_target;
use walrs_validation::traits::ValidateRef;
use walrs_validation::{Rule, UriOptions, UrlOptions};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // URL with default options (http/https)
        let rule = Rule::<String>::Url(UrlOptions::default());
        let _ = rule.validate_ref(s);

        // URI with default options
        let rule = Rule::<String>::Uri(UriOptions::default());
        let _ = rule.validate_ref(s);

        // URL with scheme restrictions
        let opts = UrlOptions {
            allowed_schemes: Some(vec!["https".into(), "http".into()]),
        };
        let rule = Rule::<String>::Url(opts);
        let _ = rule.validate_ref(s);

        // URI — absolute only, https/http schemes
        let opts = UriOptions {
            allow_absolute: true,
            allow_relative: false,
            allowed_schemes: Some(vec!["https".into(), "http".into()]),
        };
        let rule = Rule::<String>::Uri(opts);
        let _ = rule.validate_ref(s);

        // URI — relative only, no scheme restriction
        let opts = UriOptions {
            allow_absolute: false,
            allow_relative: true,
            allowed_schemes: None,
        };
        let rule = Rule::<String>::Uri(opts);
        let _ = rule.validate_ref(s);
    }
});

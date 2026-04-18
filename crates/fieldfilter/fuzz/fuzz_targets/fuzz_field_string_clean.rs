#![no_main]

use libfuzzer_sys::fuzz_target;
use walrs_fieldfilter::{Field, FieldBuilder, FilterOp, Rule};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Field with email validation + trim/lowercase filters
        let field: Field<String> = FieldBuilder::default()
            .name("email")
            .rule(Rule::Email(Default::default()))
            .filters(vec![FilterOp::Trim, FilterOp::Lowercase])
            .build()
            .unwrap();
        let _ = field.clean(s.to_string());
        let _ = field.clean_ref(s);

        // Field with length validation + strip tags
        let field: Field<String> = FieldBuilder::default()
            .name("title")
            .rule(Rule::MinLength(1).and(Rule::MaxLength(200)))
            .filters(vec![FilterOp::Trim, FilterOp::StripTags])
            .build()
            .unwrap();
        let _ = field.clean(s.to_string());
    }
});

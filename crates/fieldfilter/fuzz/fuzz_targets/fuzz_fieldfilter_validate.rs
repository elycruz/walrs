#![no_main]
#![allow(deprecated)]

use arbitrary::Arbitrary;
use indexmap::IndexMap;
use libfuzzer_sys::fuzz_target;
use walrs_fieldfilter::{Field, FieldBuilder, FieldFilter, FilterOp, Rule, Value};

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    name_value: String,
    email_value: String,
    age_value: String,
}

fuzz_target!(|input: FuzzInput| {
    let mut ff = FieldFilter::new();

    // Name field: required, length 1-100, trimmed
    let name_field: Field<Value> = FieldBuilder::default()
        .name("name")
        .rule(Rule::MinLength(1).and(Rule::MaxLength(100)))
        .filters(vec![FilterOp::Trim, FilterOp::StripTags])
        .build()
        .unwrap();
    ff.add_field("name".to_string(), name_field);

    // Email field: email validation
    let email_field: Field<Value> = FieldBuilder::default()
        .name("email")
        .rule(Rule::Email(Default::default()))
        .filters(vec![FilterOp::Trim, FilterOp::Lowercase])
        .build()
        .unwrap();
    ff.add_field("email".to_string(), email_field);

    // Build input data
    let mut data = IndexMap::new();
    data.insert("name".to_string(), Value::Str(input.name_value));
    data.insert("email".to_string(), Value::Str(input.email_value));
    data.insert("age".to_string(), Value::Str(input.age_value));

    // Run full pipeline
    let _ = ff.validate(&data);
    let _ = ff.clean(data);
});

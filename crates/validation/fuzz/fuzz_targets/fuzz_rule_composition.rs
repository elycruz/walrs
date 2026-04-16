#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use walrs_validation::traits::ValidateRef;
use walrs_validation::Rule;

/// A simplified fuzz input that selects a rule variant and provides a string.
#[derive(Debug, Arbitrary)]
struct FuzzInput {
    rule_kind: u8,
    input: String,
    min_len: u8,
    max_len: u8,
}

fuzz_target!(|input: FuzzInput| {
    let rule: Rule<String> = match input.rule_kind % 8 {
        0 => Rule::Email(Default::default()),
        1 => Rule::Url(Default::default()),
        2 => Rule::Uri(Default::default()),
        3 => Rule::Ip(Default::default()),
        4 => Rule::Hostname(Default::default()),
        5 => Rule::MinLength(input.min_len as usize),
        6 => Rule::MaxLength(input.max_len as usize),
        7 => {
            // Composed rule: MinLength AND MaxLength
            let min = std::cmp::min(input.min_len, input.max_len) as usize;
            let max = std::cmp::max(input.min_len, input.max_len) as usize;
            Rule::All(vec![Rule::MinLength(min), Rule::MaxLength(max)])
        }
        _ => unreachable!(),
    };
    let _ = rule.validate_ref(input.input.as_str());
});

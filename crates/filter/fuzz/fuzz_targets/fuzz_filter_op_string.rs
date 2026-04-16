#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use walrs_filter::FilterOp;

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    op_kind: u8,
    input: String,
    max_length: u16,
    from: String,
    to: String,
}

fuzz_target!(|input: FuzzInput| {
    let op: FilterOp<String> = match input.op_kind % 8 {
        0 => FilterOp::Trim,
        1 => FilterOp::Lowercase,
        2 => FilterOp::Uppercase,
        3 => FilterOp::StripTags,
        4 => FilterOp::Slug {
            max_length: Some(input.max_length as usize),
        },
        5 => FilterOp::Truncate {
            max_length: input.max_length as usize,
        },
        6 => FilterOp::Replace {
            from: input.from.clone(),
            to: input.to.clone(),
        },
        7 => FilterOp::Chain(vec![
            FilterOp::Trim,
            FilterOp::StripTags,
            FilterOp::Slug {
                max_length: Some(input.max_length as usize),
            },
        ]),
        _ => unreachable!(),
    };
    let _ = op.apply(input.input);
});

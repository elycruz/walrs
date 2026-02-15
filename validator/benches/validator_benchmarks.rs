//! Benchmarks for walrs_validator
//!
//! Run with: `cargo bench -p walrs_validator`
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use walrs_validator::{
    LengthValidatorBuilder, PatternValidatorBuilder, RangeValidatorBuilder,
    NumberValidatorBuilder, EqualityValidatorBuilder,
    Validate, ValidateRef, ValidateExt,
};
use regex::Regex;
use std::borrow::Cow;
fn bench_length_validator(c: &mut Criterion) {
    let mut group = c.benchmark_group("LengthValidator");
    let validator = LengthValidatorBuilder::<str>::default()
        .min_length(5)
        .max_length(100)
        .build()
        .unwrap();
    let inputs = [
        ("short", "hi"),
        ("valid_short", "hello"),
        ("valid_medium", "hello world this is a test"),
        ("valid_long", "this is a much longer string that should still pass validation easily"),
        ("too_long", &"x".repeat(150)),
    ];
    for (name, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("validate_ref", name),
            &input,
            |b, input| {
                b.iter(|| validator.validate_ref(black_box(*input)))
            },
        );
    }
    group.finish();
}
fn bench_range_validator(c: &mut Criterion) {
    let mut group = c.benchmark_group("RangeValidator");
    let validator = RangeValidatorBuilder::<i32>::default()
        .min(0)
        .max(100)
        .build()
        .unwrap();
    let inputs = [
        ("below_min", -10),
        ("at_min", 0),
        ("middle", 50),
        ("at_max", 100),
        ("above_max", 150),
    ];
    for (name, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("validate", name),
            &input,
            |b, input| {
                b.iter(|| validator.validate(black_box(*input)))
            },
        );
    }
    group.finish();
}
fn bench_number_validator(c: &mut Criterion) {
    let mut group = c.benchmark_group("NumberValidator");
    let validator = NumberValidatorBuilder::<i32>::default()
        .min(0)
        .max(100)
        .step(5)
        .build()
        .unwrap();
    let inputs = [
        ("valid", 50),
        ("invalid_step", 51),
        ("below_min", -5),
        ("above_max", 105),
    ];
    for (name, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("validate", name),
            &input,
            |b, input| {
                b.iter(|| validator.validate(black_box(*input)))
            },
        );
    }
    group.finish();
}
fn bench_pattern_validator(c: &mut Criterion) {
    let mut group = c.benchmark_group("PatternValidator");
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    let validator = PatternValidatorBuilder::default()
        .pattern(Cow::Owned(email_regex))
        .build()
        .unwrap();
    let inputs = [
        ("valid_simple", "test@example.com"),
        ("valid_complex", "user.name+tag@subdomain.example.co.uk"),
        ("invalid_no_at", "invalidemail.com"),
        ("invalid_no_domain", "invalid@"),
    ];
    for (name, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("validate_ref", name),
            &input,
            |b, input| {
                b.iter(|| validator.validate_ref(black_box(*input)))
            },
        );
    }
    group.finish();
}
fn bench_equality_validator(c: &mut Criterion) {
    let mut group = c.benchmark_group("EqualityValidator");
    let validator = EqualityValidatorBuilder::<&str>::default()
        .rhs_value("expected_value")
        .build()
        .unwrap();
    let inputs = [
        ("equal", "expected_value"),
        ("not_equal_short", "wrong"),
        ("not_equal_long", "this is a completely different value"),
    ];
    for (name, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("validate", name),
            &input,
            |b, input| {
                b.iter(|| validator.validate(black_box(*input)))
            },
        );
    }
    group.finish();
}
fn bench_combined_validators(c: &mut Criterion) {
    let mut group = c.benchmark_group("CombinedValidators");
    let min_validator = RangeValidatorBuilder::<i32>::default()
        .min(0)
        .build()
        .unwrap();
    let max_validator = RangeValidatorBuilder::<i32>::default()
        .max(100)
        .build()
        .unwrap();
    let combined = min_validator.and(max_validator);
    let inputs = [50, -10, 150];
    for input in inputs {
        group.bench_with_input(
            BenchmarkId::new("and_combinator", input),
            &input,
            |b, input| {
                b.iter(|| combined.validate(black_box(*input)))
            },
        );
    }
    group.finish();
}
fn bench_validator_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("ValidatorComparison");
    let length_validator = LengthValidatorBuilder::<str>::default()
        .min_length(5)
        .max_length(50)
        .build()
        .unwrap();
    let range_validator = RangeValidatorBuilder::<i32>::default()
        .min(0)
        .max(100)
        .build()
        .unwrap();
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    let pattern_validator = PatternValidatorBuilder::default()
        .pattern(Cow::Owned(email_regex))
        .build()
        .unwrap();
    group.bench_function("length_validator", |b| {
        b.iter(|| length_validator.validate_ref(black_box("hello world")))
    });
    group.bench_function("range_validator", |b| {
        b.iter(|| range_validator.validate(black_box(50)))
    });
    group.bench_function("pattern_validator", |b| {
        b.iter(|| pattern_validator.validate_ref(black_box("test@example.com")))
    });
    group.finish();
}
criterion_group!(
    benches,
    bench_length_validator,
    bench_range_validator,
    bench_number_validator,
    bench_pattern_validator,
    bench_equality_validator,
    bench_combined_validators,
    bench_validator_comparison,
);
criterion_main!(benches);

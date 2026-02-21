//! Benchmarks for walrs_validator
//!
//! Run with: `cargo bench -p walrs_validator`
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use regex::Regex;
use std::borrow::Cow;
use walrs_validator::{
  EqualityValidatorBuilder, FnRefValidator, FnValidator, LengthValidatorBuilder,
  PatternValidatorBuilder, RangeValidatorBuilder, StepValidatorBuilder, Validate, ValidateExt,
  ValidateRef, ValidateRefExt, ValidatorAll, Violation, ViolationType,
};
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
    (
      "valid_long",
      "this is a much longer string that should still pass validation easily",
    ),
    ("too_long", &"x".repeat(150)),
  ];
  for (name, input) in inputs {
    group.bench_with_input(
      BenchmarkId::new("validate_ref", name),
      &input,
      |b, input| b.iter(|| validator.validate_ref(black_box(*input))),
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
    group.bench_with_input(BenchmarkId::new("validate", name), &input, |b, input| {
      b.iter(|| validator.validate(black_box(*input)))
    });
  }
  group.finish();
}
fn bench_step_validator(c: &mut Criterion) {
  let mut group = c.benchmark_group("StepValidator");
  let validator = StepValidatorBuilder::<i32>::default()
    .step(5)
    .build()
    .unwrap();
  let inputs = [
    ("valid_zero", 0),
    ("valid_step", 50),
    ("invalid_step", 51),
    ("valid_negative", -10),
    ("invalid_negative", -7),
  ];
  for (name, input) in inputs {
    group.bench_with_input(BenchmarkId::new("validate", name), &input, |b, input| {
      b.iter(|| validator.validate(black_box(*input)))
    });
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
      |b, input| b.iter(|| validator.validate_ref(black_box(*input))),
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
    group.bench_with_input(BenchmarkId::new("validate", name), &input, |b, input| {
      b.iter(|| validator.validate(black_box(*input)))
    });
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
      |b, input| b.iter(|| combined.validate(black_box(*input))),
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
fn bench_fn_validator(c: &mut Criterion) {
  let mut group = c.benchmark_group("FnValidator");

  let positive = FnValidator::new(|v: i32| {
    if v > 0 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::RangeUnderflow, "must be positive"))
    }
  });

  let inputs = [("valid", 42), ("boundary", 1), ("invalid", -1), ("zero", 0)];
  for (name, input) in inputs {
    group.bench_with_input(BenchmarkId::new("validate", name), &input, |b, input| {
      b.iter(|| positive.validate(black_box(*input)))
    });
  }
  group.finish();
}

fn bench_fn_ref_validator(c: &mut Criterion) {
  let mut group = c.benchmark_group("FnRefValidator");

  let non_empty = FnRefValidator::new(|v: &str| {
    if !v.is_empty() {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::ValueMissing, "must not be empty"))
    }
  });

  let inputs = [
    ("valid_short", "hi"),
    ("valid_medium", "hello world"),
    ("valid_long", "the quick brown fox jumps over the lazy dog"),
    ("invalid_empty", ""),
  ];
  for (name, input) in inputs {
    group.bench_with_input(
      BenchmarkId::new("validate_ref", name),
      &input,
      |b, input| b.iter(|| non_empty.validate_ref(black_box(*input))),
    );
  }
  group.finish();
}

fn bench_fn_validator_combinators(c: &mut Criterion) {
  let mut group = c.benchmark_group("FnValidatorCombinators");

  let positive = FnValidator::new(|v: i32| {
    if v > 0 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::RangeUnderflow, "must be positive"))
    }
  });
  let lte_100 = FnValidator::new(|v: i32| {
    if v <= 100 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::RangeOverflow, "must be <= 100"))
    }
  });

  // and
  let and_validator = positive.and(lte_100);
  let inputs = [("valid", 50), ("below_min", -1), ("above_max", 101)];
  for (name, input) in inputs {
    group.bench_with_input(BenchmarkId::new("and", name), &input, |b, input| {
      b.iter(|| and_validator.validate(black_box(*input)))
    });
  }

  // or
  let even = FnValidator::new(|v: i32| {
    if v % 2 == 0 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::CustomError, "must be even"))
    }
  });
  let gt_50 = FnValidator::new(|v: i32| {
    if v > 50 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::RangeUnderflow, "must be > 50"))
    }
  });
  let or_validator = even.or(gt_50);
  let or_inputs = [("first_passes", 4), ("second_passes", 99), ("both_fail", 3)];
  for (name, input) in or_inputs {
    group.bench_with_input(BenchmarkId::new("or", name), &input, |b, input| {
      b.iter(|| or_validator.validate(black_box(*input)))
    });
  }

  // not
  let positive2 = FnValidator::new(|v: i32| {
    if v > 0 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::RangeUnderflow, "must be positive"))
    }
  });
  let not_validator = positive2.not("must not be positive");
  let not_inputs = [("passes", -1), ("fails", 1)];
  for (name, input) in not_inputs {
    group.bench_with_input(BenchmarkId::new("not", name), &input, |b, input| {
      b.iter(|| not_validator.validate(black_box(*input)))
    });
  }

  // optional
  let positive3 = FnValidator::new(|v: i32| {
    if v > 0 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::RangeUnderflow, "must be positive"))
    }
  });
  let optional_validator = positive3.optional(|v: i32| v == 0);
  let opt_inputs = [("skipped", 0), ("valid", 5), ("invalid", -1)];
  for (name, input) in opt_inputs {
    group.bench_with_input(BenchmarkId::new("optional", name), &input, |b, input| {
      b.iter(|| optional_validator.validate(black_box(*input)))
    });
  }

  // when
  let lte_100b = FnValidator::new(|v: i32| {
    if v <= 100 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::RangeOverflow, "must be <= 100"))
    }
  });
  let when_validator = lte_100b.when(|&v: &i32| v > 0);
  let when_inputs = [("skipped", -5), ("passes", 50), ("fails", 200)];
  for (name, input) in when_inputs {
    group.bench_with_input(BenchmarkId::new("when", name), &input, |b, input| {
      b.iter(|| when_validator.validate(black_box(*input)))
    });
  }

  group.finish();
}

fn bench_fn_ref_validator_combinators(c: &mut Criterion) {
  let mut group = c.benchmark_group("FnRefValidatorCombinators");

  // and
  let non_empty = FnRefValidator::new(|v: &str| {
    if !v.is_empty() {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::ValueMissing, "must not be empty"))
    }
  });
  let short_enough = FnRefValidator::new(|v: &str| {
    if v.len() <= 10 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::TooLong, "must be <= 10 chars"))
    }
  });
  let and_validator = non_empty.and(short_enough);
  let and_inputs = [
    ("valid", "hello"),
    ("first_fails", ""),
    ("second_fails", "hello world!!"),
  ];
  for (name, input) in and_inputs {
    group.bench_with_input(BenchmarkId::new("and", name), &input, |b, input| {
      b.iter(|| and_validator.validate_ref(black_box(*input)))
    });
  }

  // or
  let starts_a = FnRefValidator::new(|v: &str| {
    if v.starts_with('a') {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::PatternMismatch, "must start with 'a'"))
    }
  });
  let ends_z = FnRefValidator::new(|v: &str| {
    if v.ends_with('z') {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::PatternMismatch, "must end with 'z'"))
    }
  });
  let or_validator = starts_a.or(ends_z);
  let or_inputs = [("first_passes", "apple"), ("second_passes", "fuzz"), ("both_fail", "hello")];
  for (name, input) in or_inputs {
    group.bench_with_input(BenchmarkId::new("or", name), &input, |b, input| {
      b.iter(|| or_validator.validate_ref(black_box(*input)))
    });
  }

  // not
  let non_empty2 = FnRefValidator::new(|v: &str| {
    if !v.is_empty() {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::ValueMissing, "must not be empty"))
    }
  });
  let not_validator = non_empty2.not("must be empty");
  let not_inputs = [("passes", ""), ("fails", "hello")];
  for (name, input) in not_inputs {
    group.bench_with_input(BenchmarkId::new("not", name), &input, |b, input| {
      b.iter(|| not_validator.validate_ref(black_box(*input)))
    });
  }

  // optional
  let min_len_3 = FnRefValidator::new(|v: &str| {
    if v.len() >= 3 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::TooShort, "min 3 chars"))
    }
  });
  let optional_validator = min_len_3.optional(|v: &str| v.is_empty());
  let opt_inputs = [("skipped", ""), ("valid", "hello"), ("invalid", "hi")];
  for (name, input) in opt_inputs {
    group.bench_with_input(BenchmarkId::new("optional", name), &input, |b, input| {
      b.iter(|| optional_validator.validate_ref(black_box(*input)))
    });
  }

  // when
  let short_enough2 = FnRefValidator::new(|v: &str| {
    if v.len() <= 10 {
      Ok(())
    } else {
      Err(Violation::new(ViolationType::TooLong, "must be <= 10 chars"))
    }
  });
  let when_validator = short_enough2.when(|v: &str| !v.is_empty());
  let when_inputs = [("skipped", ""), ("passes", "hi"), ("fails", "hello world!!")];
  for (name, input) in when_inputs {
    group.bench_with_input(BenchmarkId::new("when", name), &input, |b, input| {
      b.iter(|| when_validator.validate_ref(black_box(*input)))
    });
  }

  group.finish();
}

fn bench_fn_ref_validator_all(c: &mut Criterion) {
  let mut group = c.benchmark_group("FnRefValidatorAll");

  // Rebuild each iteration isn't ideal for bench_function, so we use a
  // pre-built ValidatorAll with three FnRefValidators.
  let all: ValidatorAll<str> = ValidatorAll::new(vec![
    Box::new(FnRefValidator::new(|v: &str| {
      if !v.is_empty() {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::ValueMissing, "must not be empty"))
      }
    })),
    Box::new(FnRefValidator::new(|v: &str| {
      if v.len() >= 3 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::TooShort, "min 3 chars"))
      }
    })),
    Box::new(FnRefValidator::new(|v: &str| {
      if v.len() <= 20 {
        Ok(())
      } else {
        Err(Violation::new(ViolationType::TooLong, "max 20 chars"))
      }
    })),
  ]);

  let inputs = [
    ("all_pass", "hello"),
    ("one_fail", "hi"),           // too short
    ("two_fail", ""),             // empty + too short
    ("third_fail", "this string is definitely longer than twenty characters"),
  ];
  for (name, input) in inputs {
    // validate_ref — short-circuits on first error
    group.bench_with_input(BenchmarkId::new("validate_ref", name), &input, |b, input| {
      b.iter(|| all.validate_ref(black_box(*input)))
    });
    // validate_all — collects every violation
    group.bench_with_input(BenchmarkId::new("validate_all", name), &input, |b, input| {
      b.iter(|| all.validate_all(black_box(*input)))
    });
  }

  group.finish();
}

criterion_group!(
  benches,
  bench_length_validator,
  bench_range_validator,
  bench_step_validator,
  bench_pattern_validator,
  bench_equality_validator,
  bench_combined_validators,
  bench_validator_comparison,
  bench_fn_validator,
  bench_fn_ref_validator,
  bench_fn_validator_combinators,
  bench_fn_ref_validator_combinators,
  bench_fn_ref_validator_all,
);
criterion_main!(benches);

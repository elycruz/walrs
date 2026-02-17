use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::borrow::Cow;
use walrs_inputfilter::{
  FilterForSized, FilterForUnsized, Input, InputBuilder, RefInput, RefInputBuilder, Violation,
  ViolationType::TypeMismatch,
};

fn bench_input_creation(c: &mut Criterion) {
  c.bench_function("input_new", |b| {
    b.iter(Input::<usize, usize>::new);
  });

  c.bench_function("input_builder_default", |b| {
    b.iter(|| {
      InputBuilder::<usize, usize>::default()
        .build()
        .unwrap()
    });
  });

  c.bench_function("input_builder_full", |b| {
    let validator = |x: usize| {
      if x > 10 {
        Err(Violation(TypeMismatch, "Too large".to_string()))
      } else {
        Ok(())
      }
    };
    let filter = |x: usize| x * 2;

    b.iter(|| {
      InputBuilder::<usize, usize>::default()
        .required(black_box(true))
        .break_on_failure(black_box(true))
        .name(black_box("test"))
        .locale(black_box("en_US"))
        .validators(vec![&validator])
        .filters(vec![&filter])
        .build()
        .unwrap()
    });
  });
}

fn bench_input_validate(c: &mut Criterion) {
  let validator = |x: usize| {
    if x > 10 {
      Err(Violation(TypeMismatch, "Too large".to_string()))
    } else {
      Ok(())
    }
  };

  let input = InputBuilder::<usize, usize>::default()
    .required(true)
    .validators(vec![&validator])
    .build()
    .unwrap();

  c.bench_function("input_validate_pass", |b| {
    b.iter(|| input.validate(black_box(5)));
  });

  c.bench_function("input_validate_fail", |b| {
    b.iter(|| input.validate(black_box(15)));
  });

  c.bench_function("input_validate_option_some", |b| {
    b.iter(|| input.validate_option(black_box(Some(5))));
  });

  c.bench_function("input_validate_option_none", |b| {
    b.iter(|| input.validate_option(black_box(None)));
  });
}

fn bench_input_filter(c: &mut Criterion) {
  let filter = |x: usize| x * 2;

  let input = InputBuilder::<usize, usize>::default()
    .filters(vec![&filter])
    .build()
    .unwrap();

  c.bench_function("input_filter", |b| {
    b.iter(|| input.filter(black_box(5)));
  });

  c.bench_function("input_filter_option_some", |b| {
    b.iter(|| input.filter_option(black_box(Some(5))));
  });

  c.bench_function("input_fn_call", |b| {
    b.iter(|| input(black_box(5)));
  });
}

fn bench_ref_input_creation(c: &mut Criterion) {
  c.bench_function("ref_input_default", |b| {
    b.iter(RefInput::<str, Cow<str>>::default);
  });

  c.bench_function("ref_input_builder_default", |b| {
    b.iter(|| {
      RefInputBuilder::<str, Cow<str>>::default()
        .build()
        .unwrap()
    });
  });

  c.bench_function("ref_input_builder_full", |b| {
    let validator = |s: &str| {
      if s.len() > 5 {
        Ok(())
      } else {
        Err(Violation(TypeMismatch, "Too short".to_string()))
      }
    };
    let filter = |s: Cow<str>| -> Cow<str> { s.to_uppercase().into() };

    b.iter(|| {
      RefInputBuilder::<str, Cow<str>>::default()
        .required(black_box(true))
        .break_on_failure(black_box(true))
        .name(black_box("test"))
        .locale(black_box("en_US"))
        .validators(vec![&validator])
        .filters(vec![&filter])
        .build()
        .unwrap()
    });
  });
}

fn bench_ref_input_validate(c: &mut Criterion) {
  let validator = |s: &str| {
    if s.len() > 5 {
      Ok(())
    } else {
      Err(Violation(TypeMismatch, "Too short".to_string()))
    }
  };

  let input = RefInputBuilder::<str, Cow<str>>::default()
    .required(true)
    .validators(vec![&validator])
    .build()
    .unwrap();

  c.bench_function("ref_input_validate_pass", |b| {
    b.iter(|| input.validate_ref(black_box("Hello, World!")));
  });

  c.bench_function("ref_input_validate_fail", |b| {
    b.iter(|| input.validate_ref(black_box("Hi")));
  });

  c.bench_function("ref_input_validate_option_some", |b| {
    b.iter(|| input.validate_ref_option(black_box(Some("Hello, World!"))));
  });

  c.bench_function("ref_input_validate_option_none", |b| {
    b.iter(|| input.validate_ref_option(black_box(None)));
  });
}

fn bench_ref_input_filter(c: &mut Criterion) {
  let filter = |s: Cow<str>| -> Cow<str> { s.to_uppercase().into() };

  let input = RefInputBuilder::<str, Cow<str>>::default()
    .filters(vec![&filter])
    .build()
    .unwrap();

  c.bench_function("ref_input_filter", |b| {
    b.iter(|| input.filter_ref(black_box("hello")));
  });

  c.bench_function("ref_input_filter_option_some", |b| {
    b.iter(|| input.filter_ref_option(black_box(Some("hello"))));
  });

  c.bench_function("ref_input_fn_call", |b| {
    b.iter(|| input(black_box("hello")));
  });
}

criterion_group!(
  benches,
  bench_input_creation,
  bench_input_validate,
  bench_input_filter,
  bench_ref_input_creation,
  bench_ref_input_validate,
  bench_ref_input_filter,
);
criterion_main!(benches);

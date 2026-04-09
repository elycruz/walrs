//! Benchmarks for walrs_filter
//!
//! Run with: `cargo bench -p walrs_filter`

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::borrow::Cow;
use walrs_filter::{Filter, FilterOp, SlugFilter, StripTagsFilter, XmlEntitiesFilter};

fn bench_slug_filter(c: &mut Criterion) {
  let mut group = c.benchmark_group("SlugFilter");

  let filter = SlugFilter::new(200, false);
  let filter_with_dashes = SlugFilter::new(200, true);

  let inputs = [
    ("short", "Hello World"),
    ("medium", "This is a Medium Length Title for Testing"),
    (
      "long",
      "This is a Much Longer Title That Contains Many Words and Should Test the Performance of the Slug Filter Implementation",
    ),
    ("special_chars", "Hello!@#$%^&*()World_Test-123"),
    ("noop", "hello-world"),
    ("noop_underscores", "hello_world_test"),
  ];

  for (name, input) in inputs {
    group.bench_with_input(BenchmarkId::new("pretty_slug", name), &input, |b, input| {
      b.iter(|| filter.filter(black_box(Cow::Borrowed(*input))))
    });

    group.bench_with_input(BenchmarkId::new("with_dashes", name), &input, |b, input| {
      b.iter(|| filter_with_dashes.filter(black_box(Cow::Borrowed(*input))))
    });
  }

  group.finish();
}

fn bench_strip_tags_filter(c: &mut Criterion) {
  let mut group = c.benchmark_group("StripTagsFilter");

  let filter = StripTagsFilter::new();

  let inputs = [
    ("no_tags", "Hello World, this is plain text"),
    ("simple_tags", "<p>Hello World</p>"),
    ("nested_tags", "<div><p>Hello <b>World</b></p></div>"),
    (
      "script_tags",
      "<script>alert('xss')</script>Safe content here",
    ),
    (
      "complex_html",
      r#"<html><head><style>body{color:red}</style></head><body><div class="container"><h1>Title</h1><p>Content with <a href="link">links</a></p></div></body></html>"#,
    ),
    ("noop_plain", "Already clean text with no tags at all"),
    ("noop_empty", ""),
  ];

  for (name, input) in inputs {
    group.bench_with_input(BenchmarkId::new("filter", name), &input, |b, input| {
      b.iter(|| filter.filter(black_box(Cow::Borrowed(*input))))
    });
  }

  group.finish();
}

fn bench_xml_entities_filter(c: &mut Criterion) {
  let mut group = c.benchmark_group("XmlEntitiesFilter");

  let filter = XmlEntitiesFilter::new();

  let inputs = [
    ("no_entities", "Hello World plain text"),
    ("few_entities", "Hello & Goodbye"),
    (
      "many_entities",
      "5 < 10 & 10 > 5 and \"quotes\" with 'apostrophes'",
    ),
    ("all_special", "<>&\"'<>&\"'<>&\"'"),
    ("noop_plain", "Already clean text no special chars"),
    ("noop_numbers", "12345678901234567890"),
  ];

  for (name, input) in inputs {
    group.bench_with_input(BenchmarkId::new("filter", name), &input, |b, input| {
      b.iter(|| filter.filter(black_box(Cow::Borrowed(*input))))
    });
  }

  group.finish();
}

fn bench_filter_comparison(c: &mut Criterion) {
  let mut group = c.benchmark_group("FilterComparison");

  let slug_filter = SlugFilter::new(200, false);
  let strip_filter = StripTagsFilter::new();
  let xml_filter = XmlEntitiesFilter::new();

  let input = "<p>Hello World & Friends</p>";

  group.bench_function("slug_filter", |b| {
    b.iter(|| slug_filter.filter(black_box(Cow::Borrowed(input))))
  });

  group.bench_function("strip_tags_filter", |b| {
    b.iter(|| strip_filter.filter(black_box(Cow::Borrowed(input))))
  });

  group.bench_function("xml_entities_filter", |b| {
    b.iter(|| xml_filter.filter(black_box(Cow::Borrowed(input))))
  });

  group.finish();
}

fn bench_filter_op_noop(c: &mut Criterion) {
  let mut group = c.benchmark_group("FilterOp_noop_vs_mutation");

  // Trim
  let trim = FilterOp::<String>::Trim;
  group.bench_function("trim_noop", |b| {
    b.iter(|| trim.apply_ref(black_box("already_trimmed")))
  });
  group.bench_function("trim_mutation", |b| {
    b.iter(|| trim.apply_ref(black_box("  needs trimming  ")))
  });

  // Lowercase
  let lower = FilterOp::<String>::Lowercase;
  group.bench_function("lowercase_noop", |b| {
    b.iter(|| lower.apply_ref(black_box("already lowercase")))
  });
  group.bench_function("lowercase_mutation", |b| {
    b.iter(|| lower.apply_ref(black_box("NEEDS LOWERING")))
  });

  // Uppercase
  let upper = FilterOp::<String>::Uppercase;
  group.bench_function("uppercase_noop", |b| {
    b.iter(|| upper.apply_ref(black_box("ALREADY UPPERCASE")))
  });
  group.bench_function("uppercase_mutation", |b| {
    b.iter(|| upper.apply_ref(black_box("needs uppering")))
  });

  // StripTags
  let strip = FilterOp::<String>::StripTags;
  group.bench_function("strip_tags_noop", |b| {
    b.iter(|| strip.apply_ref(black_box("no tags here")))
  });
  group.bench_function("strip_tags_mutation", |b| {
    b.iter(|| strip.apply_ref(black_box("<p>has tags</p>")))
  });

  // HtmlEntities
  let entities = FilterOp::<String>::HtmlEntities;
  group.bench_function("html_entities_noop", |b| {
    b.iter(|| entities.apply_ref(black_box("no special chars")))
  });
  group.bench_function("html_entities_mutation", |b| {
    b.iter(|| entities.apply_ref(black_box("has <special> & chars")))
  });

  // Slug
  let slug = FilterOp::<String>::Slug { max_length: None };
  group.bench_function("slug_noop", |b| {
    b.iter(|| slug.apply_ref(black_box("already-a-slug")))
  });
  group.bench_function("slug_mutation", |b| {
    b.iter(|| slug.apply_ref(black_box("Needs Slug Conversion!")))
  });

  group.finish();
}

criterion_group!(
  benches,
  bench_slug_filter,
  bench_strip_tags_filter,
  bench_xml_entities_filter,
  bench_filter_comparison,
  bench_filter_op_noop,
);

criterion_main!(benches);

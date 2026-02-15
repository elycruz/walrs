//! Benchmarks for walrs_filter
//!
//! Run with: `cargo bench -p walrs_filter`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::borrow::Cow;
use walrs_filter::{Filter, SlugFilter, StripTagsFilter, XmlEntitiesFilter};

fn bench_slug_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("SlugFilter");

    let filter = SlugFilter::new(200, false);
    let filter_with_dashes = SlugFilter::new(200, true);

    let inputs = [
        ("short", "Hello World"),
        ("medium", "This is a Medium Length Title for Testing"),
        ("long", "This is a Much Longer Title That Contains Many Words and Should Test the Performance of the Slug Filter Implementation"),
        ("special_chars", "Hello!@#$%^&*()World_Test-123"),
    ];

    for (name, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("pretty_slug", name),
            &input,
            |b, input| {
                b.iter(|| filter.filter(black_box(Cow::Borrowed(*input))))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_dashes", name),
            &input,
            |b, input| {
                b.iter(|| filter_with_dashes.filter(black_box(Cow::Borrowed(*input))))
            },
        );
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
        ("script_tags", "<script>alert('xss')</script>Safe content here"),
        ("complex_html", r#"<html><head><style>body{color:red}</style></head><body><div class="container"><h1>Title</h1><p>Content with <a href="link">links</a></p></div></body></html>"#),
    ];

    for (name, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("filter", name),
            &input,
            |b, input| {
                b.iter(|| filter.filter(black_box(Cow::Borrowed(*input))))
            },
        );
    }

    group.finish();
}

fn bench_xml_entities_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("XmlEntitiesFilter");

    let filter = XmlEntitiesFilter::new();

    let inputs = [
        ("no_entities", "Hello World plain text"),
        ("few_entities", "Hello & Goodbye"),
        ("many_entities", "5 < 10 & 10 > 5 and \"quotes\" with 'apostrophes'"),
        ("all_special", "<>&\"'<>&\"'<>&\"'"),
    ];

    for (name, input) in inputs {
        group.bench_with_input(
            BenchmarkId::new("filter", name),
            &input,
            |b, input| {
                b.iter(|| filter.filter(black_box(Cow::Borrowed(*input))))
            },
        );
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

criterion_group!(
    benches,
    bench_slug_filter,
    bench_strip_tags_filter,
    bench_xml_entities_filter,
    bench_filter_comparison,
);

criterion_main!(benches);

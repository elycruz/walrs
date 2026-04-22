//! Benchmarks for walrs_filter
//!
//! Run with: `cargo bench -p walrs_filter`

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::borrow::Cow;
use std::sync::Arc;
use walrs_filter::{Filter, FilterOp, SlugFilter, StripTagsFilter, TryFilterOp, XmlEntitiesFilter};

const NORMALIZE_WHITESPACE_DIRTY_INPUT: &str = r#"  Lorem   ipsum	dolor

sit amet,     consectetur	 adipiscing elit.  
  Sed		do  eiusmod   tempor

	incididunt ut labore    et dolore
magna	 aliqua.   Ut

 enim	 ad minim	veniam,   quis
 nostrud
exercitation    ullamco	 laboris	nisi   ut aliquip

   ex	 ea commodo
consequat.  "#;

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
    (
      "noop_already_encoded",
      "Tom &amp; Jerry &#38; friends &#x26; co",
    ),
    (
      "mixed_raw_and_encoded",
      "Tom &amp; Jerry & AT&T with <b>tags</b>",
    ),
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

fn bench_filter_op_chain(c: &mut Criterion) {
  let mut group = c.benchmark_group("FilterOp_Chain");

  let chain_1: FilterOp<String> = FilterOp::Chain(vec![FilterOp::Trim]);
  let chain_3: FilterOp<String> = FilterOp::Chain(vec![
    FilterOp::Trim,
    FilterOp::Lowercase,
    FilterOp::StripTags,
  ]);
  let chain_5: FilterOp<String> = FilterOp::Chain(vec![
    FilterOp::Trim,
    FilterOp::Lowercase,
    FilterOp::StripTags,
    FilterOp::HtmlEntities,
    FilterOp::Slug { max_length: None },
  ]);

  let input = "  <b>Hello World & Friends</b>  ";

  group.bench_function("chain_1", |b| {
    b.iter(|| chain_1.apply_ref(black_box(input)))
  });
  group.bench_function("chain_3", |b| {
    b.iter(|| chain_3.apply_ref(black_box(input)))
  });
  group.bench_function("chain_5", |b| {
    b.iter(|| chain_5.apply_ref(black_box(input)))
  });

  group.finish();
}

fn bench_filter_op_sanitize(c: &mut Criterion) {
  let mut group = c.benchmark_group("FilterOp_Sanitize");

  // Digits — noop vs mutation
  let digits = FilterOp::<String>::Digits;
  group.bench_function("digits_noop", |b| {
    b.iter(|| digits.apply_ref(black_box("1234567890")))
  });
  group.bench_function("digits_mutation", |b| {
    b.iter(|| digits.apply_ref(black_box("abc123-def!456")))
  });

  // NormalizeWhitespace — single-pass early-exit scan on clean input,
  // full rebuild on dirty input.
  let normalize = FilterOp::<String>::NormalizeWhitespace;
  group.bench_function("normalize_whitespace_noop", |b| {
    b.iter(|| normalize.apply_ref(black_box("hello world go")))
  });
  group.bench_function("normalize_whitespace_mutation", |b| {
    b.iter(|| normalize.apply_ref(black_box(NORMALIZE_WHITESPACE_DIRTY_INPUT)))
  });

  // AllowChars / DenyChars
  let allow = FilterOp::<String>::AllowChars {
    set: "abcdefghijklmnopqrstuvwxyz ".to_string(),
  };
  group.bench_function("allow_chars_noop", |b| {
    b.iter(|| allow.apply_ref(black_box("hello world")))
  });
  group.bench_function("allow_chars_mutation", |b| {
    b.iter(|| allow.apply_ref(black_box("Hello, World! 123")))
  });

  let deny = FilterOp::<String>::DenyChars {
    set: "<>&\"'".to_string(),
  };
  group.bench_function("deny_chars_noop", |b| {
    b.iter(|| deny.apply_ref(black_box("plain safe text")))
  });
  group.bench_function("deny_chars_mutation", |b| {
    b.iter(|| deny.apply_ref(black_box("<script>alert(\"xss\")</script>")))
  });

  // UrlEncode — RFC 3986 mode, noop vs mutation.
  let url_encode = FilterOp::<String>::UrlEncode {
    encode_unreserved: false,
  };
  group.bench_function("url_encode_noop_alphanum", |b| {
    b.iter(|| url_encode.apply_ref(black_box("HelloWorld123")))
  });
  group.bench_function("url_encode_mutation", |b| {
    b.iter(|| url_encode.apply_ref(black_box("hello world&foo=bar")))
  });

  // StripNewlines
  let strip_nl = FilterOp::<String>::StripNewlines;
  group.bench_function("strip_newlines_noop", |b| {
    b.iter(|| strip_nl.apply_ref(black_box("no newlines here at all")))
  });
  group.bench_function("strip_newlines_mutation", |b| {
    b.iter(|| strip_nl.apply_ref(black_box("line1\nline2\r\nline3\rline4")))
  });

  // Alnum — Unicode path
  let alnum = FilterOp::<String>::Alnum {
    allow_whitespace: false,
  };
  group.bench_function("alnum_ascii_noop", |b| {
    b.iter(|| alnum.apply_ref(black_box("abc123")))
  });
  group.bench_function("alnum_unicode_mutation", |b| {
    b.iter(|| alnum.apply_ref(black_box("café-日本語!")))
  });

  group.finish();
}

fn bench_try_filter_op_conversions(c: &mut Criterion) {
  let mut group = c.benchmark_group("TryFilterOp_Conversions");

  // ToBool — already-canonical (expected borrowed, no alloc) vs permissive.
  let to_bool = TryFilterOp::<String>::ToBool;
  group.bench_function("to_bool_canonical_true", |b| {
    b.iter(|| to_bool.try_apply_ref(black_box("true")).unwrap())
  });
  group.bench_function("to_bool_canonical_false", |b| {
    b.iter(|| to_bool.try_apply_ref(black_box("false")).unwrap())
  });
  group.bench_function("to_bool_permissive_yes", |b| {
    b.iter(|| to_bool.try_apply_ref(black_box("YES")).unwrap())
  });

  // ToInt — canonical borrowed vs needing trim/strip.
  let to_int = TryFilterOp::<String>::ToInt;
  group.bench_function("to_int_canonical", |b| {
    b.iter(|| to_int.try_apply_ref(black_box("42")).unwrap())
  });
  group.bench_function("to_int_with_whitespace", |b| {
    b.iter(|| to_int.try_apply_ref(black_box("  042  ")).unwrap())
  });

  // UrlDecode — plain text (borrowed) vs real decoding.
  let url_decode = TryFilterOp::<String>::UrlDecode;
  group.bench_function("url_decode_noop", |b| {
    b.iter(|| url_decode.try_apply_ref(black_box("plaintext")).unwrap())
  });
  group.bench_function("url_decode_mutation", |b| {
    b.iter(|| {
      url_decode
        .try_apply_ref(black_box("hello%20world%20caf%C3%A9"))
        .unwrap()
    })
  });

  group.finish();
}

fn bench_filter_op_clamp(c: &mut Criterion) {
  let mut group = c.benchmark_group("FilterOp_Clamp");

  let clamp_i32 = FilterOp::<i32>::Clamp { min: 0, max: 100 };
  let clamp_f64 = FilterOp::<f64>::Clamp {
    min: 0.0,
    max: 100.0,
  };

  group.bench_function("clamp_i32_in_range", |b| {
    b.iter(|| clamp_i32.apply(black_box(50_i32)))
  });
  group.bench_function("clamp_i32_below_min", |b| {
    b.iter(|| clamp_i32.apply(black_box(-10_i32)))
  });
  group.bench_function("clamp_i32_above_max", |b| {
    b.iter(|| clamp_i32.apply(black_box(200_i32)))
  });
  group.bench_function("clamp_f64_in_range", |b| {
    b.iter(|| clamp_f64.apply(black_box(50.0_f64)))
  });
  group.bench_function("clamp_f64_below_min", |b| {
    b.iter(|| clamp_f64.apply(black_box(-10.0_f64)))
  });
  group.bench_function("clamp_f64_above_max", |b| {
    b.iter(|| clamp_f64.apply(black_box(200.0_f64)))
  });

  group.finish();
}

fn bench_try_filter_op(c: &mut Criterion) {
  let mut group = c.benchmark_group("TryFilterOp");

  // Infallible wrapping cost
  let infallible: TryFilterOp<String> = TryFilterOp::Infallible(FilterOp::Trim);
  group.bench_function("infallible_trim_noop", |b| {
    b.iter(|| {
      infallible
        .try_apply_ref(black_box("already_trimmed"))
        .unwrap()
    })
  });
  group.bench_function("infallible_trim_mutation", |b| {
    b.iter(|| {
      infallible
        .try_apply_ref(black_box("  needs trimming  "))
        .unwrap()
    })
  });

  // TryCustom success path
  let try_custom: TryFilterOp<String> =
    TryFilterOp::TryCustom(Arc::new(|s: String| Ok(s.to_uppercase())));
  group.bench_function("try_custom_success", |b| {
    b.iter(|| {
      try_custom
        .try_apply(black_box("hello".to_string()))
        .unwrap()
    })
  });

  // Chain with infallible ops
  let chain: TryFilterOp<String> = TryFilterOp::Chain(vec![
    TryFilterOp::Infallible(FilterOp::Trim),
    TryFilterOp::Infallible(FilterOp::Lowercase),
    TryFilterOp::Infallible(FilterOp::StripTags),
  ]);
  group.bench_function("chain_3_infallible", |b| {
    b.iter(|| chain.try_apply_ref(black_box("  <b>HELLO</b>  ")).unwrap())
  });

  // Chain that short-circuits on error
  let fail_on_empty: TryFilterOp<String> = TryFilterOp::Chain(vec![
    TryFilterOp::Infallible(FilterOp::Trim),
    TryFilterOp::TryCustom(Arc::new(|s: String| {
      if s.is_empty() {
        Err(walrs_filter::FilterError::new("empty"))
      } else {
        Ok(s)
      }
    })),
  ]);
  group.bench_function("chain_short_circuit_ok", |b| {
    b.iter(|| {
      fail_on_empty
        .try_apply(black_box("hello".to_string()))
        .unwrap()
    })
  });
  group.bench_function("chain_short_circuit_err", |b| {
    b.iter(|| fail_on_empty.try_apply(black_box("".to_string())).is_err())
  });

  group.finish();
}

#[cfg(feature = "validation")]
fn bench_filter_op_value(c: &mut Criterion) {
  use walrs_validation::Value;

  let mut group = c.benchmark_group("FilterOp_Value");

  let trim = FilterOp::<Value>::Trim;
  group.bench_function("trim_str_noop", |b| {
    b.iter(|| trim.apply_ref(black_box(&Value::Str("already_trimmed".to_string()))))
  });
  group.bench_function("trim_str_mutation", |b| {
    b.iter(|| trim.apply_ref(black_box(&Value::Str("  needs trimming  ".to_string()))))
  });
  group.bench_function("trim_non_str_passthrough", |b| {
    b.iter(|| trim.apply_ref(black_box(&Value::I64(42))))
  });

  let clamp = FilterOp::<Value>::Clamp {
    min: Value::I64(0),
    max: Value::I64(100),
  };
  group.bench_function("clamp_i64_in_range", |b| {
    b.iter(|| clamp.apply_ref(black_box(&Value::I64(50))))
  });
  group.bench_function("clamp_i64_above_max", |b| {
    b.iter(|| clamp.apply_ref(black_box(&Value::I64(200))))
  });

  let chain: FilterOp<Value> = FilterOp::Chain(vec![FilterOp::Trim, FilterOp::Lowercase]);
  group.bench_function("chain_trim_lowercase", |b| {
    b.iter(|| chain.apply_ref(black_box(&Value::Str("  HELLO WORLD  ".to_string()))))
  });

  group.finish();
}

#[cfg(not(feature = "validation"))]
fn bench_filter_op_value(_c: &mut Criterion) {}

criterion_group!(
  benches,
  bench_slug_filter,
  bench_strip_tags_filter,
  bench_xml_entities_filter,
  bench_filter_comparison,
  bench_filter_op_noop,
  bench_filter_op_chain,
  bench_filter_op_sanitize,
  bench_try_filter_op_conversions,
  bench_filter_op_clamp,
  bench_try_filter_op,
  bench_filter_op_value,
);

criterion_main!(benches);

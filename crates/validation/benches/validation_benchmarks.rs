use criterion::{criterion_group, criterion_main, Criterion};
use walrs_validation::{Rule, Validate, ValidateRef};

// ============================================================================
// String validation benchmarks
// ============================================================================

fn bench_string_email(c: &mut Criterion) {
    use walrs_validation::options::EmailOptions;
    let rule = Rule::<String>::Email(EmailOptions::default());
    c.bench_function("email valid", |b| {
        b.iter(|| rule.validate_ref("user@example.com"))
    });
    c.bench_function("email invalid", |b| {
        b.iter(|| rule.validate_ref("not-an-email"))
    });
}

fn bench_string_pattern(c: &mut Criterion) {
    let rule = Rule::<String>::pattern(r"^[a-zA-Z0-9_\-]{3,30}$").unwrap();
    let compiled = rule.clone().compile();

    c.bench_function("pattern rule", |b| {
        b.iter(|| rule.validate_ref("hello_world"))
    });
    c.bench_function("pattern compiled rule", |b| {
        b.iter(|| compiled.validate_ref("hello_world"))
    });
}

fn bench_string_length(c: &mut Criterion) {
    let rule = Rule::<String>::MinLength(3).and(Rule::MaxLength(50));
    c.bench_function("string length check", |b| {
        b.iter(|| rule.validate_ref("hello world"))
    });
}

fn bench_string_url(c: &mut Criterion) {
    use walrs_validation::options::UrlOptions;
    let rule = Rule::<String>::Url(UrlOptions::default());
    c.bench_function("url valid", |b| {
        b.iter(|| rule.validate_ref("https://www.example.com/path?query=1"))
    });
}

fn bench_string_ip(c: &mut Criterion) {
    use walrs_validation::options::IpOptions;
    let rule = Rule::<String>::Ip(IpOptions::default());
    c.bench_function("ipv4 valid", |b| {
        b.iter(|| rule.validate_ref("192.168.1.1"))
    });
}

fn bench_string_hostname(c: &mut Criterion) {
    use walrs_validation::options::HostnameOptions;
    let rule = Rule::<String>::Hostname(HostnameOptions::default());
    c.bench_function("hostname valid", |b| {
        b.iter(|| rule.validate_ref("example.com"))
    });
}

// ============================================================================
// Numeric validation benchmarks
// ============================================================================

fn bench_numeric_min_max(c: &mut Criterion) {
    let rule = Rule::<i32>::Min(0).and(Rule::Max(1000));
    c.bench_function("numeric min/max valid", |b| {
        b.iter(|| rule.validate(500))
    });
    c.bench_function("numeric min/max invalid", |b| {
        b.iter(|| rule.validate(-1))
    });
}

fn bench_numeric_range(c: &mut Criterion) {
    let rule = Rule::<i32>::Range { min: 0, max: 1000 };
    c.bench_function("numeric range valid", |b| {
        b.iter(|| rule.validate(500))
    });
}

fn bench_numeric_step(c: &mut Criterion) {
    let rule = Rule::<i32>::Step(5);
    c.bench_function("numeric step valid", |b| {
        b.iter(|| rule.validate(25))
    });
}

// ============================================================================
// Composite rule benchmarks
// ============================================================================

fn bench_composite_all(c: &mut Criterion) {
    let rule = Rule::<String>::Required
        .and(Rule::MinLength(3))
        .and(Rule::MaxLength(50))
        .and(Rule::pattern(r"^[a-zA-Z0-9_]+$").unwrap());

    c.bench_function("composite All (4 rules) valid", |b| {
        b.iter(|| rule.validate_ref("hello_world"))
    });
    c.bench_function("composite All (4 rules) invalid", |b| {
        b.iter(|| rule.validate_ref("hi"))
    });
}

fn bench_composite_any(c: &mut Criterion) {
    let rule = Rule::<i32>::Equals(0)
        .or(Rule::Equals(100))
        .or(Rule::Equals(200));

    c.bench_function("composite Any (3 rules) first match", |b| {
        b.iter(|| rule.validate(0))
    });
    c.bench_function("composite Any (3 rules) last match", |b| {
        b.iter(|| rule.validate(200))
    });
}

// ============================================================================
// CompiledRule vs uncompiled
// ============================================================================

fn bench_compiled_vs_uncompiled(c: &mut Criterion) {
    let rule = Rule::<String>::pattern(r"^[a-z]{3,30}$").unwrap();
    let compiled = rule.clone().compile();
    let value = "hello";

    c.bench_function("pattern rule (pre-compiled regex)", |b| {
        b.iter(|| rule.validate_ref(value))
    });
    c.bench_function("compiled rule (cached wrapper)", |b| {
        b.iter(|| compiled.validate_ref(value))
    });
}

criterion_group!(
    string_benches,
    bench_string_email,
    bench_string_pattern,
    bench_string_length,
    bench_string_url,
    bench_string_ip,
    bench_string_hostname,
);

criterion_group!(
    numeric_benches,
    bench_numeric_min_max,
    bench_numeric_range,
    bench_numeric_step,
);

criterion_group!(
    composite_benches,
    bench_composite_all,
    bench_composite_any,
    bench_compiled_vs_uncompiled,
);

criterion_main!(string_benches, numeric_benches, composite_benches);

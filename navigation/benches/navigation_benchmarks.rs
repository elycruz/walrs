use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use walrs_navigation::{Container, Page};

fn bench_page_creation(c: &mut Criterion) {
    c.bench_function("page_new", |b| {
        b.iter(|| Page::new());
    });

    c.bench_function("page_builder", |b| {
        b.iter(|| {
            Page::builder()
                .label(black_box("Home"))
                .uri(black_box("/"))
                .build()
        });
    });
}

fn bench_container_operations(c: &mut Criterion) {
    c.bench_function("container_new", |b| {
        b.iter(|| Container::new());
    });

    let mut group = c.benchmark_group("add_page");
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut nav = Container::new();
                for i in 0..size {
                    nav.add_page(
                        Page::builder()
                            .label(black_box(format!("Page {}", i)))
                            .uri(black_box(format!("/page/{}", i)))
                            .build(),
                    );
                }
            });
        });
    }
    group.finish();
}

fn bench_find_operations(c: &mut Criterion) {
    let mut nav = Container::new();
    for i in 0..1000 {
        nav.add_page(
            Page::builder()
                .label(format!("Page {}", i))
                .uri(format!("/page/{}", i))
                .build(),
        );
    }

    c.bench_function("find_by_uri", |b| {
        b.iter(|| {
            nav.find_by_uri(black_box("/page/500"));
        });
    });

    c.bench_function("find_by_label", |b| {
        b.iter(|| {
            nav.find_by_label(black_box("Page 500"));
        });
    });
}

fn bench_nested_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_add");
    for depth in [2, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            b.iter(|| {
                let mut root = Page::builder().label("Root").build();
                let mut current = &mut root;

                for i in 0..depth {
                    let child = Page::builder()
                        .label(black_box(format!("Level {}", i)))
                        .build();
                    current.add_page(child);
                    // Note: We can't easily get mutable reference to the just-added child
                    // so we're testing repeated additions at the same level instead
                }
            });
        });
    }
    group.finish();
}

fn bench_traversal(c: &mut Criterion) {
    let mut nav = Container::new();
    for i in 0..100 {
        let mut parent = Page::builder()
            .label(format!("Parent {}", i))
            .build();
        for j in 0..10 {
            parent.add_page(
                Page::builder()
                    .label(format!("Child {}-{}", i, j))
                    .build(),
            );
        }
        nav.add_page(parent);
    }

    c.bench_function("traverse_all", |b| {
        b.iter(|| {
            let mut count = 0;
            nav.traverse(&mut |_page| {
                count += 1;
            });
            black_box(count);
        });
    });
}

fn bench_serialization(c: &mut Criterion) {
    let mut nav = Container::new();
    for i in 0..50 {
        let mut parent = Page::builder()
            .label(format!("Parent {}", i))
            .uri(format!("/parent/{}", i))
            .build();
        for j in 0..5 {
            parent.add_page(
                Page::builder()
                    .label(format!("Child {}-{}", i, j))
                    .uri(format!("/parent/{}/child/{}", i, j))
                    .build(),
            );
        }
        nav.add_page(parent);
    }

    c.bench_function("to_json", |b| {
        b.iter(|| {
            nav.to_json().unwrap();
        });
    });

    c.bench_function("to_yaml", |b| {
        b.iter(|| {
            nav.to_yaml().unwrap();
        });
    });

    let json = nav.to_json().unwrap();
    c.bench_function("from_json", |b| {
        b.iter(|| {
            Container::from_json(black_box(&json)).unwrap();
        });
    });

    let yaml = nav.to_yaml().unwrap();
    c.bench_function("from_yaml", |b| {
        b.iter(|| {
            Container::from_yaml(black_box(&yaml)).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_page_creation,
    bench_container_operations,
    bench_find_operations,
    bench_nested_operations,
    bench_traversal,
    bench_serialization
);
criterion_main!(benches);

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use walrs_navigation::{Container, Page};

fn bench_page_creation(c: &mut Criterion) {
    c.bench_function("page_new", |b| {
        b.iter(Page::new);
    });

    c.bench_function("page_builder", |b| {
        b.iter(|| {
            Page::builder()
                .label(black_box("Home"))
                .uri(black_box("/"))
                .build()
        });
    });

    c.bench_function("page_builder_full", |b| {
        b.iter(|| {
            Page::builder()
                .label(black_box("Home"))
                .uri(black_box("/"))
                .title(black_box("Home Page"))
                .fragment(black_box("top"))
                .route(black_box("home"))
                .resource(black_box("mvc:home"))
                .privilege(black_box("view"))
                .class(black_box("nav-item"))
                .id(black_box("home"))
                .target(black_box("_self"))
                .order(1)
                .attribute("data-id", "1")
                .build()
        });
    });
}

fn bench_container_operations(c: &mut Criterion) {
    c.bench_function("container_new", |b| {
        b.iter(Container::new);
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

    c.bench_function("add_pages_100", |b| {
        b.iter(|| {
            let mut nav = Container::new();
            let pages: Vec<Page> = (0..100)
                .map(|i| {
                    Page::builder()
                        .label(format!("Page {}", i))
                        .uri(format!("/page/{}", i))
                        .build()
                })
                .collect();
            nav.add_pages(pages);
        });
    });
}

fn bench_find_operations(c: &mut Criterion) {
    let mut nav = Container::new();
    for i in 0..1000 {
        nav.add_page(
            Page::builder()
                .label(format!("Page {}", i))
                .uri(format!("/page/{}", i))
                .class(if i % 2 == 0 { "even" } else { "odd" })
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

    c.bench_function("has_page_non_recursive", |b| {
        b.iter(|| {
            nav.has_page(|p| p.uri.as_deref() == Some(black_box("/page/500")), false);
        });
    });
}

fn bench_nested_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_add");
    for depth in [2, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            b.iter(|| {
                let mut root = Page::builder().label("Root").build();
                let current = &mut root;

                for i in 0..depth {
                    let child = Page::builder()
                        .label(black_box(format!("Level {}", i)))
                        .build();
                    current.add_page(child);
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

    c.bench_function("traverse_with_depth", |b| {
        b.iter(|| {
            let mut max_depth = 0;
            nav.traverse_with_depth(&mut |_page, depth| {
                if depth > max_depth {
                    max_depth = depth;
                }
            });
            black_box(max_depth);
        });
    });
}

fn bench_breadcrumbs(c: &mut Criterion) {
    let mut nav = Container::new();
    let mut current = Page::builder().label("L0").uri("/l0").build();
    // Build 10 levels deep
    for i in (1..10).rev() {
        let mut child = Page::builder()
            .label(format!("L{}", i))
            .uri(format!("/l{}", i))
            .build();
        if i == 9 {
            child.active = true;
        }
        current.add_page(child);
        let mut wrapper = Page::builder()
            .label(format!("W{}", i))
            .uri(format!("/w{}", i))
            .build();
        wrapper.add_page(current);
        current = wrapper;
    }
    nav.add_page(current);

    c.bench_function("breadcrumbs", |b| {
        b.iter(|| {
            black_box(nav.breadcrumbs());
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

fn bench_view_helpers(c: &mut Criterion) {
    use walrs_navigation::view;

    let mut nav = Container::new();
    for i in 0..20 {
        let mut parent = Page::builder()
            .label(format!("Section {}", i))
            .uri(format!("/section/{}", i))
            .build();
        for j in 0..5 {
            parent.add_page(
                Page::builder()
                    .label(format!("Item {}-{}", i, j))
                    .uri(format!("/section/{}/item/{}", i, j))
                    .build(),
            );
        }
        nav.add_page(parent);
    }

    // Set one page active for breadcrumb testing
    let mut nav_with_active = nav.clone();
    nav_with_active.set_active_by_uri("/section/10/item/3");

    c.bench_function("render_menu", |b| {
        b.iter(|| {
            black_box(view::render_menu(&nav));
        });
    });

    c.bench_function("render_breadcrumbs", |b| {
        b.iter(|| {
            black_box(view::render_breadcrumbs(&nav_with_active, " > "));
        });
    });

    c.bench_function("render_sitemap", |b| {
        b.iter(|| {
            black_box(view::render_sitemap(&nav));
        });
    });

    c.bench_function("render_sitemap_hierarchical", |b| {
        b.iter(|| {
            black_box(view::render_sitemap_hierarchical(&nav));
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
    bench_breadcrumbs,
    bench_serialization,
    bench_view_helpers
);
criterion_main!(benches);

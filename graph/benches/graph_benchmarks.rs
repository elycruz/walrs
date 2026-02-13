use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use walrs_graph::Graph;

fn bench_graph_new(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_new");

    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| Graph::new(black_box(size)));
        });
    }
    group.finish();
}

fn bench_add_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_edge");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut g = Graph::new(size);
                for i in 0..size-1 {
                    g.add_edge(black_box(i), black_box(i + 1)).unwrap();
                }
            });
        });
    }
    group.finish();
}

fn bench_adj(c: &mut Criterion) {
    let mut group = c.benchmark_group("adj");

    for size in [10, 100, 1000].iter() {
        // Setup: Create a graph with edges
        let mut g = Graph::new(*size);
        for i in 0..*size-1 {
            g.add_edge(i, i + 1).unwrap();
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    black_box(g.adj(black_box(i)).unwrap());
                }
            });
        });
    }
    group.finish();
}

fn bench_degree(c: &mut Criterion) {
    let mut group = c.benchmark_group("degree");

    for size in [10, 100, 1000].iter() {
        // Setup: Create a graph with edges
        let mut g = Graph::new(*size);
        for i in 0..*size-1 {
            g.add_edge(i, i + 1).unwrap();
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    black_box(g.degree(black_box(i)).unwrap());
                }
            });
        });
    }
    group.finish();
}

fn bench_has_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("has_edge");

    for size in [10, 100, 1000].iter() {
        // Setup: Create a graph with edges
        let mut g = Graph::new(*size);
        for i in 0..*size-1 {
            g.add_edge(i, i + 1).unwrap();
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for i in 0..size-1 {
                    black_box(g.has_edge(black_box(i), black_box(i + 1)));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_graph_new,
    bench_add_edge,
    bench_adj,
    bench_degree,
    bench_has_edge
);
criterion_main!(benches);

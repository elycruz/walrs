use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use walrs_digraph::{DisymGraph, DisymGraphData};

fn names(size: usize) -> Vec<String> {
  (0..size).map(|i| format!("s{}", i)).collect()
}

/// Builds a chain graph `s0 -> s1 -> ... -> s{size-1}`.
fn chain(size: usize) -> DisymGraph {
  let names = names(size);
  let mut g = DisymGraph::new();
  for i in 0..size - 1 {
    g.add_edge(&names[i], &[&names[i + 1]]).unwrap();
  }
  g
}

fn bench_add_edge(c: &mut Criterion) {
  let mut group = c.benchmark_group("disymgraph_add_edge");

  for size in [10usize, 100, 1000].iter() {
    let names = names(*size);
    group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
      b.iter(|| {
        let mut g = DisymGraph::new();
        for i in 0..size - 1 {
          g.add_edge(black_box(&names[i]), black_box(&[&names[i + 1]]))
            .unwrap();
        }
        g
      });
    });
  }
  group.finish();
}

fn bench_add_vertex_dedupe(c: &mut Criterion) {
  let mut group = c.benchmark_group("disymgraph_add_vertex_dedupe");

  for size in [10usize, 100, 1000].iter() {
    let names = names(*size);
    let mut g = chain(*size);
    group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
      b.iter(|| {
        for name in names.iter().take(size) {
          black_box(g.add_vertex(black_box(name)));
        }
      });
    });
  }
  group.finish();
}

fn bench_lookups(c: &mut Criterion) {
  let mut group = c.benchmark_group("disymgraph_lookups");

  for size in [10usize, 100, 1000].iter() {
    let names = names(*size);
    let g = chain(*size);

    group.bench_with_input(BenchmarkId::new("index", size), size, |b, &size| {
      b.iter(|| {
        for name in names.iter().take(size) {
          black_box(g.index(black_box(name)));
        }
      });
    });

    group.bench_with_input(BenchmarkId::new("name_as_ref", size), size, |b, &size| {
      b.iter(|| {
        for i in 0..size {
          black_box(g.name_as_ref(black_box(i)));
        }
      });
    });

    group.bench_with_input(BenchmarkId::new("adj", size), size, |b, &size| {
      b.iter(|| {
        for name in names.iter().take(size) {
          black_box(g.adj(black_box(name)));
        }
      });
    });
  }
  group.finish();
}

fn bench_try_from_data(c: &mut Criterion) {
  let mut group = c.benchmark_group("disymgraph_try_from_data");

  for size in [10usize, 100, 1000].iter() {
    let names = names(*size);
    // Every edge target is also listed as its own entry.
    let data: DisymGraphData = (0..*size)
      .map(|i| {
        let edges = if i + 1 < *size {
          Some(vec![names[i + 1].clone()])
        } else {
          None
        };
        (names[i].clone(), edges)
      })
      .collect();

    group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
      b.iter(|| DisymGraph::try_from(black_box(data)).unwrap());
    });
  }
  group.finish();
}

fn bench_reverse(c: &mut Criterion) {
  let mut group = c.benchmark_group("disymgraph_reverse");

  for size in [10usize, 100, 1000].iter() {
    let g = chain(*size);
    group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
      b.iter(|| black_box(&g).reverse().unwrap());
    });
  }
  group.finish();
}

criterion_group!(
  benches,
  bench_add_edge,
  bench_add_vertex_dedupe,
  bench_lookups,
  bench_try_from_data,
  bench_reverse
);
criterion_main!(benches);

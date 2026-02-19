use criterion::{Criterion, criterion_group, criterion_main};
use walrs_rbac::RbacBuilder;

fn build_rbac() -> walrs_rbac::Rbac {
  RbacBuilder::new()
    .add_role("guest", &["read.public"], None)
    .unwrap()
    .add_role("user", &["write.post", "comment.post"], Some(&["guest"]))
    .unwrap()
    .add_role("editor", &["edit.post", "publish.post"], Some(&["user"]))
    .unwrap()
    .add_role("admin", &["admin.panel", "manage.users"], Some(&["editor"]))
    .unwrap()
    .build()
    .unwrap()
}

fn benchmark_build(c: &mut Criterion) {
  c.bench_function("rbac_build", |b| {
    b.iter(build_rbac);
  });
}

fn benchmark_is_granted(c: &mut Criterion) {
  let rbac = build_rbac();

  c.bench_function("rbac_is_granted_direct", |b| {
    b.iter(|| rbac.is_granted("admin", "admin.panel"));
  });

  c.bench_function("rbac_is_granted_inherited", |b| {
    b.iter(|| rbac.is_granted("admin", "read.public"));
  });

  c.bench_function("rbac_is_granted_denied", |b| {
    b.iter(|| rbac.is_granted("guest", "admin.panel"));
  });
}

fn benchmark_has_role(c: &mut Criterion) {
  let rbac = build_rbac();

  c.bench_function("rbac_has_role_existing", |b| {
    b.iter(|| rbac.has_role("admin"));
  });

  c.bench_function("rbac_has_role_missing", |b| {
    b.iter(|| rbac.has_role("nonexistent"));
  });
}

criterion_group!(
  benches,
  benchmark_build,
  benchmark_is_granted,
  benchmark_has_role
);
criterion_main!(benches);

//! Benchmark demonstrating RBAC performance.
//!
//! Run with: `cargo run --example benchmark_rbac`

use walrs_rbac::RbacBuilder;
use std::time::Instant;

fn main() -> std::result::Result<(), walrs_rbac::RbacError> {
  println!("=== RBAC Performance Benchmark ===\n");

  let iterations = 100_000;

  // Benchmark build
  let start = Instant::now();
  for _ in 0..iterations {
    let _rbac = RbacBuilder::new()
      .add_role("guest", &["read.public"], None)?
      .add_role("user", &["write.post", "comment.post"], Some(&["guest"]))?
      .add_role("editor", &["edit.post", "publish.post"], Some(&["user"]))?
      .add_role("admin", &["admin.panel", "manage.users"], Some(&["editor"]))?
      .build()?;
  }
  let elapsed = start.elapsed();
  println!("Build {} iterations: {:?}", iterations, elapsed);
  println!("  Per build: {:?}\n", elapsed / iterations as u32);

  // Build one for permission checks
  let rbac = RbacBuilder::new()
    .add_role("guest", &["read.public"], None)?
    .add_role("user", &["write.post", "comment.post"], Some(&["guest"]))?
    .add_role("editor", &["edit.post", "publish.post"], Some(&["user"]))?
    .add_role("admin", &["admin.panel", "manage.users"], Some(&["editor"]))?
    .build()?;

  // Benchmark is_granted (direct permission)
  let start = Instant::now();
  for _ in 0..iterations {
    let _ = rbac.is_granted("admin", "admin.panel");
  }
  let elapsed = start.elapsed();
  println!("is_granted (direct) {} iterations: {:?}", iterations, elapsed);
  println!("  Per check: {:?}\n", elapsed / iterations as u32);

  // Benchmark is_granted (inherited permission - deepest)
  let start = Instant::now();
  for _ in 0..iterations {
    let _ = rbac.is_granted("admin", "read.public");
  }
  let elapsed = start.elapsed();
  println!("is_granted (inherited) {} iterations: {:?}", iterations, elapsed);
  println!("  Per check: {:?}\n", elapsed / iterations as u32);

  // Benchmark is_granted (denied)
  let start = Instant::now();
  for _ in 0..iterations {
    let _ = rbac.is_granted("guest", "admin.panel");
  }
  let elapsed = start.elapsed();
  println!("is_granted (denied) {} iterations: {:?}", iterations, elapsed);
  println!("  Per check: {:?}", elapsed / iterations as u32);

  Ok(())
}

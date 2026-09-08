//! Models an AWS-IAM-style policy as a directed symbol graph whose vertices carry
//! typed payloads: principals attach to policy statements, which scope resources.
//!
//! Run with: `cargo run -p walrs_digraph --example iam_policy_graph`

use serde::{Deserialize, Serialize};
use walrs_digraph::{DirectedCycle, DisymGraph, DisymGraphData, Symbol, Topology};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum Effect {
  Allow,
  Deny,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum Node {
  Principal {
    id: String,
  },
  Statement {
    id: String,
    effect: Effect,
    actions: Vec<String>,
  },
  Resource {
    id: String,
    arn: String,
  },
}

impl Symbol for Node {
  fn id(&self) -> &str {
    match self {
      Node::Principal { id } | Node::Statement { id, .. } | Node::Resource { id, .. } => id,
    }
  }
}

fn statement(id: &str, effect: Effect, actions: &[&str]) -> Node {
  Node::Statement {
    id: id.to_string(),
    effect,
    actions: actions.iter().map(|a| a.to_string()).collect(),
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut policy: DisymGraph<Node> = DisymGraph::default();

  // Vertices carry the domain payload; `id()` is what edges refer to.
  policy.add_symbol(Node::Principal { id: "alice".into() });
  policy.add_symbol(Node::Principal { id: "bob".into() });
  policy.add_symbol(statement(
    "AllowBucketRead",
    Effect::Allow,
    &["s3:GetObject", "s3:ListBucket"],
  ));
  policy.add_symbol(statement(
    "DenyBucketWrite",
    Effect::Deny,
    &["s3:PutObject"],
  ));
  policy.add_symbol(Node::Resource {
    id: "reports-bucket".into(),
    arn: "arn:aws:s3:::reports".into(),
  });

  // principal -> statement -> resource
  policy.connect("alice", &["AllowBucketRead", "DenyBucketWrite"])?;
  policy.connect("bob", &["AllowBucketRead"])?;
  policy.connect("AllowBucketRead", &["reports-bucket"])?;
  policy.connect("DenyBucketWrite", &["reports-bucket"])?;

  println!("Statements attached to alice:");
  for node in policy.adj_symbols("alice").unwrap() {
    if let Node::Statement {
      id,
      effect,
      actions,
    } = node
    {
      println!("  {id}: {effect:?} {actions:?}");
    }
  }

  println!(
    "\nHas cycle: {}",
    DirectedCycle::new(policy.graph()).has_cycle()
  );

  let order: Vec<&str> = Topology::new(policy.graph())
    .order()
    .unwrap()
    .iter()
    .map(|i| policy.name_as_ref(*i).unwrap())
    .collect();
  println!("Topological order: {}", order.join(" -> "));

  // JSON round-trip through the plain-data form; `Vec<(Node, Option<Vec<String>>)>`
  // is serde-able because `Node` is.
  let data = DisymGraphData::<Node>::try_from(&policy)?;
  let json = serde_json::to_string_pretty(&data)?;
  println!("\nSerialized policy graph:\n{json}");

  let restored: DisymGraphData<Node> = serde_json::from_str(&json)?;
  let policy2 = DisymGraph::try_from(&restored)?;
  assert_eq!(policy2.vert_count(), policy.vert_count());
  assert_eq!(policy2.edge_count(), policy.edge_count());
  assert_eq!(policy2.symbol(2), policy.symbol(2));
  println!(
    "\nRound-trip OK: {} vertices, {} edges",
    policy2.vert_count(),
    policy2.edge_count()
  );

  Ok(())
}

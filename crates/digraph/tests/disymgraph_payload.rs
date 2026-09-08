//! `DisymGraph<T>` with a non-string payload type (IAM-policy style statements).

use std::io::BufReader;

use walrs_digraph::{
  invalid_vert_symbol_msg, DigraphDFSShape, DirectedCycle, DirectedPathsDFS, DisymGraph,
  DisymGraphData, Symbol, Topology,
};

#[derive(Debug, Clone, PartialEq)]
enum Effect {
  Allow,
  Deny,
}

#[derive(Debug, Clone, PartialEq)]
struct Stmt {
  id: String,
  effect: Effect,
  actions: Vec<String>,
}

impl Stmt {
  fn new(id: &str, effect: Effect, actions: &[&str]) -> Self {
    Stmt {
      id: id.to_string(),
      effect,
      actions: actions.iter().map(|a| a.to_string()).collect(),
    }
  }
}

impl Symbol for Stmt {
  fn id(&self) -> &str {
    &self.id
  }
}

/// A payload that can also be built from a bare id (enables `add_vertex`/`add_edge`/file loaders).
#[derive(Debug, Clone, PartialEq)]
struct Named {
  name: String,
  visits: u32,
}

impl Symbol for Named {
  fn id(&self) -> &str {
    &self.name
  }
}

impl From<String> for Named {
  fn from(name: String) -> Self {
    Named { name, visits: 0 }
  }
}

/// principal -> statement -> resource chain plus a second statement on the same principal.
fn policy_graph() -> DisymGraph<Stmt> {
  let mut g: DisymGraph<Stmt> = DisymGraph::default();
  g.add_symbol(Stmt::new("alice", Effect::Allow, &[]));
  g.add_symbol(Stmt::new("AllowRead", Effect::Allow, &["s3:GetObject"]));
  g.add_symbol(Stmt::new("DenyWrite", Effect::Deny, &["s3:PutObject"]));
  g.add_symbol(Stmt::new("bucket", Effect::Allow, &[]));
  g.connect("alice", &["AllowRead", "DenyWrite"]).unwrap();
  g.connect("AllowRead", &["bucket"]).unwrap();
  g.connect("DenyWrite", &["bucket"]).unwrap();
  g
}

#[test]
fn payload_default_and_add_symbol_dedupes_by_id() {
  let mut g: DisymGraph<Stmt> = DisymGraph::default();
  assert_eq!(g.vert_count(), 0);
  assert_eq!(g.edge_count(), 0);

  let a = g.add_symbol(Stmt::new("a", Effect::Allow, &["x"]));
  let b = g.add_symbol(Stmt::new("b", Effect::Deny, &[]));
  // Same id, different payload: existing index returned, payload kept.
  let a2 = g.add_symbol(Stmt::new("a", Effect::Deny, &["y"]));

  assert_eq!((a, b, a2), (0, 1, 0));
  assert_eq!(g.vert_count(), 2);
  assert_eq!(g.symbol(0).unwrap().effect, Effect::Allow);
  assert_eq!(g.symbol(0).unwrap().actions, vec!["x".to_string()]);
  assert!(g.contains("a") && g.has_vertex("b") && !g.contains("c"));
  assert!(g.validate_vertex("a").is_ok());
  assert_eq!(
    g.validate_vertex("zzz").unwrap_err(),
    invalid_vert_symbol_msg("zzz")
  );
}

#[test]
fn payload_symbol_and_name_lookups() {
  let g = policy_graph();

  assert_eq!(g.index("AllowRead"), Some(1));
  assert_eq!(g.index("nope"), None);
  assert_eq!(g.indices(&["bucket", "nope", "alice"]), Some(vec![3, 0]));
  assert_eq!(g.indices(&[]), None);

  assert_eq!(g.name(2), Some("DenyWrite".to_string()));
  assert_eq!(g.name_as_ref(2), Some("DenyWrite"));
  assert_eq!(g.name(99), None);
  assert_eq!(
    g.names(&[0, 3]),
    Some(vec!["alice".to_string(), "bucket".to_string()])
  );

  assert_eq!(
    g.symbol(1).unwrap().actions,
    vec!["s3:GetObject".to_string()]
  );
  assert!(g.symbol(99).is_none());
  let syms = g.symbols(&[2, 99, 0]);
  assert_eq!(syms.len(), 2);
  assert_eq!(syms[0].effect, Effect::Deny);
  assert_eq!(syms[1].id(), "alice");
}

#[test]
fn payload_connect_and_adjacency() {
  let g = policy_graph();

  assert_eq!(g.edge_count(), 4);
  assert_eq!(g.adj("alice"), Some(vec!["AllowRead", "DenyWrite"]));
  assert_eq!(g.adj_indices("alice"), Some(&vec![1, 2]));
  assert_eq!(g.outdegree(0), Ok(2));
  assert_eq!(g.indegree(3), Ok(2));

  let stmts = g.adj_symbols("alice").unwrap();
  assert_eq!(stmts.len(), 2);
  assert_eq!(stmts[0].effect, Effect::Allow);
  assert_eq!(stmts[1].effect, Effect::Deny);
  assert_eq!(stmts[1].actions, vec!["s3:PutObject".to_string()]);

  assert_eq!(g.adj_symbols("bucket"), Some(vec![]));
  assert_eq!(g.adj_symbols("missing"), None);
  assert_eq!(g.adj("missing"), None);
}

#[test]
fn payload_connect_unknown_endpoint_errors_and_is_atomic() {
  let mut g = policy_graph();
  let edges_before = g.edge_count();

  assert_eq!(
    g.connect("ghost", &["bucket"]).unwrap_err(),
    invalid_vert_symbol_msg("ghost")
  );
  assert_eq!(
    g.connect("alice", &["bucket", "ghost"]).unwrap_err(),
    invalid_vert_symbol_msg("ghost")
  );
  // Nothing was added, not even the valid "alice -> bucket" edge.
  assert_eq!(g.edge_count(), edges_before);
  assert_eq!(g.adj("alice"), Some(vec!["AllowRead", "DenyWrite"]));
}

#[test]
fn payload_directed_cycle_detection() {
  let mut g = policy_graph();
  assert!(!DirectedCycle::new(g.graph()).has_cycle());

  g.connect("bucket", &["alice"]).unwrap();
  let cycle = DirectedCycle::new(g.graph());
  assert!(cycle.has_cycle());
  let ids: Vec<&str> = cycle
    .cycle()
    .unwrap()
    .iter()
    .map(|i| g.name_as_ref(*i).unwrap())
    .collect();
  assert!(ids.contains(&"alice") && ids.contains(&"bucket"));
}

#[test]
fn payload_topological_order() {
  let g = policy_graph();
  let topo = Topology::new(g.graph());
  assert!(topo.is_dag());

  let order: Vec<&str> = topo
    .order()
    .unwrap()
    .iter()
    .map(|i| g.name_as_ref(*i).unwrap())
    .collect();
  let pos = |id: &str| order.iter().position(|x| *x == id).unwrap();
  assert!(pos("alice") < pos("AllowRead"));
  assert!(pos("alice") < pos("DenyWrite"));
  assert!(pos("AllowRead") < pos("bucket"));
  assert!(pos("DenyWrite") < pos("bucket"));
}

#[test]
fn payload_dfs_reachability() {
  let g = policy_graph();
  let dfs = DirectedPathsDFS::new(g.graph(), g.index("AllowRead").unwrap()).unwrap();

  assert_eq!(dfs.marked(g.index("bucket").unwrap()), Ok(true));
  assert_eq!(dfs.marked(g.index("DenyWrite").unwrap()), Ok(false));
  assert_eq!(dfs.marked(g.index("alice").unwrap()), Ok(false));

  let reachable: Vec<&Stmt> = (0..g.vert_count())
    .filter(|i| dfs.marked(*i).unwrap())
    .filter_map(|i| g.symbol(i))
    .collect();
  assert_eq!(reachable.len(), 2);
}

#[test]
fn payload_reverse_keeps_payloads() {
  let g = policy_graph();
  let r = g.reverse().unwrap();

  assert_eq!(r.vert_count(), g.vert_count());
  assert_eq!(r.edge_count(), g.edge_count());
  assert_eq!(r.adj("bucket"), Some(vec!["AllowRead", "DenyWrite"]));
  assert_eq!(r.adj("alice"), Some(vec![]));
  assert_eq!(r.symbol(1), g.symbol(1));
  assert_eq!(r.adj_symbols("bucket").unwrap()[1].effect, Effect::Deny);
}

#[test]
fn payload_clone_is_independent() {
  let g = policy_graph();
  let mut c = g.clone();
  c.add_symbol(Stmt::new("extra", Effect::Allow, &[]));
  assert_eq!(g.vert_count(), 4);
  assert_eq!(c.vert_count(), 5);
}

#[test]
fn payload_data_round_trip() {
  let g = policy_graph();
  let data = DisymGraphData::<Stmt>::try_from(&g).unwrap();

  assert_eq!(data.len(), 4);
  assert_eq!(data[0].0, Stmt::new("alice", Effect::Allow, &[]));
  assert_eq!(
    data[0].1,
    Some(vec!["AllowRead".to_string(), "DenyWrite".to_string()])
  );
  assert_eq!(data[3].0.id(), "bucket");
  assert_eq!(data[3].1, None);

  let g2 = DisymGraph::<Stmt>::try_from(&data).unwrap();
  assert_eq!(g2.vert_count(), g.vert_count());
  assert_eq!(g2.edge_count(), g.edge_count());
  for i in 0..g.vert_count() {
    assert_eq!(g2.symbol(i), g.symbol(i));
    assert_eq!(
      g2.adj(g.name_as_ref(i).unwrap()),
      g.adj(g.name_as_ref(i).unwrap())
    );
  }

  // Owned variants
  let g3 = DisymGraph::try_from(data).unwrap();
  let data2: DisymGraphData<Stmt> = DisymGraphData::try_from(g3).unwrap();
  assert_eq!(data2.len(), 4);
}

#[test]
fn data_edges_may_reference_later_entries() {
  let data: DisymGraphData<Stmt> = vec![
    (
      Stmt::new("a", Effect::Allow, &[]),
      Some(vec!["b".to_string()]),
    ),
    (Stmt::new("b", Effect::Deny, &[]), None),
  ];
  let g = DisymGraph::try_from(&data).unwrap();
  assert_eq!(g.adj("a"), Some(vec!["b"]));
}

#[test]
fn data_unknown_edge_target_errors_for_payload() {
  let data: DisymGraphData<Stmt> = vec![(
    Stmt::new("a", Effect::Allow, &[]),
    Some(vec!["missing".to_string()]),
  )];
  assert_eq!(
    DisymGraph::try_from(&data).unwrap_err(),
    invalid_vert_symbol_msg("missing")
  );
}

#[test]
fn data_unknown_edge_target_errors_for_string() {
  let data: DisymGraphData = vec![("user".to_string(), Some(vec!["guest".to_string()]))];
  assert_eq!(
    DisymGraph::try_from(&data).unwrap_err(),
    invalid_vert_symbol_msg("guest")
  );
}

#[test]
fn from_string_payload_add_edge_auto_creates() {
  let mut g: DisymGraph<Named> = DisymGraph::default();
  g.add_edge("admin", &["user", "moderator"]).unwrap();
  g.add_edge("user", &["guest"]).unwrap();
  assert_eq!(g.add_vertex("guest"), 3);

  assert_eq!(g.vert_count(), 4);
  assert_eq!(g.adj("admin"), Some(vec!["user", "moderator"]));
  assert_eq!(g.symbol(0), Some(&Named::from("admin".to_string())));
  assert_eq!(g.adj_symbols("user").unwrap()[0].visits, 0);
}

#[test]
fn from_string_payload_loads_from_reader() {
  let input = "JFK MCO ORD\nORD DEN\nDEN\n";
  let g: DisymGraph<Named> = DisymGraph::try_from(BufReader::new(input.as_bytes())).unwrap();
  assert_eq!(g.vert_count(), 4);
  assert_eq!(g.adj("JFK"), Some(vec!["MCO", "ORD"]));
  assert_eq!(g.symbol(g.index("DEN").unwrap()).unwrap().name, "DEN");

  let mut reader = BufReader::new(input.as_bytes());
  let g2: DisymGraph<Named> = DisymGraph::try_from(&mut reader).unwrap();
  assert_eq!(g2.edge_count(), 3);
}

#[test]
fn string_default_interop() {
  let mut a = DisymGraph::new();
  let mut b: DisymGraph<String> = DisymGraph::default();
  a.add_edge("x", &["y"]).unwrap();
  b.add_symbol("x".to_string());
  b.add_symbol("y".to_string());
  b.connect("x", &["y"]).unwrap();

  assert_eq!(a.adj("x"), b.adj("x"));
  assert_eq!(a.adj_symbols("x"), Some(vec![&"y".to_string()]));
  assert_eq!(a.symbol(0), Some(&"x".to_string()));
  assert_eq!(
    DisymGraphData::try_from(&a).unwrap(),
    DisymGraphData::try_from(&b).unwrap()
  );
}

mod serde_round_trip {
  use super::{DisymGraph, DisymGraphData, Symbol};
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  struct Stmt {
    sid: String,
    effect: String,
    actions: Vec<String>,
  }

  impl Symbol for Stmt {
    fn id(&self) -> &str {
      &self.sid
    }
  }

  #[test]
  fn payload_serde_json_round_trip() {
    let mut g: DisymGraph<Stmt> = DisymGraph::default();
    g.add_symbol(Stmt {
      sid: "AllowRead".into(),
      effect: "Allow".into(),
      actions: vec!["s3:GetObject".into()],
    });
    g.add_symbol(Stmt {
      sid: "bucket".into(),
      effect: String::new(),
      actions: vec![],
    });
    g.connect("AllowRead", &["bucket"]).unwrap();

    let data = DisymGraphData::<Stmt>::try_from(&g).unwrap();
    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"sid\":\"AllowRead\""));
    assert!(json.contains("[\"bucket\"]"));

    let parsed: DisymGraphData<Stmt> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, data);

    let g2 = DisymGraph::try_from(&parsed).unwrap();
    assert_eq!(g2.vert_count(), 2);
    assert_eq!(g2.adj("AllowRead"), Some(vec!["bucket"]));
    assert_eq!(
      g2.symbol(0).unwrap().actions,
      vec!["s3:GetObject".to_string()]
    );
    assert_eq!(g2.symbol(0), g.symbol(0));
  }

  #[test]
  fn string_default_serde_json_shape_unchanged() {
    let mut g = DisymGraph::new();
    g.add_edge("admin", &["user"]).unwrap();
    let data = DisymGraphData::try_from(&g).unwrap();
    assert_eq!(
      serde_json::to_string(&data).unwrap(),
      r#"[["admin",["user"]],["user",null]]"#
    );
    let parsed: DisymGraphData =
      serde_json::from_str(r#"[["admin",["user"]],["user",null]]"#).unwrap();
    assert_eq!(parsed, data);
  }
}

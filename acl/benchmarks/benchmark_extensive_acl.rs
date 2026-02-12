/// Benchmark program for the extensive ACL configuration.
///
/// This program loads a large ACL with 46 roles, 79 resources, and 300+ rules,
/// then performs random permission checks to measure performance and memory usage.
///
/// Run with: `cargo run --release --example benchmark_extensive_acl`

use std::fs::File;
use std::time::Instant;
use std::convert::TryFrom;
use walrs_acl::simple::{Acl, AclBuilder, AclData};
use rand::seq::SliceRandom;

fn main() -> Result<(), String> {
    println!("=== ACL Performance Benchmark ===\n");

    // Load the extensive ACL from JSON file
    println!("Loading extensive ACL from JSON...");
    let start = Instant::now();

    let file = File::open("test-fixtures/example-extensive-acl-array.json")
        .map_err(|e| format!("Failed to open ACL file: {}", e))?;

    let acl_data: AclData = serde_json::from_reader(file)
        .map_err(|e| format!("Failed to parse ACL JSON: {}", e))?;

    // Estimate AclData size
    let acl_data_size = estimate_acl_data_size(&acl_data);

    let acl = AclBuilder::try_from(&acl_data)?.build()?;

    let load_duration = start.elapsed();
    println!("✓ ACL loaded in {:?}", load_duration);
    println!("  - Roles: {}", acl.role_count());
    println!("  - Resources: {}", acl.resource_count());
    println!("  - Estimated AclData size: ~{} KB", acl_data_size / 1024);
    println!("  - Estimated ACL structure size: ~{} KB", estimate_acl_size(&acl) / 1024);
    println!();

    // Extract all roles and resources for random testing
    let roles = extract_roles(&acl_data);
    let resources = extract_resources(&acl_data);
    let privileges = extract_privileges(&acl_data);

    println!("Extracted test data:");
    println!("  - {} unique roles", roles.len());
    println!("  - {} unique resources", resources.len());
    println!("  - {} unique privileges", privileges.len());
    println!();

    // Run benchmarks
    run_benchmark("Single permission check", &acl, &roles, &resources, &privileges, 1);
    run_benchmark("10 permission checks", &acl, &roles, &resources, &privileges, 10);
    run_benchmark("100 permission checks", &acl, &roles, &resources, &privileges, 100);
    run_benchmark("1,000 permission checks", &acl, &roles, &resources, &privileges, 1_000);
    run_benchmark("10,000 permission checks", &acl, &roles, &resources, &privileges, 10_000);
    run_benchmark("100,000 permission checks", &acl, &roles, &resources, &privileges, 100_000);

    println!();
    println!("=== Specific Scenario Benchmarks ===\n");

    // Test specific scenarios
    run_inheritance_benchmark(&acl, &roles, &resources, &privileges);
    run_role_hierarchy_benchmark(&acl);
    run_resource_hierarchy_benchmark(&acl);
    run_deny_rule_benchmark(&acl);

    println!();
    println!("=== Benchmark Complete ===");

    Ok(())
}

fn run_benchmark(
    name: &str,
    acl: &Acl,
    roles: &[String],
    resources: &[String],
    privileges: &[String],
    iterations: usize,
) {
    let mut rng = rand::thread_rng();

    let start = Instant::now();
    let mut allowed_count = 0;
    let mut denied_count = 0;

    for _ in 0..iterations {
        let role = roles.choose(&mut rng).unwrap();
        let resource = resources.choose(&mut rng).unwrap();
        let privilege = privileges.choose(&mut rng).unwrap();

        if acl.is_allowed(Some(role.as_str()), Some(resource.as_str()), Some(privilege.as_str())) {
            allowed_count += 1;
        } else {
            denied_count += 1;
        }
    }

    let duration = start.elapsed();
    let avg_time = duration / iterations as u32;
    let checks_per_sec = iterations as f64 / duration.as_secs_f64();

    println!("{}", name);
    println!("  Total time: {:?}", duration);
    println!("  Average per check: {:?}", avg_time);
    println!("  Checks per second: {:.0}", checks_per_sec);
    println!("  Results: {} allowed, {} denied", allowed_count, denied_count);
    println!();
}

fn run_inheritance_benchmark(
    acl: &Acl,
    _roles: &[String],
    resources: &[String],
    privileges: &[String],
) {
    println!("Role inheritance checks (1,000 iterations):");

    let mut rng = rand::thread_rng();
    let start = Instant::now();

    // Test with roles that have deep inheritance (e.g., super_admin)
    let deep_roles = vec!["super_admin", "administrator", "cfo", "engineering_manager"];

    for _ in 0..1_000 {
        let role = deep_roles.choose(&mut rng).unwrap();
        let resource = resources.choose(&mut rng).unwrap();
        let privilege = privileges.choose(&mut rng).unwrap();

        let _ = acl.is_allowed(Some(role), Some(resource.as_str()), Some(privilege.as_str()));
    }

    let duration = start.elapsed();
    println!("  Time: {:?}", duration);
    println!("  Avg per check: {:?}", duration / 1_000);
    println!();
}

fn run_role_hierarchy_benchmark(acl: &Acl) {
    println!("Role hierarchy inheritance checks:");

    // Test the deepest role hierarchy
    let hierarchy = vec![
        "guest",
        "authenticated",
        "subscriber",
        "contributor",
        "author",
        "editor",
        "moderator",
        "administrator",
        "super_admin",
    ];

    let start = Instant::now();

    // Check if each role inherits from all its ancestors
    for (i, role) in hierarchy.iter().enumerate() {
        for ancestor in &hierarchy[..i] {
            acl.inherits_role(role, ancestor);
        }
    }

    let duration = start.elapsed();
    println!("  Checked {} inheritance relationships", hierarchy.len() * (hierarchy.len() - 1) / 2);
    println!("  Time: {:?}", duration);
    println!();
}

fn run_resource_hierarchy_benchmark(acl: &Acl) {
    println!("Resource hierarchy inheritance checks:");

    // Test resource hierarchies
    let resource_chains = vec![
        vec!["public_pages", "blog", "blog_post", "blog_comment"],
        vec!["public_pages", "forum", "forum_thread", "forum_post"],
        vec!["public_pages", "wiki", "wiki_page"],
        vec!["user_profile", "user_settings", "user_private_data"],
        vec!["admin_panel", "admin_users", "admin_settings", "admin_system"],
        vec!["api", "api_public", "api_private"],
        vec!["reports", "report_analytics", "report_financial"],
        vec!["development", "dev_repository", "dev_deployment"],
        vec!["finance", "finance_payroll", "finance_budget"],
    ];

    let start = Instant::now();
    let mut check_count = 0;

    for chain in &resource_chains {
        for (i, resource) in chain.iter().enumerate() {
            for ancestor in &chain[..i] {
                acl.inherits_resource(resource, ancestor);
                check_count += 1;
            }
        }
    }

    let duration = start.elapsed();
    println!("  Checked {} inheritance relationships", check_count);
    println!("  Time: {:?}", duration);
    println!();
}

fn run_deny_rule_benchmark(acl: &Acl) {
    println!("Deny rule evaluation (1,000 checks):");

    // Test resources with explicit deny rules
    let deny_scenarios = vec![
        ("editor", "admin_panel", "read"),
        ("moderator", "admin_panel", "edit"),
        ("moderator", "user_private_data", "read"),
        ("contributor", "finance", "read"),
        ("author", "finance", "write"),
        ("editor", "finance", "read"),
        ("administrator", "admin_system", "delete"),
        ("analyst", "finance_payroll", "read"),
        ("developer", "dev_deployment", "deploy_production"),
        ("support_tier1", "support_ticket", "delete"),
    ];

    let start = Instant::now();

    for _ in 0..1_000 {
        for (role, resource, privilege) in &deny_scenarios {
            let _ = acl.is_allowed(Some(role), Some(resource), Some(privilege));
        }
    }

    let duration = start.elapsed();
    let total_checks = 1_000 * deny_scenarios.len();
    println!("  Total checks: {}", total_checks);
    println!("  Time: {:?}", duration);
    println!("  Avg per check: {:?}", duration / total_checks as u32);
    println!();
}

fn extract_roles(acl_data: &AclData) -> Vec<String> {
    let mut roles = Vec::new();

    if let Some(roles_list) = &acl_data.roles {
        for (role, _parents) in roles_list {
            roles.push(role.clone());
        }
    }

    roles.sort();
    roles
}

fn extract_resources(acl_data: &AclData) -> Vec<String> {
    let mut resources = Vec::new();

    if let Some(resources_list) = &acl_data.resources {
        for (resource, _parents) in resources_list {
            resources.push(resource.clone());
        }
    }

    resources.sort();
    resources
}

fn extract_privileges(acl_data: &AclData) -> Vec<String> {
    let mut privileges = std::collections::HashSet::new();

    // Extract from allow rules
    if let Some(allow_rules) = &acl_data.allow {
        for (_resource, role_privilege_list) in allow_rules.iter() {
            if let Some(list) = role_privilege_list {
                for (_role, privs) in list.iter() {
                    if let Some(priv_list) = privs {
                        for privilege in priv_list {
                            privileges.insert(privilege.clone());
                        }
                    }
                }
            }
        }
    }

    // Extract from deny rules
    if let Some(deny_rules) = &acl_data.deny {
        for (_resource, role_privilege_list) in deny_rules.iter() {
            if let Some(list) = role_privilege_list {
                for (_role, privs) in list.iter() {
                    if let Some(priv_list) = privs {
                        for privilege in priv_list {
                            privileges.insert(privilege.clone());
                        }
                    }
                }
            }
        }
    }

    // Add some common privileges if none found
    if privileges.is_empty() {
        privileges.insert("read".to_string());
        privileges.insert("write".to_string());
        privileges.insert("edit".to_string());
        privileges.insert("delete".to_string());
    }

    let mut result: Vec<String> = privileges.into_iter().collect();
    result.sort();
    result
}

/// Estimates the memory size of AclData structure
fn estimate_acl_data_size(acl_data: &AclData) -> usize {
    let mut size = std::mem::size_of::<AclData>();

    // Estimate roles size
    if let Some(roles) = &acl_data.roles {
        size += std::mem::size_of::<Vec<(String, Option<Vec<String>>)>>();
        for (role_name, parents) in roles {
            size += std::mem::size_of::<String>() + role_name.len();
            if let Some(parent_list) = parents {
                size += std::mem::size_of::<Vec<String>>();
                for parent in parent_list {
                    size += std::mem::size_of::<String>() + parent.len();
                }
            }
        }
    }

    // Estimate resources size
    if let Some(resources) = &acl_data.resources {
        size += std::mem::size_of::<Vec<(String, Option<Vec<String>>)>>();
        for (resource_name, parents) in resources {
            size += std::mem::size_of::<String>() + resource_name.len();
            if let Some(parent_list) = parents {
                size += std::mem::size_of::<Vec<String>>();
                for parent in parent_list {
                    size += std::mem::size_of::<String>() + parent.len();
                }
            }
        }
    }

    // Estimate allow rules size
    if let Some(allow_rules) = &acl_data.allow {
        size += estimate_rules_size(allow_rules);
    }

    // Estimate deny rules size
    if let Some(deny_rules) = &acl_data.deny {
        size += estimate_rules_size(deny_rules);
    }

    size
}

/// Estimates the memory size of rules (allow or deny)
fn estimate_rules_size(rules: &Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>) -> usize {
    let mut size = std::mem::size_of::<Vec<(String, Option<Vec<(String, Option<Vec<String>>)>>)>>();

    for (resource, role_privilege_list) in rules {
        size += std::mem::size_of::<String>() + resource.len();

        if let Some(list) = role_privilege_list {
            size += std::mem::size_of::<Vec<(String, Option<Vec<String>>)>>();
            for (role, privileges) in list {
                size += std::mem::size_of::<String>() + role.len();
                if let Some(priv_list) = privileges {
                    size += std::mem::size_of::<Vec<String>>();
                    for privilege in priv_list {
                        size += std::mem::size_of::<String>() + privilege.len();
                    }
                }
            }
        }
    }

    size
}

/// Estimates the memory size of the compiled Acl structure
fn estimate_acl_size(acl: &Acl) -> usize {
    let mut size = std::mem::size_of::<Acl>();

    // Estimate role graph size (vertices + edges)
    // Each role is stored as a vertex with potential edges to parent roles
    let role_count = acl.role_count();
    size += role_count * 64; // Rough estimate per role (name + graph node)

    // Estimate resource graph size
    let resource_count = acl.resource_count();
    size += resource_count * 64; // Rough estimate per resource

    // Estimate rules structure size (nested HashMaps)
    // This is a rough estimate based on typical HashMap overhead + keys + values
    let estimated_rules_size = (role_count + resource_count) * 128;
    size += estimated_rules_size;

    size
}

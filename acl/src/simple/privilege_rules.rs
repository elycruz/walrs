use std::collections::HashMap;

use crate::simple::rule::{Rule, RuleContextScope};
use crate::simple::types::Privilege;

#[derive(Debug, PartialEq, Clone)]
pub struct PrivilegeRules {
  pub for_all_privileges: Rule,
  pub by_privilege_id: Option<HashMap<Privilege, Rule>>,
}

impl PrivilegeRules {
  pub fn new(create_privilege_map: bool) -> Self {
    PrivilegeRules {
      for_all_privileges: Rule::Deny,
      by_privilege_id: if create_privilege_map {
        Some(HashMap::new())
      } else {
        None
      },
    }
  }

  /// Returns set rule for privilege id.
  pub fn get_rule(&self, privilege_id: Option<&str>) -> &Rule {
    privilege_id
      .zip(self.by_privilege_id.as_ref())
      .and_then(|(privilege_id, privilege_map)| privilege_map.get(privilege_id))
      .unwrap_or(&self.for_all_privileges)
  }

  pub fn set_rule(&mut self, privilege_ids: Option<&[&str]>, rule: Rule) -> RuleContextScope {
    if let Some(ps) = privilege_ids {
      if !ps.is_empty() {
        ps.iter().for_each(|p| {
          self
            .by_privilege_id
            .get_or_insert(HashMap::new())
            .insert(p.to_string(), rule);
        });
      } else {
        self.for_all_privileges = rule;
        return RuleContextScope::ForAllSymbols;
      }
      RuleContextScope::PerSymbol
    } else {
      self.for_all_privileges = rule;
      RuleContextScope::ForAllSymbols
    }
  }
}

#[cfg(test)]
mod test_privilege_rules {
  use super::PrivilegeRules;
  use crate::simple::rule::Rule;

  fn test_default_state(prs: &PrivilegeRules, with_created_maps: bool) {
    // Tests default generation/non-generation of internal hashmaps
    assert_eq!(
      prs.by_privilege_id.is_some(),
      with_created_maps,
      "Expected `prs.by_privilege_id.is_some()` to equal `{}`",
      with_created_maps
    );

    // Test default rule `for_all_roles`
    assert_eq!(
      prs.for_all_privileges,
      Rule::Deny,
      "Expected `prs.for_all_privileges` to equal `Rule::Deny`"
    );
  }

  #[test]
  fn test_new() {
    for with_created_maps in [false, true] {
      let prs = PrivilegeRules::new(with_created_maps.into());
      test_default_state(&prs, with_created_maps);
    }
  }

  #[test]
  fn test_get_rule() {
    // Test empty, "default", PrivilegeRules results
    // ----
    for with_created_maps in [false, true] {
      let prs = PrivilegeRules::new(with_created_maps.into());
      test_default_state(&prs, with_created_maps);
    }

    // Test populated `PrivilegeRules` instances
    // ----
    let account_index_privilege = "account-index";
    let index_privilege = "index";

    let mut privilege_rules = PrivilegeRules::new(true);

    for (privilege, expected_rule) in [
      (index_privilege, Rule::Allow),
      (account_index_privilege, Rule::Deny),
    ] {
      // Set privilege rules
      privilege_rules
        .by_privilege_id
        .as_mut()
        .and_then(|privilege_id_map| {
          privilege_id_map.insert(privilege.to_string(), expected_rule);
          Some(())
        })
        .expect("Expecting a `privilege_id_map`;  None found");

      // Test for expected (1)
      assert_eq!(
        &privilege_rules.get_rule(Some(privilege)),
        privilege_rules
          .by_privilege_id
          .as_ref()
          .unwrap()
          .get(privilege)
          .as_ref()
          .unwrap(),
        "Expected returned `RuleType` to equal {:?}",
        expected_rule
      );

      assert_eq!(
        privilege_rules.get_rule(Some(privilege)),
        &expected_rule,
        "Expected returned `RuleType` to equal `{:#?}`, for \"{:?}\"",
        expected_rule,
        privilege
      );
    }
  }

  #[test]
  fn test_set_rule() {
    let account_index_privilege = "account-index";
    let index_privilege = "index";
    let create = "create";
    let read = "read";
    let update = "update";
    let delete = "delete";
    for (create_internal_map, privileges_ids, expected_rule) in [
      (false, vec![index_privilege], Rule::Allow),
      (false, vec![account_index_privilege], Rule::Deny),
      (true, vec![index_privilege], Rule::Allow),
      (true, vec![account_index_privilege], Rule::Deny),
      (true, vec![create, read, update, delete], Rule::Deny),
    ] {
      let mut prs = PrivilegeRules::new(create_internal_map.into());
      test_default_state(&prs, create_internal_map);

      prs.set_rule(Some(&privileges_ids), expected_rule);

      // Test for expected (1)
      privileges_ids.iter().for_each(|pid| {
        assert_eq!(
          prs.get_rule(Some(pid)),
          prs.by_privilege_id.as_ref().unwrap().get(*pid).unwrap(),
          "Expected returned `RuleType` to equal {:?}",
          expected_rule
        );

        assert_eq!(
          prs.get_rule(Some(pid)),
          &expected_rule,
          "Expected returned `RuleType` to equal `{:#?}`, for \"{:?}\"",
          expected_rule,
          privileges_ids
        );
      });
    }

    // Test scenario where Priv*Rules contains allowed, and denied, rules
    // ----
    let mut prs = PrivilegeRules::new(true);
    let mut prs_2 = PrivilegeRules::new(false);
    let denied_privileges = vec!["create", "read", "update", "delete"];
    let allowed_privileges = vec!["index"];

    // Set rules for rule set with "initiated" internal map
    prs.set_rule(Some(&denied_privileges), Rule::Deny);
    prs.set_rule(Some(&allowed_privileges), Rule::Allow);

    // Set rules on rule set with "uninitiated" internal map
    prs_2.set_rule(Some(&denied_privileges), Rule::Deny);
    prs_2.set_rule(Some(&allowed_privileges), Rule::Allow);

    // Test results for each rule set
    for (privilege_ids, rule) in [
      (&denied_privileges, Rule::Deny),
      (&allowed_privileges, Rule::Allow),
    ] {
      privilege_ids.iter().for_each(|pid| {
        assert_eq!(prs.get_rule(Some(pid)), &rule, "Mismatching `Rule`");
        assert_eq!(prs_2.get_rule(Some(pid)), &rule, "Mismatching `Rule`");
      });
    }
  }
}

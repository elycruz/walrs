#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

use crate::prelude::ToString;
use crate::simple::{PrivilegeRules, Role, RuleContextScope};

#[derive(Debug, PartialEq, Clone)]
pub struct RolePrivilegeRules {
    pub for_all_roles: PrivilegeRules,
    pub by_role_id: Option<HashMap<Role, PrivilegeRules>>,
}

impl RolePrivilegeRules {
    pub fn new(create_child_maps: bool) -> Self {
        RolePrivilegeRules {
            for_all_roles: PrivilegeRules::new(create_child_maps),
            by_role_id: if create_child_maps {
                Some(HashMap::new())
            } else {
                None
            },
        }
    }

    pub fn get_privilege_rules(&self, role: Option<&str>) -> &PrivilegeRules {
        role
            .zip(self.by_role_id.as_ref())
            .and_then(|(role, role_map)| role_map.get(role))
            .unwrap_or(&self.for_all_roles)
    }

    pub fn get_privilege_rules_mut(&mut self, role: Option<&str>) -> &mut PrivilegeRules {
        role
            .zip(self.by_role_id.as_mut())
            .and_then(|(role, role_map)| role_map.get_mut(role))
            .unwrap_or(&mut self.for_all_roles)
    }

    pub fn set_privilege_rules_for_role_ids(
        &mut self,
        role_ids: &[&str],
        privilege_rules: PrivilegeRules,
    ) -> RuleContextScope {
        if role_ids.is_empty() {
            self.for_all_roles = privilege_rules;
            RuleContextScope::ForAllSymbols
        } else {
            role_ids.iter().for_each(|role_id| {
                self
                    .by_role_id
                    .get_or_insert(HashMap::new())
                    .insert(role_id.to_string(), privilege_rules.clone());
            });
            RuleContextScope::PerSymbol
        }
    }

    pub fn set_privilege_rules(
        &mut self,
        role_ids: Option<&[&str]>,
        privilege_rules: Option<PrivilegeRules>,
    ) -> RuleContextScope {
        if role_ids.is_some() && privilege_rules.is_some() {
            privilege_rules
                .zip(role_ids)
                .map(|(privilege_rules, role_ids)| {
                    self.set_privilege_rules_for_role_ids(role_ids, privilege_rules)
                })
                .unwrap()
        } else if privilege_rules.is_some() && role_ids.is_none() {
            self.for_all_roles = privilege_rules.unwrap();
            RuleContextScope::ForAllSymbols
        } else if privilege_rules.is_none() && role_ids.is_some() {
            self.set_privilege_rules_for_role_ids(role_ids.unwrap(), PrivilegeRules::new(false))
        } else {
            self.for_all_roles = PrivilegeRules::new(false);
            RuleContextScope::ForAllSymbols
        }
    }
}

#[cfg(test)]
mod test_role_privilege_rules {
    #[cfg(feature = "std")]
    use std::collections::HashMap;
    #[cfg(not(feature = "std"))]
    use alloc::collections::BTreeMap as HashMap;

    use crate::simple::Rule;
    use super::{PrivilegeRules, RolePrivilegeRules};

    fn test_constructed_defaults(rprs: &RolePrivilegeRules, with_child_maps: bool) {
        assert_eq!(
            rprs.by_role_id.is_some(),
            with_child_maps,
            "Expected `rprs.by_role_id.is_some()` to equal `{}`",
            with_child_maps
        );
    }

    // Tests setter, and getter results
    fn test_when_roles_and_privileges(
        r_ids: &[&str],
        p_ids: &[&str],
        rpr: &RolePrivilegeRules,
        expected_rule: &Rule,
    ) {
        p_ids.iter().for_each(|p_id| {
            r_ids.iter().for_each(|r_id| {
                let found_privilege_rules = rpr.by_role_id.as_ref().unwrap().get(*r_id).unwrap();
                let found_rule = found_privilege_rules
                    .by_privilege_id
                    .as_ref()
                    .unwrap()
                    .get(*p_id)
                    .unwrap();
                assert_eq!(
                    found_rule, expected_rule,
                    "Found rule is not equal to expected"
                );
                assert_eq!(
                    rpr.get_privilege_rules(Some(r_id)),
                    found_privilege_rules,
                    "`#RolePrivilegeRules.get_privilege_rule(Some({:?})) != {:?}`",
                    r_id,
                    found_privilege_rules
                );
            });
        });
    }

    // Tests setter, and getter results
    fn test_when_only_roles(r_ids: &[&str], rpr: &RolePrivilegeRules, expected_rule: &Rule) {
        r_ids.iter().for_each(|r_id| {
            let found_privilege_rules = rpr.by_role_id.as_ref().unwrap().get(*r_id).unwrap();
            let found_rule = &found_privilege_rules.for_all_privileges;
            assert_eq!(
                found_rule, expected_rule,
                "Found rule is not equal to 'expected' rule"
            );
            assert_eq!(
                rpr.get_privilege_rules(Some(r_id)),
                found_privilege_rules,
                "`#RolePrivilegeRules.get_privilege_rule(Some({:?})) != {:?}`",
                r_id,
                found_privilege_rules
            );
        });
    }

    // Tests setter, and getter results
    fn test_when_only_privileges(p_ids: &[&str], rpr: &RolePrivilegeRules, expected_rule: &Rule) {
        p_ids.iter().for_each(|p_id| {
            assert_eq!(
                rpr
                    .for_all_roles
                    .by_privilege_id
                    .as_ref()
                    .unwrap()
                    .get(*p_id)
                    .unwrap(),
                expected_rule
            );
        });
        assert_eq!(
            rpr.get_privilege_rules(None),
            &rpr.for_all_roles,
            "`#RolePrivilegeRules.get_privilege_rule(None) != &rpr.for_all_roles`",
        );
    }

    // Tests setter and getter results
    fn test_when_no_roles_no_privileges(rpr: &RolePrivilegeRules, expected_rule: &Rule) {
        assert_eq!(&rpr.for_all_roles.for_all_privileges, expected_rule);
        assert_eq!(rpr.get_privilege_rules(None), &rpr.for_all_roles);
    }

    #[test]
    fn test_new() {
        for create_child_maps in [false, true] {
            let rprs = RolePrivilegeRules::new(create_child_maps.into());
            test_constructed_defaults(&rprs, create_child_maps);
        }
    }

    #[test]
    fn test_get_and_set_privilege_rules() {
        let role_privileges = RolePrivilegeRules::new(true);
        assert_eq!(
            role_privileges.get_privilege_rules(None),
            &role_privileges.for_all_roles,
            "Expecting returned value to equal privilege rules \"for all roles\""
        );

        assert_eq!(
            role_privileges.get_privilege_rules(Some("hello")),
            &role_privileges.for_all_roles,
            "Expecting returned value to equal privilege rules \"for all roles\""
        );

        // Role, and privilege, Ids
        let admin_role = "admin";
        let user_role = "user";
        let guest_role = "guest";
        let user_privilege = "create";
        let guest_privilege = "index";
        let admin_privilege = "delete";

        // Privilege lists
        let guest_privileges = vec![guest_privilege];
        let user_privileges = vec![user_privilege, guest_privilege];
        let admin_privileges = vec![admin_privilege, user_privilege, guest_privilege];

        // Role lists
        let guest_roles = vec![guest_role];
        let user_roles = vec![user_role];
        let admin_roles = vec![admin_role];

        // Run tests
        for (role_ids, privilege_ids, expected_rule) in [
            (None, None, Rule::Deny),
            (Some(vec![].as_slice()), Some(vec![].as_slice()), Rule::Deny),
            (None, None, Rule::Allow),
            (
                Some(vec![].as_slice()),
                Some(vec![].as_slice()),
                Rule::Allow,
            ),
            (Some(guest_roles.as_slice()), None, Rule::Allow),
            (None, Some(guest_privileges.as_slice()), Rule::Allow),
            (Some(guest_roles.as_slice()), None, Rule::Deny),
            (None, Some(guest_privileges.as_slice()), Rule::Deny),
            (
                Some(guest_roles.as_slice()),
                Some(guest_privileges.as_slice()),
                Rule::Allow,
            ),
            (
                Some(user_roles.as_slice()),
                Some(user_privileges.as_slice()),
                Rule::Allow,
            ),
            (
                Some(admin_roles.as_slice()),
                Some(admin_privileges.as_slice()),
                Rule::Allow,
            ),
            (
                Some(guest_roles.as_slice()),
                Some(guest_privileges.as_slice()),
                Rule::Deny,
            ),
            (
                Some(user_roles.as_slice()),
                Some(user_privileges.as_slice()),
                Rule::Deny,
            ),
            (
                Some(admin_roles.as_slice()),
                Some(admin_privileges.as_slice()),
                Rule::Deny,
            ),
            // Cases to trigger lines 317-320: non-empty roles, empty privileges
            (
                Some(guest_roles.as_slice()),
                Some(vec![].as_slice()),
                Rule::Allow,
            ),
            (
                Some(user_roles.as_slice()),
                Some(vec![].as_slice()),
                Rule::Deny,
            ),
            // Cases to trigger lines 322-325: empty roles, non-empty privileges
            (
                Some(vec![].as_slice()),
                Some(guest_privileges.as_slice()),
                Rule::Allow,
            ),
            (
                Some(vec![].as_slice()),
                Some(user_privileges.as_slice()),
                Rule::Deny,
            ),
        ] {
            let mut role_privilege_rules = RolePrivilegeRules::new(false);
            test_constructed_defaults(&role_privilege_rules, false);

            let mut role_privilege_rules_2 = RolePrivilegeRules::new(true);
            test_constructed_defaults(&role_privilege_rules_2, true);

            // Add privilege rules, either "for all roles", or for given roles (per role)
            let mut privilege_rules = PrivilegeRules::new(false);
            let privilege_rules = match privilege_ids.as_ref() {
                Some(p_ids) => {
                    if !p_ids.is_empty() {
                        p_ids.iter().for_each(|p_id| {
                            privilege_rules
                                .by_privilege_id
                                .get_or_insert(HashMap::new())
                                .insert(p_id.to_string(), expected_rule);
                        });
                    } else {
                        privilege_rules.for_all_privileges = expected_rule;
                    }
                    Some(privilege_rules)
                }
                _ => {
                    privilege_rules.for_all_privileges = expected_rule;
                    Some(privilege_rules)
                }
            };

            // Set side-effects
            role_privilege_rules.set_privilege_rules(role_ids, privilege_rules.clone());
            role_privilege_rules_2.set_privilege_rules(role_ids, privilege_rules.clone());

            // Log iteration name
            // println!(
            //   "\n#RolePrivilegeRules.set_privilege_rules for ({:?}, {:?}, {:?})",
            //   &role_ids, &privilege_ids, &expected_rule
            // );

            // Test assertions
            // ----
            // If role_ids and privilege_ids
            if role_ids.is_some() && privilege_ids.is_some() {
                role_ids.zip(privilege_ids).map(|(r_ids, p_ids)| {
                    let p_ids_len = p_ids.len();
                    let r_ids_len = r_ids.len();

                    // if role ids len, and privilege ids len
                    if r_ids_len > 0 && p_ids_len > 0 {
                        test_when_roles_and_privileges(r_ids, p_ids, &role_privilege_rules, &expected_rule);
                        test_when_roles_and_privileges(r_ids, p_ids, &role_privilege_rules_2, &expected_rule);
                    }
                    // If only role IDs len
                    else if r_ids_len > 0 && p_ids_len == 0 {
                        test_when_only_roles(r_ids, &role_privilege_rules, &expected_rule);
                        test_when_only_roles(r_ids, &role_privilege_rules_2, &expected_rule);
                    }
                    // If only privilege IDs len
                    else if r_ids_len == 0 && p_ids_len > 0 {
                        test_when_only_privileges(p_ids, &role_privilege_rules, &expected_rule);
                        test_when_only_privileges(p_ids, &role_privilege_rules_2, &expected_rule);
                    }
                    // If no ID lengths
                    else if r_ids_len == 0 && p_ids_len == 0 {
                        test_when_no_roles_no_privileges(&role_privilege_rules, &expected_rule);
                        test_when_no_roles_no_privileges(&role_privilege_rules_2, &expected_rule);
                    }
                });
            } else if role_ids.is_some() {
                test_when_only_roles(
                    role_ids.as_ref().unwrap(),
                    &role_privilege_rules,
                    &expected_rule,
                );
                test_when_only_roles(
                    role_ids.as_ref().unwrap(),
                    &role_privilege_rules_2,
                    &expected_rule,
                );
            } else if privilege_ids.is_some() {
                test_when_only_privileges(
                    privilege_ids.as_ref().unwrap(),
                    &role_privilege_rules,
                    &expected_rule,
                );
                test_when_only_privileges(
                    privilege_ids.as_ref().unwrap(),
                    &role_privilege_rules_2,
                    &expected_rule,
                );
            } else {
                test_when_no_roles_no_privileges(&role_privilege_rules, &expected_rule);
                test_when_no_roles_no_privileges(&role_privilege_rules_2, &expected_rule);
            }
        }
    }

    #[test]
    fn test_get_privilege_rules_mut() {
        // Test with no role (None) - should return for_all_roles
        let mut rprs = RolePrivilegeRules::new(true);

        // Verify initial state
        assert_eq!(
            rprs.get_privilege_rules_mut(None).for_all_privileges,
            Rule::Deny,
            "Default for_all_privileges should be Deny"
        );

        // Test mutation via returned mutable reference
        rprs.get_privilege_rules_mut(None).for_all_privileges = Rule::Allow;
        assert_eq!(
            rprs.for_all_roles.for_all_privileges,
            Rule::Allow,
            "for_all_roles should be mutated to Allow"
        );

        // Test with role that doesn't exist in map - should return for_all_roles
        let mut rprs2 = RolePrivilegeRules::new(true);
        rprs2.get_privilege_rules_mut(Some("nonexistent")).for_all_privileges = Rule::Allow;
        assert_eq!(
            rprs2.for_all_roles.for_all_privileges,
            Rule::Allow,
            "Should fall back to for_all_roles when role not found"
        );

        // Test with role that exists in map
        let mut rprs3 = RolePrivilegeRules::new(true);
        let admin_role = "admin";

        // First set up a role in the map
        rprs3.set_privilege_rules_for_role_ids(&[admin_role], PrivilegeRules::new(true));

        // Now get mutable reference and modify it
        let admin_rules = rprs3.get_privilege_rules_mut(Some(admin_role));
        admin_rules.for_all_privileges = Rule::Allow;

        // Verify the role-specific rules were modified
        let stored_rules = rprs3.by_role_id.as_ref().unwrap().get(admin_role).unwrap();
        assert_eq!(
            stored_rules.for_all_privileges,
            Rule::Allow,
            "Role-specific privilege rules should be mutated"
        );

        // Verify for_all_roles was NOT modified
        assert_eq!(
            rprs3.for_all_roles.for_all_privileges,
            Rule::Deny,
            "for_all_roles should remain unchanged"
        );

        // Test with None when by_role_id map is None
        let mut rprs4 = RolePrivilegeRules::new(false);
        rprs4.get_privilege_rules_mut(None).for_all_privileges = Rule::Allow;
        assert_eq!(
            rprs4.for_all_roles.for_all_privileges,
            Rule::Allow,
            "Should mutate for_all_roles when by_role_id is None"
        );

        // Test with Some role when by_role_id map is None - should return for_all_roles
        let mut rprs5 = RolePrivilegeRules::new(false);
        rprs5.get_privilege_rules_mut(Some("admin")).for_all_privileges = Rule::Allow;
        assert_eq!(
            rprs5.for_all_roles.for_all_privileges,
            Rule::Allow,
            "Should fall back to for_all_roles when by_role_id is None"
        );
    }

    #[test]
    fn test_set_privilege_rules_explicit_branches() {
        use crate::simple::RuleContextScope;

        // Branch 1: privilege_rules.is_some() && role_ids.is_none()
        // This sets for_all_roles directly
        {
            let mut rprs = RolePrivilegeRules::new(true);
            let mut privilege_rules = PrivilegeRules::new(true);
            privilege_rules.for_all_privileges = Rule::Allow;

            let scope = rprs.set_privilege_rules(None, Some(privilege_rules.clone()));

            assert_eq!(scope, RuleContextScope::ForAllSymbols, "Should return ForAllSymbols scope");
            assert_eq!(
                rprs.for_all_roles.for_all_privileges,
                Rule::Allow,
                "for_all_roles should be set to the provided privilege_rules"
            );
        }

        // Branch 2: privilege_rules.is_none() && role_ids.is_some() (with non-empty roles)
        // This creates default PrivilegeRules for the specified roles
        {
            let mut rprs = RolePrivilegeRules::new(true);
            let roles: &[&str] = &["admin", "user"];

            let scope = rprs.set_privilege_rules(Some(roles), None);

            assert_eq!(scope, RuleContextScope::PerSymbol, "Should return PerSymbol scope");

            // Verify roles were added with default PrivilegeRules
            for role in roles {
                let stored = rprs.by_role_id.as_ref().unwrap().get(*role);
                assert!(stored.is_some(), "Role '{}' should exist in by_role_id", role);
                assert_eq!(
                    stored.unwrap().for_all_privileges,
                    Rule::Deny,
                    "Default PrivilegeRules should have Deny rule"
                );
            }
        }

        // Branch 2 variant: privilege_rules.is_none() && role_ids.is_some() (with empty roles)
        // This should set for_all_roles to default PrivilegeRules
        {
            let mut rprs = RolePrivilegeRules::new(true);
            // First set a non-default value
            rprs.for_all_roles.for_all_privileges = Rule::Allow;

            let empty_roles: &[&str] = &[];
            let scope = rprs.set_privilege_rules(Some(empty_roles), None);

            assert_eq!(scope, RuleContextScope::ForAllSymbols, "Should return ForAllSymbols scope for empty roles");
            assert_eq!(
                rprs.for_all_roles.for_all_privileges,
                Rule::Deny,
                "for_all_roles should be reset to default PrivilegeRules"
            );
        }

        // Branch 3: Both None - resets for_all_roles to default
        {
            let mut rprs = RolePrivilegeRules::new(true);
            rprs.for_all_roles.for_all_privileges = Rule::Allow;

            let scope = rprs.set_privilege_rules(None, None);

            assert_eq!(scope, RuleContextScope::ForAllSymbols, "Should return ForAllSymbols scope");
            assert_eq!(
                rprs.for_all_roles.for_all_privileges,
                Rule::Deny,
                "for_all_roles should be reset to default"
            );
        }

        // Branch 4: Both Some with non-empty role_ids
        {
            let mut rprs = RolePrivilegeRules::new(true);
            let mut privilege_rules = PrivilegeRules::new(true);
            privilege_rules.for_all_privileges = Rule::Allow;
            let roles: &[&str] = &["editor"];

            let scope = rprs.set_privilege_rules(Some(roles), Some(privilege_rules.clone()));

            assert_eq!(scope, RuleContextScope::PerSymbol, "Should return PerSymbol scope");
            let stored = rprs.by_role_id.as_ref().unwrap().get("editor").unwrap();
            assert_eq!(
                stored.for_all_privileges,
                Rule::Allow,
                "Role-specific rules should be set"
            );
        }

        // Branch 4 variant: Both Some with empty role_ids
        {
            let mut rprs = RolePrivilegeRules::new(true);
            let mut privilege_rules = PrivilegeRules::new(true);
            privilege_rules.for_all_privileges = Rule::Allow;
            let empty_roles: &[&str] = &[];

            let scope = rprs.set_privilege_rules(Some(empty_roles), Some(privilege_rules.clone()));

            assert_eq!(scope, RuleContextScope::ForAllSymbols, "Should return ForAllSymbols scope for empty roles");
            assert_eq!(
                rprs.for_all_roles.for_all_privileges,
                Rule::Allow,
                "for_all_roles should be set when role_ids is empty"
            );
        }
    }
}

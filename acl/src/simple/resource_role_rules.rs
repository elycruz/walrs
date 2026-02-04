use std::collections::HashMap;
use crate::simple::{Resource, RuleContextScope};
use crate::simple::role_privilege_rules::RolePrivilegeRules;

#[derive(Debug, PartialEq, Clone)]
pub struct ResourceRoleRules {
    pub for_all_resources: RolePrivilegeRules,
    // @todo Update implementation to use `Option<...>` here.
    pub by_resource_id: HashMap<Resource, RolePrivilegeRules>,
}

impl ResourceRoleRules {
    pub fn new() -> Self {
        ResourceRoleRules {
            for_all_resources: RolePrivilegeRules::new(true),
            by_resource_id: HashMap::new(),
        }
    }

    pub fn get_role_privilege_rules(&self, resource: Option<&str>) -> &RolePrivilegeRules {
        resource
            .and_then(|resource| self.by_resource_id.get(resource))
            .unwrap_or(&self.for_all_resources)
    }

    pub fn get_role_privilege_rules_mut(
        &mut self,
        resource: Option<&str>,
    ) -> &mut RolePrivilegeRules {
        resource
            .and_then(|resource| self.by_resource_id.get_mut(resource))
            .unwrap_or(&mut self.for_all_resources)
    }

    pub fn get_or_create_role_privilege_rules_mut(
        &mut self,
        resource: Option<&str>,
    ) -> &mut RolePrivilegeRules {
        resource
            .and_then(|resource| self.by_resource_id.get_mut(resource))
            .unwrap_or(&mut self.for_all_resources)
    }

    pub fn set_role_privilege_rules(
        &mut self,
        resources: Option<&[&str]>,
        role_privilege_rules: Option<RolePrivilegeRules>,
    ) -> RuleContextScope {
        let _role_privilege_rules = role_privilege_rules.unwrap_or(RolePrivilegeRules::new(false));
        match resources {
            Some(resource_ids) => {
                if !resource_ids.is_empty() {
                    resource_ids.iter().for_each(|r_id| {
                        self
                            .by_resource_id
                            .insert(r_id.to_string(), _role_privilege_rules.clone());
                    });
                } else {
                    self.for_all_resources = _role_privilege_rules;
                }
                RuleContextScope::PerSymbol
            }
            _ => {
                self.for_all_resources = _role_privilege_rules;
                RuleContextScope::ForAllSymbols
            }
        }
    }
}

#[cfg(test)]
mod test_resource_role_rules {
    use crate::simple::PrivilegeRules;
    use crate::simple::rule::Rule;
    use super::*;

    #[test]
    fn test_get_and_set_role_privilege_rules() {
        // Role IDs
        let guest_role = "guest";
        let user_role = "user";
        let admin_role = "admin";

        // Resource IDs
        let users_resource = "users"; // only admin should have access
        let account_resource = "account"; // user, and inheritors of user, should have access
        let posts_resource = "posts"; // guests, and inheritors, guests, should have access
        let new_rpr = |create_internal_maps: bool| Some(RolePrivilegeRules::new(create_internal_maps));

        for (resources, role_priv_rules) in [
            (None, None),
            (Some([].as_slice()), None),
            (Some([posts_resource].as_slice()), None),
            (Some([posts_resource, account_resource].as_slice()), None),
            (Some([posts_resource].as_slice()), new_rpr(false)),
            (
                Some([posts_resource, account_resource].as_slice()),
                new_rpr(false),
            ),
            (Some([posts_resource].as_slice()), new_rpr(true)),
            (
                Some([posts_resource, account_resource].as_slice()),
                new_rpr(true),
            ),
        ]
            .into_iter()
        {
            let mut ctrl = ResourceRoleRules::new();

            ctrl.set_role_privilege_rules(resources.as_deref(), role_priv_rules.clone());

            // Ensure we have a result to compare internals to;  `ResourceRoleRules` struct's internals
            // sets actual `RolePrivilegeRule` objects when incoming role_priv_rules are `None`,
            // hence resolution here.
            let role_rules = role_priv_rules
                .as_ref()
                .map(|rules| rules.clone())
                .or(Some(RolePrivilegeRules::new(false)));

            // Set state
            resources
                .and_then(|resources| {
                    resources.iter().for_each(|r| {
                        assert_eq!(
                            ctrl.by_resource_id.get(*r),
                            role_rules.as_ref(),
                            "resource \"{}\" role rules not equal to expected",
                            r
                        );
                        assert_eq!(
                            ctrl.get_role_privilege_rules(Some(r)),
                            role_rules.as_ref().unwrap(),
                            "resource \"{}\" role rules not equal to expected",
                            r
                        );
                    });
                    if resources.is_empty() {
                        assert_eq!(&ctrl.for_all_resources, role_rules.as_ref().unwrap());
                        assert_eq!(
                            ctrl.get_role_privilege_rules(None),
                            role_rules.as_ref().unwrap()
                        );
                    }
                    Some(resources)
                })
                .or_else(|| {
                    assert_eq!(&ctrl.for_all_resources, role_rules.as_ref().unwrap());
                    assert_eq!(
                        ctrl.get_role_privilege_rules(None),
                        role_rules.as_ref().unwrap()
                    );
                    None
                });
        }
    }

    #[test]
    fn test_get_role_privilege_rules_mut() {
        let role = "admin";
        let resource = "existent";
        let non_existent_resource = "non-existent";

        let mut ctrl = ResourceRoleRules::new();

        // Test non-existent resource
        let mut rules_mut = ctrl.get_role_privilege_rules_mut(Some(non_existent_resource));
        assert_eq!(&rules_mut.for_all_roles.for_all_privileges, &Rule::Deny);
        assert!(rules_mut.for_all_roles.by_privilege_id.as_ref().unwrap().is_empty(), "by_resource_id map should be empty");
        assert!(rules_mut.by_role_id.as_ref().unwrap().is_empty(), "by_role_id map should be empty");

        // Test existing resource
        // ----
        // Initial state
        rules_mut = ctrl.get_role_privilege_rules_mut(Some(resource));
        assert_eq!(&rules_mut.for_all_roles.for_all_privileges, &Rule::Deny);

        // Update rules for "existing" resource
        let mut new_role_rules = RolePrivilegeRules::new(true);
        {
            // Set privileges
            let mut new_privileges = PrivilegeRules::new(true);
            // Allow all privileges
            new_privileges.set_rule(None, Rule::Allow);

            // Set privilege rules for role
            let mut priv_rules_map = HashMap::new();
            priv_rules_map.insert(role.to_string(), new_privileges);
            new_role_rules.by_role_id = Some(priv_rules_map);
        }

        ctrl.set_role_privilege_rules(Some(&[resource]), Some(new_role_rules));

        // Verify resource update
        let role_rules = ctrl.by_resource_id.get_mut(resource).unwrap();
        assert_eq!(role_rules.by_role_id.as_ref().unwrap().get(role).unwrap().for_all_privileges, Rule::Allow, "resource update should've taken effect");

        // Test mut rules getter
        let rules_mut = ctrl.get_role_privilege_rules_mut(Some(resource));
        assert_eq!(rules_mut.clone(), ctrl.by_resource_id.get(resource).unwrap().clone(), "mut rules getter should return the same object as the resource's rules");
    }

    #[test]
    fn test_get_or_create_role_privilege_rules_mut() {
        let role = "admin";
        let resource = "existent";
        let non_existent_resource = "non-existent";

        let mut ctrl = ResourceRoleRules::new();

        // Test with non-existent resource - currently returns for_all_resources (doesn't create)
        {
            let rules_mut = ctrl.get_or_create_role_privilege_rules_mut(Some(non_existent_resource));
            assert_eq!(&rules_mut.for_all_roles.for_all_privileges, &Rule::Deny);
            assert!(rules_mut.for_all_roles.by_privilege_id.as_ref().unwrap().is_empty(), "by_privilege_id map should be empty");
            assert!(rules_mut.by_role_id.as_ref().unwrap().is_empty(), "by_role_id map should be empty");

            // Modify to verify we're modifying for_all_resources
            rules_mut.for_all_roles.for_all_privileges = Rule::Allow;
        }
        assert_eq!(ctrl.for_all_resources.for_all_roles.for_all_privileges, Rule::Allow,
            "For non-existent resource, modification should affect for_all_resources");

        // Reset control
        ctrl = ResourceRoleRules::new();

        // Test with None resource - should return for_all_resources
        {
            let rules_mut = ctrl.get_or_create_role_privilege_rules_mut(None);
            assert_eq!(&rules_mut.for_all_roles.for_all_privileges, &Rule::Deny);
            rules_mut.for_all_roles.for_all_privileges = Rule::Allow;
        }
        assert_eq!(ctrl.for_all_resources.for_all_roles.for_all_privileges, Rule::Allow,
            "For None resource, modification should affect for_all_resources");

        // Reset for_all_resources
        ctrl = ResourceRoleRules::new();

        // Test existing resource
        // ----
        // Create a resource entry first
        let mut new_role_rules = RolePrivilegeRules::new(true);
        {
            // Set privileges
            let mut new_privileges = PrivilegeRules::new(true);
            // Allow all privileges
            new_privileges.set_rule(None, Rule::Allow);

            // Set privilege rules for role
            let mut priv_rules_map = HashMap::new();
            priv_rules_map.insert(role.to_string(), new_privileges);
            new_role_rules.by_role_id = Some(priv_rules_map);
        }

        ctrl.set_role_privilege_rules(Some(&[resource]), Some(new_role_rules));

        // Verify resource update
        {
            let role_rules = ctrl.by_resource_id.get_mut(resource).unwrap();
            assert_eq!(role_rules.by_role_id.as_ref().unwrap().get(role).unwrap().for_all_privileges, Rule::Allow,
                "resource update should've taken effect");
        }

        // Test get_or_create_role_privilege_rules_mut with existing resource
        {
            let rules_mut = ctrl.get_or_create_role_privilege_rules_mut(Some(resource));
            assert_eq!(rules_mut.by_role_id.as_ref().unwrap().get(role).unwrap().for_all_privileges, Rule::Allow,
                "get_or_create should return existing resource rules");
        }

        // Verify resource rules match expected
        let rules_mut = ctrl.get_or_create_role_privilege_rules_mut(Some(resource));
        assert_eq!(rules_mut.clone(), ctrl.by_resource_id.get(resource).unwrap().clone(),
            "get_or_create should return the same object as the resource's rules");
    }
}

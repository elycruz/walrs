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
}

use crate::{can_permission, Permission, Role, User};

pub struct RolePolicy;

impl RolePolicy {
    pub fn can_show(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::RolesShow);
    }
    pub fn can_create(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::RolesCreate);
    }
    pub fn can_update(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::RolesUpdate);
    }
    pub fn can_delete(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::RolesDelete);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy() {
        let mut user = User::empty("".to_string());
        let mut user_roles: Vec<Role> = vec![Role::default()];
        assert_eq!(RolePolicy::can_show(&user, &user_roles), false);
        user.is_super_admin = true;
        assert_eq!(RolePolicy::can_show(&user, &user_roles), true);
        user.is_super_admin = false;
        assert_eq!(RolePolicy::can_show(&user, &user_roles), false);
        user.roles_ids = Some(vec![1]);
        assert_eq!(RolePolicy::can_show(&user, &user_roles), false);
        let mut role = Role::default();
        role.id = 1;
        role.permissions = Some(vec![
            Permission::RolesShow.to_string(),
            Permission::RolesDelete.to_string(),
        ]);
        user_roles = vec![role];
        assert_eq!(RolePolicy::can_show(&user, &user_roles), true);
        assert_eq!(RolePolicy::can_delete(&user, &user_roles), true);
        assert_eq!(RolePolicy::can_create(&user, &user_roles), false);
        assert_eq!(RolePolicy::can_update(&user, &user_roles), false);
    }
}

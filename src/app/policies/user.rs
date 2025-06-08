use crate::{can_permission, Permission, Role, User};

pub struct UserPolicy;

impl UserPolicy {
    pub fn can_show(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::UsersShow);
    }
    pub fn can_create(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::UsersCreate);
    }
    pub fn can_update(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::UsersUpdate);
    }
    pub fn can_delete(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::UsersDelete);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy() {
        let mut user = User::empty("".to_string());
        let mut user_roles: Vec<Role> = vec![Role::default()];
        assert_eq!(UserPolicy::can_show(&user, &user_roles), false);
        user.is_super_admin = true;
        assert_eq!(UserPolicy::can_show(&user, &user_roles), true);
        user.is_super_admin = false;
        assert_eq!(UserPolicy::can_show(&user, &user_roles), false);
        user.roles_ids = Some(vec![1]);
        assert_eq!(UserPolicy::can_show(&user, &user_roles), false);
        let mut role = Role::default();
        role.id = 1;
        role.permissions = Some(vec![
            Permission::UsersShow.to_string(),
            Permission::UsersDelete.to_string(),
        ]);
        user_roles = vec![role];
        assert_eq!(UserPolicy::can_show(&user, &user_roles), true);
        assert_eq!(UserPolicy::can_delete(&user, &user_roles), true);
        assert_eq!(UserPolicy::can_create(&user, &user_roles), false);
        assert_eq!(UserPolicy::can_update(&user, &user_roles), false);
    }
}

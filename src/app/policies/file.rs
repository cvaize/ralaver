use crate::{can_permission, Permission, User, Role};

pub struct FilePolicy;

impl FilePolicy {
    pub fn can_show(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::FilesShow);
    }
    pub fn can_create(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::FilesCreate);
    }
    pub fn can_update(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::FilesUpdate);
    }
    pub fn can_delete(user: &User, user_roles: &Vec<Role>) -> bool {
        can_permission!(user, user_roles, Permission::FilesDelete);
    }
}
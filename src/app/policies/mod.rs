mod file;
mod role;
mod user;

pub use self::file::{*};
pub use self::role::{*};
pub use self::user::{*};

pub struct PolicyError;

#[macro_export]
macro_rules! return_true_is_super_admin {
    ($user:expr) => {
        if $user.is_super_admin {
            return true;
        }
    };
}

#[macro_export]
macro_rules! return_false_is_roles_ids_empty {
    ($user:expr) => {
        if $user.roles_ids.is_none() {
            return false;
        }
        if let Some(roles_ids) = &$user.roles_ids {
            if roles_ids.len() == 0 {
                return false;
            }
        }
    };
}

#[macro_export]
macro_rules! return_is_super_admin_or_roles_empty {
    ($user:expr) => {
        if $user.is_super_admin {
            return true;
        }
        if $user.roles_ids.is_none() {
            return false;
        }
        if let Some(roles_ids) = &$user.roles_ids {
            if roles_ids.len() == 0 {
                return false;
            }
        }
    };
}

#[macro_export]
macro_rules! can_permission {
    ($user:expr, $roles:expr, $permission:expr) => {
        if $user.is_super_admin {
            return true;
        }
        if $user.roles_ids.is_none() {
            return false;
        }
        if let Some(roles_ids) = &$user.roles_ids {
            if roles_ids.len() == 0 {
                return false;
            }
        }
        let roles_ids = $user.roles_ids.as_ref().unwrap();
        for role in $roles {
            if let Some(permissions) = &role.permissions {
                if permissions.len() != 0 && roles_ids.contains(&role.id) {
                    if permissions.contains(&$permission.to_string()) {
                        return true;
                    }
                }
            }
        }
        return false;
    };
}
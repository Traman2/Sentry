//! System user accounts, exposed as data (as opposed to [`super::process`],
//! which only uses [`Users`] to resolve a process owner's display name).

use serde::Serialize;
use sysinfo::Users;

#[derive(Debug, Clone, Serialize)]
pub struct UserAccount {
    pub uid: String,
    pub gid: String,
    pub name: String,
    pub groups: Vec<String>,
}

pub fn collect(users: &Users) -> Vec<UserAccount> {
    users
        .list()
        .iter()
        .map(|user| UserAccount {
            uid: user.id().to_string(),
            gid: user.group_id().to_string(),
            name: user.name().to_string(),
            groups: user
                .groups()
                .into_iter()
                .map(|group| group.name().to_string())
                .collect(),
        })
        .collect()
}

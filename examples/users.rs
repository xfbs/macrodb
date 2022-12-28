use macrodb::table;
use std::collections::*;

type UserId = u64;
type GroupId = u64;

#[derive(Clone)]
pub struct User {
    id: UserId,
    name: String,
    group: GroupId,
}

#[derive(Clone)]
pub struct Group {
    id: GroupId,
    name: String,
}

pub enum Error {
    UserNotFound,
    UserIdExists,
    UserNameExists,
    GroupNotFound,
    GroupIdExists,
    GroupNameExists,
}

#[derive(Default)]
pub struct Database {
    users: BTreeMap<UserId, User>,
    user_by_name: HashMap<String, UserId>,
    groups: BTreeMap<GroupId, Group>,
    group_by_name: HashMap<String, GroupId>,
    users_by_group: BTreeMap<GroupId, BTreeSet<UserId>>,
}

impl Database {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        foreign groups group => Error::GroupNotFound,
        unique user_by_name name => Error::UserNameExists
    );
    table!(
        groups: Group,
        id: GroupId,
        missing Error => Error::GroupNotFound,
        primary groups id => Error::GroupIdExists,
        unique group_by_name name => Error::GroupNameExists
    );
}

fn main() {
    let mut database = Database::default();
}

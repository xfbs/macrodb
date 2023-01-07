use macrodb::table;
use std::collections::{BTreeMap as Map, BTreeSet as Set};

type UserId = String;

#[derive(Clone, Debug, PartialEq, Eq)]
struct User {
    id: UserId,
    name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Error {
    UserIdExists,
    UserNotFound,
    UserNameExists,
}

#[derive(Clone, Debug, Default)]
struct Database {
    users: Map<UserId, User>,
    user_by_name: Map<String, UserId>,
}

impl Database {
    table!(
        users: User,
        id: UserId,
        noautokey,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_name name => Error::UserNameExists
    );
}

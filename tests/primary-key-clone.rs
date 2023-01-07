use macrodb::table;
use std::collections::{BTreeMap as Map, BTreeSet as Set};

type UserId = String;

#[derive(Clone, Debug, PartialEq, Eq)]
struct User {
    id: UserId,
    name: String,
    age: u16,
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
    users_by_age: Map<u16, Set<UserId>>,
}

impl Database {
    table!(
        users: User,
        id: UserId,
        noautokey,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        index users_by_age age => (),
        unique user_by_name name => Error::UserNameExists
    );
}

#[test]
fn can_insert() {
    let mut database = Database::default();
    let user = User {
        id: "id".into(),
        name: "name".into(),
        age: 21,
    };
    database.users_insert(user).unwrap();
}

#[test]
fn cannot_insert_existing_id() {
    let mut database = Database::default();
    let user = User {
        id: "id".into(),
        name: "name".into(),
        age: 21,
    };
    database.users_insert(user).unwrap();
    let user = User {
        id: "id".into(),
        name: "other".into(),
        age: 21,
    };
    let result = database.users_insert(user);
    assert_eq!(result, Err(Error::UserIdExists));
}

#[test]
fn cannot_insert_existing_name() {
    let mut database = Database::default();
    let user = User {
        id: "id".into(),
        name: "name".into(),
        age: 21,
    };
    database.users_insert(user).unwrap();
    let user = User {
        id: "other".into(),
        name: "name".into(),
        age: 21,
    };
    let result = database.users_insert(user);
    assert_eq!(result, Err(Error::UserNameExists));
}

#[test]
fn can_update() {
    let mut database = Database::default();
    let mut user = User {
        id: "id".into(),
        name: "name".into(),
        age: 21,
    };
    database.users_insert(user.clone()).unwrap();
    user.name = "other".into();
    database.users_update(user.clone()).unwrap();
    user.age = 22;
    database.users_update(user.clone()).unwrap();
}

#[test]
fn can_delete() {
    let mut database = Database::default();
    let user = User {
        id: "id".into(),
        name: "name".into(),
        age: 21,
    };
    database.users_insert(user.clone()).unwrap();
    database.users_delete(user.id.clone()).unwrap();
    assert!(database.users.is_empty());
    assert!(database.user_by_name.is_empty());
    assert!(database.users_by_age.is_empty());
}

#[test]
fn cannot_delete_missing() {
    let mut database = Database::default();
    let result = database.users_delete("missing".into());
    assert_eq!(result, Err(Error::UserNotFound));
}

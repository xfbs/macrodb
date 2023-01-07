use crate::table;
use std::collections::{BTreeMap as Map, BTreeSet as Set};

type UserId = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
struct User {
    id: UserId,
    first: String,
    last: String,
    age: u16,
    city: String,
    country: String,
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
    user_by_name: Map<(String, String), UserId>,
    users_by_age: Map<u16, Set<UserId>>,
    users_by_location: Map<(String, String), Set<UserId>>,
}

impl Database {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        index users_by_location (city, country) => (),
        unique user_by_name (first, last) => Error::UserNameExists,
        index users_by_age age => ()
    );
}

#[test]
fn can_insert_user() {
    let mut database = Database::default();
    let id = database.users_next_id();
    let user = User {
        id,
        first: "John".into(),
        last: "Doe".into(),
        age: 21,
        city: "Atlantic City".into(),
        country: "United States".into(),
    };
    database.users_insert(user.clone()).unwrap();
    assert_eq!(database.users.get(&id), Some(&user));
    assert!(database.users_by_age.get(&user.age).unwrap().contains(&id));
    assert!(database
        .users_by_location
        .get(&(user.city.clone(), user.country.clone()))
        .unwrap()
        .contains(&id));
    assert_eq!(
        database
            .user_by_name
            .get(&(user.first.clone(), user.last.clone())),
        Some(&id)
    );
}

#[test]
fn cannot_insert_user_existing_name() {
    let mut database = Database::default();
    database
        .users_insert(User {
            id: database.users_next_id(),
            first: "John".into(),
            last: "Doe".into(),
            age: 21,
            city: "Atlantic City".into(),
            country: "United States".into(),
        })
        .unwrap();
    let result = database.users_insert(User {
        id: database.users_next_id(),
        first: "John".into(),
        last: "Doe".into(),
        age: 24,
        city: "Elsewhere".into(),
        country: "United States".into(),
    });
    assert_eq!(result, Err(Error::UserNameExists));
}

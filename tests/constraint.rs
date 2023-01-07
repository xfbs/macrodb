use macrodb::table;
use std::collections::BTreeMap as Map;

type UserId = i64;

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
    UserNameEmpty,
}

#[derive(Clone, Debug, Default)]
struct Database {
    users: Map<UserId, User>,
    user_by_name: Map<String, UserId>,
}

impl Database {
    fn check_user(&self, row: &User) -> Result<(), Error> {
        if row.name.is_empty() {
            return Err(Error::UserNameEmpty);
        }

        Ok(())
    }

    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_name name => Error::UserNameExists,
        constraint check_user _ => ()
    );
}

#[test]
fn can_insert() {
    let mut database = Database::default();
    let user = User {
        id: database.users_next_id(),
        name: "name".into(),
    };
    database.users_insert(user).unwrap();
}

#[test]
fn cannot_insert_user_empty_name() {
    let mut database = Database::default();
    let user = User {
        id: database.users_next_id(),
        name: "".into(),
    };
    let result = database.users_insert(user);
    assert_eq!(result, Err(Error::UserNameEmpty));
}

#[test]
fn cannot_update_user_empty_name() {
    let mut database = Database::default();
    let mut user = User {
        id: database.users_next_id(),
        name: "John".into(),
    };
    database.users_insert(user.clone()).unwrap();
    user.name = "".into();
    let result = database.users_update(user);
    assert_eq!(result, Err(Error::UserNameEmpty));
}

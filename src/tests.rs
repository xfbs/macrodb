use super::table;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Errors that can result from database operations.
#[derive(Debug, Clone, PartialEq, Eq)]
enum UserError {
    UserNotFound,
    UserIdExists,
    UserEmailExists,
}

/// Primary key of user records
type UserId = u64;

/// User record
#[derive(Clone, Debug, PartialEq, Eq)]
struct User {
    id: UserId,
    name: String,
    email: String,
    age: u32,
}

impl Default for User {
    fn default() -> Self {
        User {
            id: 0,
            name: "user".into(),
            email: "user@example.com".into(),
            age: 21,
        }
    }
}

/// User database
#[derive(Default)]
struct Users {
    /// Stores User records
    users: BTreeMap<UserId, User>,
    /// Users by email unique index
    user_by_email: BTreeMap<String, UserId>,
    /// Users by age index
    users_by_age: HashMap<u32, HashSet<UserId>>,
    /// Users by name index
    users_by_name: BTreeMap<String, BTreeSet<UserId>>,
}

impl Users {
    table!(
        users: User,
        id: UserId,
        missing UserError => UserError::UserNotFound,
        primary users id => UserError::UserIdExists,
        unique user_by_email email => UserError::UserEmailExists,
        index users_by_age age => (),
        index users_by_name name => ()
    );
}

#[test]
fn can_insert_user() {
    let mut data = Users::default();
    let user = User::default();
    data.users_insert(user.clone()).unwrap();
    assert_eq!(data.users.get(&user.id), Some(&user));
    assert_eq!(data.user_by_email.get(&user.email), Some(&user.id));
    assert_eq!(
        data.users_by_age.get(&user.age),
        Some(&[user.id].into_iter().collect())
    );
    assert_eq!(
        data.users_by_name.get(&user.name),
        Some(&[user.id].into_iter().collect())
    );
}

#[test]
fn cannot_insert_user_existing_id() {
    let mut data = Users::default();
    let user = User::default();
    data.users_insert(user.clone()).unwrap();
    assert_eq!(
        data.users_insert(user.clone()),
        Err(UserError::UserIdExists)
    );
}

#[test]
fn cannot_insert_user_existing_email() {
    let mut data = Users::default();
    let mut user = User::default();
    data.users_insert(user.clone()).unwrap();
    user.id += 1;
    assert_eq!(
        data.users_insert(user.clone()),
        Err(UserError::UserEmailExists)
    );
}

#[test]
fn can_insert_two_users() {
    let mut data = Users::default();
    let user1 = User::default();
    data.users_insert(user1.clone()).unwrap();

    let mut user2 = user1.clone();
    user2.id += 1;
    user2.email = "other@example.com".into();
    data.users_insert(user2.clone()).unwrap();

    assert_eq!(data.users.get(&user1.id), Some(&user1));
    assert_eq!(data.users.get(&user2.id), Some(&user2));

    assert_eq!(data.user_by_email.get(&user1.email), Some(&user1.id));
    assert_eq!(data.user_by_email.get(&user2.email), Some(&user2.id));

    assert_eq!(
        data.users_by_age.get(&user1.age),
        Some(&[user1.id, user2.id].into_iter().collect())
    );
    assert_eq!(
        data.users_by_name.get(&user1.name),
        Some(&[user1.id, user2.id].into_iter().collect())
    );
}

#[test]
fn can_insert_three_users() {
    let mut data = Users::default();
    let user1 = User::default();
    data.users_insert(user1.clone()).unwrap();

    let mut user2 = user1.clone();
    user2.id += 1;
    user2.email = "other@example.com".into();
    data.users_insert(user2.clone()).unwrap();

    let mut user3 = user2.clone();
    user3.id += 1;
    user3.email = "old@example.com".into();
    user3.age = 70;
    data.users_insert(user3.clone()).unwrap();

    assert_eq!(data.users.get(&user1.id), Some(&user1));
    assert_eq!(data.users.get(&user2.id), Some(&user2));
    assert_eq!(data.users.get(&user3.id), Some(&user3));

    assert_eq!(data.user_by_email.get(&user1.email), Some(&user1.id));
    assert_eq!(data.user_by_email.get(&user2.email), Some(&user2.id));
    assert_eq!(data.user_by_email.get(&user3.email), Some(&user3.id));

    assert_eq!(
        data.users_by_age.get(&user1.age),
        Some(&[user1.id, user2.id].into_iter().collect())
    );
    assert_eq!(
        data.users_by_age.get(&user3.age),
        Some(&[user3.id].into_iter().collect())
    );

    assert_eq!(
        data.users_by_name.get(&user1.name),
        Some(&[user1.id, user2.id, user3.id].into_iter().collect())
    );
}

#[test]
fn can_update_user_email() {
    let mut data = Users::default();
    let old = User::default();
    data.users_insert(old.clone()).unwrap();
    let mut new = old.clone();
    new.email = "new@example.com".into();
    let result = data.users_update(new.clone()).unwrap();
    assert_eq!(result, old);

    assert_eq!(data.users.get(&new.id), Some(&new));
    assert_eq!(data.user_by_email.get(&new.email), Some(&new.id));
    assert_eq!(data.user_by_email.get(&old.email), None);
    assert_eq!(
        data.users_by_age.get(&new.age),
        Some(&[new.id].into_iter().collect())
    );
    assert_eq!(
        data.users_by_name.get(&new.name),
        Some(&[new.id].into_iter().collect())
    );
}

#[test]
fn can_update_user_age() {
    let mut data = Users::default();
    let old = User::default();
    data.users_insert(old.clone()).unwrap();
    let mut new = old.clone();
    new.age = 30;
    let result = data.users_update(new.clone()).unwrap();
    assert_eq!(result, old);

    assert_eq!(data.users.get(&new.id), Some(&new));
    assert_eq!(data.user_by_email.get(&new.email), Some(&new.id));
    assert_eq!(data.users_by_age.get(&old.age), None);
    assert_eq!(
        data.users_by_age.get(&new.age),
        Some(&[new.id].into_iter().collect())
    );
    assert_eq!(
        data.users_by_name.get(&new.name),
        Some(&[new.id].into_iter().collect())
    );
}

#[test]
fn can_update_user_name() {
    let mut data = Users::default();
    let old = User::default();
    data.users_insert(old.clone()).unwrap();
    let mut new = old.clone();
    new.name = "other".into();
    let result = data.users_update(new.clone()).unwrap();
    assert_eq!(result, old);

    assert_eq!(data.users_by_name.get(&old.name), None);
    assert_eq!(
        data.users_by_name.get(&new.name),
        Some(&[new.id].into_iter().collect())
    );
}

#[test]
fn can_update_user_all() {
    let mut data = Users::default();
    let old = User::default();
    data.users_insert(old.clone()).unwrap();
    let mut new = old.clone();
    new.age = 30;
    new.email = "new@example.com".into();
    new.name = "other".into();
    let result = data.users_update(new.clone()).unwrap();
    assert_eq!(result, old);

    assert_eq!(data.users.get(&new.id), Some(&new));
    assert_eq!(data.user_by_email.get(&new.email), Some(&new.id));
    assert_eq!(data.user_by_email.get(&old.email), None);
    assert_eq!(data.users_by_age.get(&old.age), None);
    assert_eq!(
        data.users_by_age.get(&new.age),
        Some(&[new.id].into_iter().collect())
    );
    assert_eq!(data.users_by_name.get(&old.name), None);
    assert_eq!(
        data.users_by_name.get(&new.name),
        Some(&[new.id].into_iter().collect())
    );
}

#[test]
fn cannot_update_user_existing_email() {
    let mut data = Users::default();
    let user1 = User::default();
    data.users_insert(user1.clone()).unwrap();

    let mut user2 = User::default();
    user2.id += 1;
    user2.email = "other@example.com".into();
    data.users_insert(user2.clone()).unwrap();

    let mut user2_new = user2.clone();
    user2_new.email = user1.email.clone();
    let result = data.users_update(user2_new.clone());

    assert_eq!(result, Err(UserError::UserEmailExists));
}

#[test]
fn can_delete_user() {
    let mut data = Users::default();
    let user = User::default();
    data.users_insert(user.clone()).unwrap();
    data.users_delete(user.id).unwrap();
    assert!(data.users.is_empty());
    assert!(data.user_by_email.is_empty());
    assert!(data.users_by_age.is_empty());
    assert!(data.users_by_name.is_empty());
}

#[test]
fn cannot_delete_user_missing() {
    let mut data = Users::default();
    let user = User::default();
    data.users_insert(user.clone()).unwrap();
    let result = data.users_delete(user.id + 1);
    assert_eq!(result, Err(UserError::UserNotFound));
}

#[test]
fn can_delete_two_users() {
    let mut data = Users::default();
    let user1 = User {
        id: 0,
        email: "user@example.com".into(),
        age: 21,
        ..User::default()
    };
    let user2 = User {
        id: 1,
        email: "other@example.com".into(),
        age: 21,
        ..User::default()
    };
    data.users_insert(user1.clone()).unwrap();
    data.users_insert(user2.clone()).unwrap();
    let result = data.users_delete(user1.id);
    assert_eq!(result, Ok(user1.clone()));
    assert_eq!(data.users.get(&user1.id), None);
    assert_eq!(data.user_by_email.get(&user1.email), None);
    assert_eq!(
        data.users_by_age.get(&user2.age),
        Some(&[user2.id].into_iter().collect())
    );
    assert_eq!(
        data.users_by_name.get(&user2.name),
        Some(&[user2.id].into_iter().collect())
    );

    let result = data.users_delete(user2.id);
    assert_eq!(result, Ok(user2.clone()));
    assert_eq!(data.users.get(&user2.id), None);
    assert_eq!(data.user_by_email.get(&user2.email), None);
    assert_eq!(data.users_by_age.get(&user2.age), None);
    assert_eq!(data.users_by_name.get(&user2.name), None);
}

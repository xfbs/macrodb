use crate::table;
use avl::{AvlTreeMap, AvlTreeSet};
use hashbrown::{HashMap as HashMapBrown, HashSet as HashSetBrown};
use im::{HashMap as HashMapIm, OrdMap};
use rand::{thread_rng, Rng};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, EnumIter)]
enum Error {
    UserNotFound,
    UserIdExists,
    UserEmailExists,
    UserNameExists,
    GroupNotFound,
    GroupIdExists,
    GroupNameExists,
    GroupNotEmpty,
}

type UserId = u64;
type UserAge = u8;
type GroupId = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
struct User {
    id: UserId,
    group: GroupId,
    name: String,
    email: String,
    age: UserAge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Group {
    id: GroupId,
    name: String,
    privileged: bool,
}

#[derive(Default)]
struct StdDatabase {
    users: BTreeMap<UserId, User>,
    user_by_email: BTreeMap<String, UserId>,
    user_by_name: BTreeMap<String, UserId>,
    users_by_age: BTreeMap<UserAge, BTreeSet<UserId>>,
    users_by_group: HashMap<GroupId, HashSet<UserId>>,

    groups: HashMap<GroupId, Group>,
    group_by_name: HashMap<String, GroupId>,
    groups_by_privileged: HashMap<bool, HashSet<GroupId>>,
}

impl StdDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        unique user_by_name name => Error::UserNameExists,
        foreign groups group => Error::GroupNotFound,
        index users_by_age age => (),
        index users_by_group group => ()
    );
    table!(
        groups: Group,
        id: GroupId,
        missing Error => Error::GroupNotFound,
        primary groups id => Error::GroupIdExists,
        unique group_by_name name => Error::GroupNameExists,
        reverse users_by_group id => Error::GroupNotEmpty,
        index groups_by_privileged privileged => ()

    );
}

#[derive(Default)]
struct ImDatabase {
    users: OrdMap<UserId, User>,
    user_by_email: OrdMap<String, UserId>,
    user_by_name: OrdMap<String, UserId>,
    users_by_age: OrdMap<UserAge, BTreeSet<UserId>>,
    users_by_group: HashMapIm<GroupId, HashSet<UserId>>,

    groups: HashMapIm<GroupId, Group>,
    group_by_name: HashMapIm<String, GroupId>,
    groups_by_privileged: HashMapIm<bool, HashSet<GroupId>>,
}

impl ImDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        unique user_by_name name => Error::UserNameExists,
        foreign groups group => Error::GroupNotFound,
        index users_by_age age => (),
        index users_by_group group => ()
    );
    table!(
        groups: Group,
        id: GroupId,
        missing Error => Error::GroupNotFound,
        primary groups id => Error::GroupIdExists,
        unique group_by_name name => Error::GroupNameExists,
        reverse users_by_group id => Error::GroupNotEmpty,
        index groups_by_privileged privileged => ()

    );
}

#[derive(Default)]
struct AvlDatabase {
    users: AvlTreeMap<UserId, User>,
    user_by_email: AvlTreeMap<String, UserId>,
    user_by_name: AvlTreeMap<String, UserId>,
    users_by_age: AvlTreeMap<UserAge, AvlTreeSet<UserId>>,
    users_by_group: AvlTreeMap<GroupId, AvlTreeSet<UserId>>,

    groups: AvlTreeMap<GroupId, Group>,
    group_by_name: AvlTreeMap<String, GroupId>,
    groups_by_privileged: AvlTreeMap<bool, AvlTreeSet<GroupId>>,
}

impl AvlDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        unique user_by_name name => Error::UserNameExists,
        foreign groups group => Error::GroupNotFound,
        index users_by_age age => (),
        index users_by_group group => ()
    );
    table!(
        groups: Group,
        id: GroupId,
        missing Error => Error::GroupNotFound,
        primary groups id => Error::GroupIdExists,
        unique group_by_name name => Error::GroupNameExists,
        reverse users_by_group id => Error::GroupNotEmpty,
        index groups_by_privileged privileged => ()

    );
}

#[derive(Default)]
struct HashBrownDatabase {
    users: HashMapBrown<UserId, User>,
    user_by_email: HashMapBrown<String, UserId>,
    user_by_name: HashMapBrown<String, UserId>,
    users_by_age: HashMapBrown<UserAge, HashSetBrown<UserId>>,
    users_by_group: HashMapBrown<GroupId, HashSetBrown<UserId>>,

    groups: HashMapBrown<GroupId, Group>,
    group_by_name: HashMapBrown<String, GroupId>,
    groups_by_privileged: HashMapBrown<bool, HashSetBrown<GroupId>>,
}

impl HashBrownDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        unique user_by_name name => Error::UserNameExists,
        foreign groups group => Error::GroupNotFound,
        index users_by_age age => (),
        index users_by_group group => ()
    );
    table!(
        groups: Group,
        id: GroupId,
        missing Error => Error::GroupNotFound,
        primary groups id => Error::GroupIdExists,
        unique group_by_name name => Error::GroupNameExists,
        reverse users_by_group id => Error::GroupNotEmpty,
        index groups_by_privileged privileged => ()

    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum UsersOperation {
    Insert(User),
    Update(User),
    Delete(UserId),
}

impl UsersOperation {
    fn random_id<R: Rng + ?Sized>(rng: &mut R) -> u64 {
        rng.gen_range(0..1000)
    }

    fn random_name<R: Rng + ?Sized>(rng: &mut R) -> String {
        format!("user-{}", rng.gen_range(0..2000))
    }

    fn random_email<R: Rng + ?Sized>(rng: &mut R) -> String {
        format!("email-{}", rng.gen_range(0..10000))
    }

    fn random_age<R: Rng + ?Sized>(rng: &mut R) -> UserAge {
        rng.gen_range(21..60)
    }

    fn random_user<R: Rng + ?Sized>(rng: &mut R) -> User {
        User {
            id: Self::random_id(rng),
            name: Self::random_name(rng),
            email: Self::random_email(rng),
            age: Self::random_age(rng),
            group: GroupsOperation::random_id(rng),
        }
    }

    fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.gen_range(0..3) {
            0 => UsersOperation::Insert(Self::random_user(rng)),
            1 => UsersOperation::Update(Self::random_user(rng)),
            2 => UsersOperation::Delete(rng.gen_range(0..1000)),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GroupsOperation {
    Insert(Group),
    Update(Group),
    Delete(GroupId),
}

impl GroupsOperation {
    fn random_id<R: Rng + ?Sized>(rng: &mut R) -> u64 {
        rng.gen_range(0..100)
    }

    fn random_name<R: Rng + ?Sized>(rng: &mut R) -> String {
        format!("group-{}", rng.gen_range(0..200))
    }

    fn random_privileged<R: Rng + ?Sized>(rng: &mut R) -> bool {
        rng.gen()
    }

    fn random_group<R: Rng + ?Sized>(rng: &mut R) -> Group {
        Group {
            id: Self::random_id(rng),
            name: Self::random_name(rng),
            privileged: Self::random_privileged(rng),
        }
    }

    fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.gen_range(0..3) {
            0 => GroupsOperation::Insert(Self::random_group(rng)),
            1 => GroupsOperation::Update(Self::random_group(rng)),
            2 => GroupsOperation::Delete(Self::random_id(rng)),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Operation {
    Users(UsersOperation),
    Groups(GroupsOperation),
}

impl Operation {
    fn random<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.gen_range(0..2) {
            0 => Operation::Users(UsersOperation::random(rng)),
            1 => Operation::Groups(GroupsOperation::random(rng)),
            _ => unreachable!(),
        }
    }
}

macro_rules! check {
    ($database:expr) => {
        for (id, group) in $database.groups.iter() {
            assert_eq!(*id, group.id);
            assert_eq!($database.group_by_name.get(&group.name), Some(id));
            assert!($database
                .groups_by_privileged
                .get(&group.privileged)
                .unwrap()
                .contains(id));
        }

        for (name, group) in $database.group_by_name.iter() {
            let group = $database.groups.get(group).unwrap();
            assert_eq!(&group.name, name);
        }

        for (privileged, groups) in $database.groups_by_privileged.iter() {
            for group in groups.iter() {
                let group = $database.groups.get(group).unwrap();
                assert_eq!(&group.privileged, privileged);
            }
        }

        for (group, users) in $database.users_by_group.iter() {
            assert!($database.groups.contains_key(group));
            for user in users.iter() {
                let user = $database.users.get(user).unwrap();
                assert_eq!(&user.group, group);
            }
        }

        for (id, user) in $database.users.iter() {
            assert_eq!(&user.id, id);
            assert_eq!($database.user_by_name.get(&user.name), Some(id));
            assert_eq!($database.user_by_email.get(&user.email), Some(id));
            assert!($database
                .users_by_group
                .get(&user.group)
                .unwrap()
                .contains(id));
            assert!($database.users_by_age.get(&user.age).unwrap().contains(id));
        }

        for (name, user) in $database.user_by_name.iter() {
            let user = $database.users.get(user).unwrap();
            assert_eq!(&user.name, name);
        }

        for (email, user) in $database.user_by_email.iter() {
            let user = $database.users.get(user).unwrap();
            assert_eq!(&user.email, email);
        }

        for (age, users) in $database.users_by_age.iter() {
            for user in users.iter() {
                let user = $database.users.get(user).unwrap();
                assert_eq!(&user.age, age);
            }
        }

        assert!(!$database.users.contains_key(&$database.users_next_id()));
        assert!(!$database.groups.contains_key(&$database.groups_next_id()));
    };
}

macro_rules! apply {
    ($database:expr, $operation:expr) => {
        match $operation {
            Operation::Users(op) => match op {
                UsersOperation::Insert(data) => $database.users_insert(data),
                UsersOperation::Update(data) => {
                    let id = data.id;
                    $database
                        .users_update(data)
                        .map(|prev| assert_eq!(prev.id, id))
                }
                UsersOperation::Delete(id) => $database
                    .users_delete(id)
                    .map(|prev| assert_eq!(prev.id, id)),
            },
            Operation::Groups(op) => match op {
                GroupsOperation::Insert(data) => $database.groups_insert(data),
                GroupsOperation::Update(data) => {
                    let id = data.id;
                    $database
                        .groups_update(data)
                        .map(|prev| assert_eq!(prev.id, id))
                }
                GroupsOperation::Delete(id) => $database
                    .groups_delete(id)
                    .map(|prev| assert_eq!(prev.id, id)),
            },
        }
    };
}

#[test]
fn std_database() {
    let mut rng = thread_rng();
    let mut database = StdDatabase::default();
    let mut errors: BTreeMap<Error, usize> = BTreeMap::new();
    let mut successes = 0;

    let batch_size = 100;
    for _ in (0..100_00).step_by(batch_size) {
        let operations: Vec<Operation> = (0..batch_size)
            .map(|_| Operation::random(&mut rng))
            .collect();
        for operation in operations.into_iter() {
            match apply!(database, operation) {
                Ok(()) => {
                    successes += 1;
                }
                Err(err) => {
                    *errors.entry(err).or_default() += 1;
                }
            }
        }

        check!(database);
    }

    assert!(successes > 0);
    for error in Error::iter() {
        assert!(*errors.get(&error).unwrap() > 0);
    }
}

#[test]
fn im_database() {
    let mut rng = thread_rng();
    let mut database = ImDatabase::default();
    let mut errors: BTreeMap<Error, usize> = BTreeMap::new();
    let mut successes = 0;

    let batch_size = 100;
    for _ in (0..100_00).step_by(batch_size) {
        let operations: Vec<Operation> = (0..batch_size)
            .map(|_| Operation::random(&mut rng))
            .collect();
        for operation in operations.into_iter() {
            match apply!(database, operation) {
                Ok(()) => {
                    successes += 1;
                }
                Err(err) => {
                    *errors.entry(err).or_default() += 1;
                }
            }
        }

        check!(database);
    }

    assert!(successes > 0);
    for error in Error::iter() {
        assert!(*errors.get(&error).unwrap() > 0);
    }
}

#[test]
fn hashbrown_database() {
    let mut rng = thread_rng();
    let mut database = HashBrownDatabase::default();
    let mut errors: BTreeMap<Error, usize> = BTreeMap::new();
    let mut successes = 0;

    let batch_size = 100;
    for _ in (0..100_00).step_by(batch_size) {
        let operations: Vec<Operation> = (0..batch_size)
            .map(|_| Operation::random(&mut rng))
            .collect();
        for operation in operations.into_iter() {
            match apply!(database, operation) {
                Ok(()) => {
                    successes += 1;
                }
                Err(err) => {
                    *errors.entry(err).or_default() += 1;
                }
            }
        }

        check!(database);
    }

    assert!(successes > 0);
    for error in Error::iter() {
        assert!(*errors.get(&error).unwrap() > 0);
    }
}

#[test]
fn avl_database() {
    let mut rng = thread_rng();
    let mut database = AvlDatabase::default();
    let mut errors: BTreeMap<Error, usize> = BTreeMap::new();
    let mut successes = 0;

    let batch_size = 100;
    for _ in (0..100_00).step_by(batch_size) {
        let operations: Vec<Operation> = (0..batch_size)
            .map(|_| Operation::random(&mut rng))
            .collect();
        for operation in operations.into_iter() {
            match apply!(database, operation) {
                Ok(()) => {
                    successes += 1;
                }
                Err(err) => {
                    *errors.entry(err).or_default() += 1;
                }
            }
        }

        check!(database);
    }

    assert!(successes > 0);
    for error in Error::iter() {
        assert!(*errors.get(&error).unwrap() > 0);
    }
}

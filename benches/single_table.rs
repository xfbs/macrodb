use avl::{AvlTreeMap, AvlTreeSet};
use btree_slab::{BTreeMap as BTreeSlabMap, BTreeSet as BTreeSlabSet};
use criterion::*;
use hashbrown::{HashMap as HashMapBrown, HashSet as HashSetBrown};
use im::{HashMap as HashMapIm, OrdMap};
use macrodb::table;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rusqlite::Connection;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::time::Duration;

const RANDOM_SEED: u64 = 12345;
const SQLITE_SCHEMA: &'static str = "
CREATE TABLE users(
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    age INTEGER NOT NULL
);

CREATE UNIQUE INDEX user_by_email ON users(email);
CREATE INDEX users_by_name ON users(name);
CREATE INDEX users_by_age ON users(age);
";

#[derive(Debug, Clone, PartialEq, Eq)]
enum Error {
    UserNotFound,
    UserIdExists,
    UserEmailExists,
}

type UserId = u64;

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

#[derive(Default, Clone)]
struct BTreeDatabase {
    users: BTreeMap<UserId, User>,
    user_by_email: BTreeMap<String, UserId>,
    users_by_age: BTreeMap<u32, BTreeSet<UserId>>,
    users_by_name: BTreeMap<String, BTreeSet<UserId>>,
}

impl BTreeDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        index users_by_age age => (),
        index users_by_name name => ()
    );
}

#[derive(Default, Clone)]
struct BTreeSlabDatabase {
    users: BTreeSlabMap<UserId, User>,
    user_by_email: BTreeSlabMap<String, UserId>,
    users_by_age: BTreeSlabMap<u32, BTreeSlabSet<UserId>>,
    users_by_name: BTreeSlabMap<String, BTreeSlabSet<UserId>>,
}

impl BTreeSlabDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        index users_by_age age => (),
        index users_by_name name => ()
    );
}

#[derive(Default, Clone)]
struct OrdMapDatabase {
    users: OrdMap<UserId, User>,
    user_by_email: OrdMap<String, UserId>,
    users_by_age: OrdMap<u32, BTreeSet<UserId>>,
    users_by_name: OrdMap<String, BTreeSet<UserId>>,
}

impl OrdMapDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        index users_by_age age => (),
        index users_by_name name => ()
    );
}

#[derive(Default, Clone)]
struct HashDatabase {
    users: HashMap<UserId, User>,
    user_by_email: HashMap<String, UserId>,
    users_by_age: HashMap<u32, HashSet<UserId>>,
    users_by_name: HashMap<String, HashSet<UserId>>,
}

impl HashDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        index users_by_age age => (),
        index users_by_name name => ()
    );
}

#[derive(Default, Clone)]
struct HashImDatabase {
    users: HashMapIm<UserId, User>,
    user_by_email: HashMapIm<String, UserId>,
    users_by_age: HashMapIm<u32, HashSet<UserId>>,
    users_by_name: HashMapIm<String, HashSet<UserId>>,
}

impl HashImDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        index users_by_age age => (),
        index users_by_name name => ()
    );
}

#[derive(Default, Clone)]
struct HashBrownDatabase {
    users: HashMapBrown<UserId, User>,
    user_by_email: HashMapBrown<String, UserId>,
    users_by_age: HashMapBrown<u32, HashSetBrown<UserId>>,
    users_by_name: HashMapBrown<String, HashSetBrown<UserId>>,
}

impl HashBrownDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        index users_by_age age => (),
        index users_by_name name => ()
    );
}

#[derive(Default, Clone)]
struct AvlDatabase {
    users: AvlTreeMap<UserId, User>,
    user_by_email: AvlTreeMap<String, UserId>,
    users_by_age: AvlTreeMap<u32, AvlTreeSet<UserId>>,
    users_by_name: AvlTreeMap<String, AvlTreeSet<UserId>>,
}

impl AvlDatabase {
    table!(
        users: User,
        id: UserId,
        missing Error => Error::UserNotFound,
        primary users id => Error::UserIdExists,
        unique user_by_email email => Error::UserEmailExists,
        index users_by_age age => (),
        index users_by_name name => ()
    );
}

fn generate_user(id: u64, num: u64) -> User {
    User {
        id: id,
        email: format!("user-{num}@example.com"),
        name: format!("user-{num}"),
        age: 21 + (num % 50) as u32,
    }
}

fn random_range(limit: u64) -> Vec<u64> {
    let mut rng = ChaCha20Rng::seed_from_u64(RANDOM_SEED);
    let mut indices: Vec<u64> = (0..limit).collect();
    indices.shuffle(&mut rng);
    indices
}

fn generate_users_random(limit: u64) -> Vec<User> {
    random_range(limit)
        .into_iter()
        .enumerate()
        .map(|(id, num)| generate_user(id as u64, num))
        .collect()
}

fn generate_users(limit: u64) -> Vec<User> {
    (0..limit)
        .map(|num| generate_user(num, num))
        .collect::<Vec<User>>()
}

macro_rules! table_benchmark {
    ($c:expr, $name:expr, $database:ty, $operations:expr) => {
        let mut group = $c.benchmark_group($name);
        group.sample_size(10);
        group.measurement_time(Duration::from_secs(10));
        for ops in $operations.into_iter() {
            group.throughput(Throughput::Elements(ops));

            group.bench_with_input(format!("insert-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || generate_users(*elems),
                    |data| {
                        let mut database = <$database>::default();
                        for user in data.into_iter() {
                            database.users_insert(user).unwrap();
                        }
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("random-insert-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || generate_users_random(*elems),
                    |data| {
                        let mut database = <$database>::default();
                        for user in data.into_iter() {
                            database.users_insert(user).unwrap();
                        }
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("update-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let mut database = <$database>::default();
                        for user in users.into_iter() {
                            database.users_insert(user).unwrap();
                        }
                        database
                    },
                    |mut database| {
                        for id in 0..*elems {
                            let mut user = database.users.get(&id).unwrap().clone();
                            user.age = 18 + (id % 53) as u32;
                            user.name = format!("new-{id}");
                            user.email = format!("new-{id}@example.com");
                            database.users_update(user).unwrap();
                        }
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("random-update-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let mut database = <$database>::default();
                        for user in users.into_iter() {
                            database.users_insert(user).unwrap();
                        }
                        let order = random_range(*elems);
                        (database, order)
                    },
                    |(mut database, order)| {
                        for id in order.into_iter() {
                            let mut user = database.users.get(&id).unwrap().clone();
                            user.age = 18 + (id % 53) as u32;
                            user.name = format!("new-{id}");
                            user.email = format!("new-{id}@example.com");
                            database.users_update(user).unwrap();
                        }
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("clone-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let mut database = <$database>::default();
                        for user in users.into_iter() {
                            database.users_insert(user).unwrap();
                        }
                        database
                    },
                    |database| black_box(database.clone()),
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("delete-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let mut database = <$database>::default();
                        for user in users.into_iter() {
                            database.users_insert(user).unwrap();
                        }
                        database
                    },
                    |mut database| {
                        for id in 0..*elems {
                            database.users_delete(id).unwrap();
                        }
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("random-delete-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let mut database = <$database>::default();
                        for user in users.into_iter() {
                            database.users_insert(user).unwrap();
                        }
                        let order = random_range(*elems);
                        (database, order)
                    },
                    |(mut database, order)| {
                        for id in order.into_iter() {
                            database.users_delete(id).unwrap();
                        }
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });
        }

        group.finish();
    };
}

macro_rules! sqlite_benchmark {
    ($c:expr, $name:expr, $operations:expr) => {
        let mut group = $c.benchmark_group($name);
        group.sample_size(10);
        group.measurement_time(Duration::from_secs(10));

        for ops in $operations.into_iter() {
            group.throughput(Throughput::Elements(ops));

            group.bench_with_input(format!("insert-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let database = Connection::open_in_memory().unwrap();
                        database.execute_batch(SQLITE_SCHEMA).unwrap();
                        (database, users)
                    },
                    |(mut database, users)| {
                        let transaction = database.transaction().unwrap();
                        let mut insertion = transaction
                            .prepare("INSERT INTO users(id, name, email, age) VALUES (?, ?, ?, ?)")
                            .unwrap();
                        for user in users.into_iter() {
                            insertion
                                .execute((user.id, &user.name, &user.email, user.age))
                                .unwrap();
                        }
                        drop(insertion);
                        transaction.commit().unwrap();
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("random-insert-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users_random(*elems);
                        let database = Connection::open_in_memory().unwrap();
                        database.execute_batch(SQLITE_SCHEMA).unwrap();
                        (database, users)
                    },
                    |(mut database, users)| {
                        let transaction = database.transaction().unwrap();
                        let mut insertion = transaction
                            .prepare("INSERT INTO users(id, name, email, age) VALUES (?, ?, ?, ?)")
                            .unwrap();
                        for user in users.into_iter() {
                            insertion
                                .execute((user.id, &user.name, &user.email, user.age))
                                .unwrap();
                        }
                        drop(insertion);
                        transaction.commit().unwrap();
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("update-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let mut database = Connection::open_in_memory().unwrap();
                        database.execute_batch(SQLITE_SCHEMA).unwrap();
                        let transaction = database.transaction().unwrap();
                        let mut insertion = transaction
                            .prepare("INSERT INTO users(id, name, email, age) VALUES (?, ?, ?, ?)")
                            .unwrap();
                        for user in users.into_iter() {
                            insertion
                                .execute((user.id, &user.name, &user.email, user.age))
                                .unwrap();
                        }
                        drop(insertion);
                        transaction.commit().unwrap();
                        database
                    },
                    |mut database| {
                        let transaction = database.transaction().unwrap();
                        for id in 0..*elems {
                            let mut lookup = transaction
                                .prepare_cached("SELECT * FROM users WHERE id = ?")
                                .unwrap();
                            let mut result = lookup.query((id,)).unwrap();
                            let row = result.next().unwrap().unwrap();
                            let mut user = User {
                                id: row.get(0).unwrap(),
                                name: row.get(1).unwrap(),
                                email: row.get(2).unwrap(),
                                age: row.get(3).unwrap(),
                            };
                            user.age = 18 + (id % 53) as u32;
                            user.name = format!("new-{id}");
                            user.email = format!("new-{id}@example.com");
                            let mut insertion = transaction
                                .prepare_cached(
                                    "UPDATE users SET name = ?, email = ?, age = ? WHERE id = ?",
                                )
                                .unwrap();
                            insertion
                                .execute((&user.name, &user.email, user.age, user.id))
                                .unwrap();
                        }
                        transaction.commit().unwrap();
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("random-update-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let mut database = Connection::open_in_memory().unwrap();
                        database.execute_batch(SQLITE_SCHEMA).unwrap();
                        let transaction = database.transaction().unwrap();
                        let mut insertion = transaction
                            .prepare("INSERT INTO users(id, name, email, age) VALUES (?, ?, ?, ?)")
                            .unwrap();
                        for user in users.into_iter() {
                            insertion
                                .execute((user.id, &user.name, &user.email, user.age))
                                .unwrap();
                        }
                        drop(insertion);
                        transaction.commit().unwrap();
                        let order = random_range(*elems);
                        (database, order)
                    },
                    |(mut database, order)| {
                        let transaction = database.transaction().unwrap();
                        for id in order.into_iter() {
                            let mut lookup = transaction
                                .prepare_cached("SELECT * FROM users WHERE id = ?")
                                .unwrap();
                            let mut result = lookup.query((id,)).unwrap();
                            let row = result.next().unwrap().unwrap();
                            let mut user = User {
                                id: row.get(0).unwrap(),
                                name: row.get(1).unwrap(),
                                email: row.get(2).unwrap(),
                                age: row.get(3).unwrap(),
                            };
                            user.age = 18 + (id % 53) as u32;
                            user.name = format!("new-{id}");
                            user.email = format!("new-{id}@example.com");
                            let mut insertion = transaction
                                .prepare_cached(
                                    "UPDATE users SET name = ?, email = ?, age = ? WHERE id = ?",
                                )
                                .unwrap();
                            insertion
                                .execute((&user.name, &user.email, user.age, user.id))
                                .unwrap();
                        }
                        transaction.commit().unwrap();
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("delete-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let mut database = Connection::open_in_memory().unwrap();
                        database.execute_batch(SQLITE_SCHEMA).unwrap();
                        let transaction = database.transaction().unwrap();
                        let mut insertion = transaction
                            .prepare("INSERT INTO users(id, name, email, age) VALUES (?, ?, ?, ?)")
                            .unwrap();
                        for user in users.into_iter() {
                            insertion
                                .execute((user.id, &user.name, &user.email, user.age))
                                .unwrap();
                        }
                        drop(insertion);
                        transaction.commit().unwrap();
                        database
                    },
                    |mut database| {
                        let transaction = database.transaction().unwrap();
                        let mut insertion = transaction
                            .prepare("DELETE FROM users WHERE id = ?")
                            .unwrap();
                        for user in 0..*elems {
                            insertion.execute((user as i64,)).unwrap();
                        }
                        drop(insertion);
                        transaction.commit().unwrap();
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });

            group.bench_with_input(format!("random-delete-{ops}"), &ops, |b, elems| {
                b.iter_batched(
                    || {
                        let users = generate_users(*elems);
                        let mut database = Connection::open_in_memory().unwrap();
                        database.execute_batch(SQLITE_SCHEMA).unwrap();
                        let transaction = database.transaction().unwrap();
                        let mut insertion = transaction
                            .prepare("INSERT INTO users(id, name, email, age) VALUES (?, ?, ?, ?)")
                            .unwrap();
                        for user in users.into_iter() {
                            insertion
                                .execute((user.id, &user.name, &user.email, user.age))
                                .unwrap();
                        }
                        drop(insertion);
                        transaction.commit().unwrap();
                        let order = random_range(*elems);
                        (database, order)
                    },
                    |(mut database, order)| {
                        let transaction = database.transaction().unwrap();
                        let mut insertion = transaction
                            .prepare("DELETE FROM users WHERE id = ?")
                            .unwrap();
                        for user in order.into_iter() {
                            insertion.execute((user as i64,)).unwrap();
                        }
                        drop(insertion);
                        transaction.commit().unwrap();
                        black_box(database)
                    },
                    BatchSize::SmallInput,
                )
            });
        }

        group.finish();
    };
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let operations = [100_000];

    table_benchmark!(c, "std::btree", BTreeDatabase, operations);
    table_benchmark!(c, "std::hash", HashDatabase, operations);
    table_benchmark!(c, "slab::btree", BTreeSlabDatabase, operations);
    table_benchmark!(c, "im::btree", OrdMapDatabase, operations);
    table_benchmark!(c, "im::hash", HashImDatabase, operations);
    table_benchmark!(c, "brown::hash", HashBrownDatabase, operations);
    table_benchmark!(c, "avl::tree", AvlDatabase, operations);

    sqlite_benchmark!(c, "sqlite", operations);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

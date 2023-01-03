use btree_slab::{BTreeMap as BTreeSlabMap, BTreeSet as BTreeSlabSet};
use criterion::*;
use macrodb::table;
use std::collections::{BTreeMap, BTreeSet};
use std::collections::{HashMap, HashSet};

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

#[derive(Default)]
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

#[derive(Default)]
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

#[derive(Default)]
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

fn generate_users(limit: usize) -> Vec<User> {
    (0..limit)
        .map(|id| User {
            id: id as u64,
            email: format!("user-{id}@example.com"),
            name: format!("user-{id}"),
            age: 21 + (id % 50) as u32,
        })
        .collect::<Vec<User>>()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let insertions = [100_000];
    let deletions = [100_000];
    let updates = [100_000];

    let mut group = c.benchmark_group("btree");
    for insertions in insertions.into_iter() {
        group.throughput(Throughput::Elements(insertions));
        group.bench_with_input(format!("insert-{insertions}"), &insertions, |b, elems| {
            b.iter_batched(
                || generate_users(*elems as usize),
                |data| {
                    let mut database = BTreeDatabase::default();
                    for user in data.into_iter() {
                        database.users_insert(user).unwrap();
                    }
                    black_box(database)
                },
                BatchSize::SmallInput,
            )
        });
    }
    for updates in updates.into_iter() {
        group.throughput(Throughput::Elements(updates));
        group.bench_with_input(format!("update-{updates}"), &updates, |b, elems| {
            b.iter_batched(
                || {
                    let users = generate_users(*elems as usize);
                    let mut database = BTreeDatabase::default();
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
    }
    for deletions in deletions.into_iter() {
        group.throughput(Throughput::Elements(deletions));
        group.bench_with_input(format!("delete-{deletions}"), &deletions, |b, elems| {
            b.iter_batched(
                || {
                    let users = generate_users(*elems as usize);
                    let mut database = BTreeDatabase::default();
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
    }
    group.finish();

    let mut group = c.benchmark_group("btree-slab");
    for insertions in insertions.into_iter() {
        group.throughput(Throughput::Elements(insertions));
        group.bench_with_input(format!("insert-{insertions}"), &insertions, |b, elems| {
            b.iter_batched(
                || generate_users(*elems as usize),
                |data| {
                    let mut database = BTreeSlabDatabase::default();
                    for user in data.into_iter() {
                        database.users_insert(user).unwrap();
                    }
                    black_box(database)
                },
                BatchSize::SmallInput,
            )
        });
    }
    for updates in updates.into_iter() {
        group.throughput(Throughput::Elements(updates));
        group.bench_with_input(format!("update-{updates}"), &updates, |b, elems| {
            b.iter_batched(
                || {
                    let users = generate_users(*elems as usize);
                    let mut database = BTreeSlabDatabase::default();
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
    }
    for deletions in deletions.into_iter() {
        group.throughput(Throughput::Elements(deletions));
        group.bench_with_input(format!("delete-{deletions}"), &deletions, |b, elems| {
            b.iter_batched(
                || {
                    let users = generate_users(*elems as usize);
                    let mut database = BTreeSlabDatabase::default();
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
    }
    group.finish();

    let mut group = c.benchmark_group("hash");
    for insertions in insertions.into_iter() {
        group.throughput(Throughput::Elements(insertions));
        group.bench_with_input(format!("insert-{insertions}"), &insertions, |b, elems| {
            b.iter_batched(
                || generate_users(*elems as usize),
                |data| {
                    let mut database = HashDatabase::default();
                    for user in data.into_iter() {
                        database.users_insert(user).unwrap();
                    }
                    black_box(database)
                },
                BatchSize::SmallInput,
            )
        });
    }
    for updates in updates.into_iter() {
        group.throughput(Throughput::Elements(updates));
        group.bench_with_input(format!("update-{updates}"), &updates, |b, elems| {
            b.iter_batched(
                || {
                    let users = generate_users(*elems as usize);
                    let mut database = HashDatabase::default();
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
    }
    for deletions in deletions.into_iter() {
        group.throughput(Throughput::Elements(deletions));
        group.bench_with_input(format!("delete-{deletions}"), &deletions, |b, elems| {
            b.iter_batched(
                || {
                    let users = generate_users(*elems as usize);
                    let mut database = HashDatabase::default();
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
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

# MacroDB [![Crate version]][crate] [![Docs badge]][docsrs]

This is a crate that lets you automatically generate code for a type-safe in-memory relational database in Rust. It supports unique and regular indices, foreign key constraints, and generic (function) constraints. It is generic over the data types used to store the indices and records in, it can for example use [HashMap](https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html) or [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) or a large number of other data structures from other crates, see the benchmark section below.

On a high level, the crate works by having you define a database struct that contains the tables and indices, and then invoking the macro to generate appropriate insertion, update and deletion methods per table. The insertion, update and deletion methods work to ensure consistency of the database and update the indices.

There is no support for transactions or concurrency yet, aside from the usual Rust semantics of having either multiple readers or a single writer. It is possible to emulate transactions using the `im` crate, which offers copy-on-write immutable data structures, and a mutex. This allows for semantics similar to that of SQLite, with many concurrent read-only transactions but only a single write transaction.

## Benchmark

The speed of the generated database depends on the data type that you select (BTreeMap or HashMap) and on the number of indices. To get a rough estimate of the speed of the in-memory database, this repository contains benchmarks that compare this crate (using various storage data types) against the popular SQLite database running in-memory. The tests operate on a table with two regular indices and one unique index.

Running the benchmarks on a MacBook Air M2, using one million (1,000,000) rows and randomized inputs (not sequential) for every benchmark, this is the performance. All values are in Kelem/s (thousand rows per second). Higher is better.

| Data types | Insert (Kelem/s) | Update (Kelem/s) | Delete (Kelem/s) | Clone (Kelem/s) |
| --- | --: | --: | --: | --: |
| `hashbrown::{HashMap, HashSet}` | 2,273 | 754 | 790 | 1,286 |
| `std::{HashMap, HashSet}` | 1,188 | 614 | 605 | 1,279 |
| `std::{BTreeMap, BTreeSet}` | 586 | 200 | 312 | 3,933 |
| `im::{HashMap, HashSet}` | 826 | 347 | 442 | +inf |
| `im::{OrdMap, OrdSet}` | 344 | 138 | 248 | +inf |
| `avl::{AvlTreeMap, AvlTreeSet}` | 532 | 162 | 238 | 1,952 |
| `btree_slab::{BTreeMap, BTreeSet}` | 372 | 121 | 216 | 1,821 |
| SQLite | 388 | 138 | 309 | n/a |

This benchmark can be recreated by setting the insertion, update and deletion counts in `benches/single_table.rs` to one million and running `cargo bench`.

Notably, the data types from the `im` crate are copy-on-write and can be cloned cheaply. This is why the clone time is indicated as `+inf`. 

## Example

See `examples/` in this repository.

## License

MIT, see [LICENSE.md]()

[Docs badge]: https://img.shields.io/docsrs/macrodb
[docsrs]: https://docs.rs/macrodb/
[Crate version]: https://img.shields.io/crates/v/macrodb
[crate]: https://crates.io/crates/macrodb


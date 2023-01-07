# MacroDB

This is a crate that lets you automatically generate code for an in-memory relational database in Rust. It supports indexes, including unique indexes, as well as foreign keys. It can use [HashMap](https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html) as well as [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) for the indices.

On a high level, the crate works by having you define a database struct that contains the tables and all of the indices, and then invoking the macro to generate appropriate insertion, update and deletion methods.

There is no support for transactions, concurrency (aside from the usual Rust semantics of having either multiple readers or a single writer) or persistence.

## Benchmark

The speed of the generated database depends on the data type that you select (BTreeMap or HashMap) and on the number of indices. To get a rough estimate of the speed of the in-memory database, this repository contains benchmarks that compare this crate (using various storage data types) against the popular SQLite database running in-memory. The tests operate on a table with two regular indices and one unique index.

Running the benchmarks on a MacBook Air M2, using one million (1,000,000) rows and randomized inputs (not sequential) for every benchmark, this is the performance. Higher is better.

| Data types | Insert | Update | Delete | Clone |
| --- | --: | --: | --: | --: |
| `std::{BTreeMap, BTreeSet}` | 586 | 200 | 312 | 3,933 |
| `std::{HashMap, HashSet}` | 1,188 | 614 | 605 | 1,279 |
| `btree_slab::{BTreeMap, BTreeSet}` | 372 | 121 | 216 | 1,821 |
| `im::{OrdMap, OrdSet}` | 344 | 138 | 248 | +inf |
| `im::{HashMap, HashSet}` | 826 | 347 | 442 | +inf |
| `hashbrown::{HashMap, HashSet}` | 2,273 | 754 | 790 | 1,286 |
| `avl::{AvlTreeMap, AvlTreeSet}` | 532 | 162 | 238 | 1,952 |
| `sqlite` | 388 | 138 | 309 | n/a |

This benchmark can be recreated by setting the insertion, update and deletion counts in `benches/single_table.rs` to one million and running `cargo bench`.

## Example

See `examples/` in this repository.

## License

MIT, see [LICENSE.md]()

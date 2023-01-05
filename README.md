# MacroDB

This is a crate that lets you automatically generate code for an in-memory database in Rust. It supports indexes, including unique indexes, as well as foreign keys. It can use [HashMap](https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html) as well as [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) for the indices.

On a high level, the crate works by having you define a database struct that contains the tables and all of the indices, and then invoking the macro to generate appropriate insertion, update and deletion methods.

There is no support for transactions, concurrency (aside from the usual Rust semantics of having either multiple readers or a single writer) or persistence.

## Benchmarks

The speed of the generated database depends on the data type that you select (BTreeMap or HashMap) and on the number of indices. The repository contains a synthetic benchmark that performs tests on a table with two indices and one unique index. On a MacBook Air M2, it can perform more than a million insertions per second.

| Data types | Operations | Count | Time (ms) | Throughput (Melem/s) |
| --- | --- | --: | --: | --: |
| BTreeMap, BTreeSet | Insert | 1,000,000 | 564.26 | 1.772 |
| BTreeMap, BTreeSet | Update | 1,000,000 | 1,033.50 | 0.967 |
| BTreeMap, BTreeSet | Delete | 1,000,000 | 352.04 | 2.840 |
| HashMap, HashSet | Insert | 1,000,000 | 807.63 | 1.238 |
| HashMap, HashSet | Update | 1,000,000 | 1,319.20 | 0.758 |
| HashMap, HashSet | Delete | 1,000,000 | 789.22 | 1.252 |
| hashbrown HashMap, HashSet | Insert | 1,000,000 | 485.43 | 2.060 |
| hashbrown HashMap, HashSet | Update | 1,000,000 | 981.00 | 1.019 |
| hashbrown HashMap, HashSet | Delete | 1,000,000 | 616.93 | 1.620 |

This benchmark can be recreated by setting the insertion, update and deletion counts in `benches/single_table.rs` to one million and running `cargo bench`.

## Example

See `examples/` in this repository.

## License

MIT, see [LICENSE.md]()

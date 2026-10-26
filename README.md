# kv-store

[![CI](https://github.com/justmeFRUZ/kv-store/actions/workflows/ci.yml/badge.svg)](https://github.com/justmeFRUZ/kv-store/actions/workflows/ci.yml)

A from-scratch key-value store built as a learning project: an in-memory hash
table with chaining and load-factor resizing, plus an LRU cache on top.

## Status

`Store` (chaining hash table with FNV-1a hashing, load-factor 0.75,
double-on-resize), `LruCache` (index-based arena, capacity-asserted > 0), and
`SkipList` (`src/skip.rs`, probabilistic ordered index) are all implemented and
tested. 37 tests pass. Week 12 proofs (amortized O(1) insertion, LRU eviction
correctness) are written up in NOTES.md; a release-profile bench pass with
repeats is deferred.

## Build and run

    cargo build
    cargo run
    cargo test

Lib + binary crate, no dependencies (`Cargo.toml`, edition 2024). `cargo run` prints
benchmark output for `Store::set`/`get`/`delete` at four table sizes, then a
small LRU demo. Types, impls and tests live in `src/lib.rs`; `src/main.rs` is
a thin binary.

## Design

- **`Store`** — `Vec<Vec<(String, String)>>` buckets. FNV-1a hash. Chaining for
  collisions. Resizes when load factor exceeds 0.75: doubles bucket count and
  rehashes.
- **`LruCache`** — index-based arena, not pointer-based. `Vec<LruNode>` holds
  the nodes, `HashMap<String, usize>` maps keys to node indices, `Vec<usize>` is
  a free list of evicted slots. `new` asserts `capacity > 0`.

Full design notes, including why the index-based LRU was chosen over
`Rc<RefCell<Node>>`, are in NOTES.md.

## Skip list

`SkipList` (`src/skip.rs`) is a probabilistic ordered index: a sorted linked
list where each node gets a random height by repeated coin flip (p = 0.5,
capped at `MAX_LEVEL = 24`), giving express lanes above the base list. Search
descends the lanes for expected O(log n); a range query is one search plus a
walk along the base list, O(log n + k). Nodes live in a `Vec` arena with
`next: Vec<Option<usize>>` lanes, so a lane end is `None`, never a magic
index. `Store::set` and `Store::delete` keep the skip list and the hash table
in one code path; `Store::range` walks the skip list for key order and looks
each value up in the hash table.

**Trade-off.** O(log n) here is *expected*, not guaranteed: with unlucky coin
flips every operation degrades toward O(n), which a balanced tree's rotation
logic would rule out. In exchange, there is no rotation to get wrong, the
arena-and-lane representation is the same shape the LRU already uses, and
range queries fall out of walking level 0 with no extra structure. For a
learning store with one writer and a workload that reads ranges, that is the
right side of the trade; for an index where worst-case latency is contractual,
it would not be.

Release profile, single run per size (the two tables above are debug profile —
do not compare across tables). Ratios are the meaningful column; digits are
single-run.

| n         | op     | SkipList  | BTreeMap  | ratio |
|-----------|--------|-----------|-----------|-------|
| 1 000     | insert | 357.8 µs  | 230.8 µs  | 1.55  |
| 10 000    | insert | 3.27 ms   | 3.41 ms   | 0.96  |
| 100 000   | insert | 38.34 ms  | 30.08 ms  | 1.27  |
| 1 000 000 | insert | 392.06 ms | 306.99 ms | 1.28  |
| 1 000     | get    | 151.6 µs  | 80.5 µs   | 1.88  |
| 10 000    | get    | 1.86 ms   | 1.03 ms   | 1.81  |
| 100 000   | get    | 19.46 ms  | 11.65 ms  | 1.67  |
| 1 000 000 | get    | 227.85 ms | 135.10 ms | 1.69  |
| 1 000     | range  | 7.6 µs    | 2.3 µs    | 3.30  |
| 10 000    | range  | 104.6 µs  | 52.9 µs   | 1.98  |
| 100 000   | range  | 1.36 ms   | 511 µs    | 2.66  |
| 1 000 000 | range  | 14.76 ms  | 5.39 ms   | 2.74  |

Insert is near parity (the n=10 000 row, 0.96, is noise — do not read it as
"faster"). Get is ~1.7–1.9× and range ~2.0–3.3× slower than `BTreeMap`,
ratios holding across four orders of magnitude — consistent with cache
behavior and pointer-chasing, not asymptotics, since both sides are
O(log n) / O(log n + k). The n=1 000 rows sit near the timing noise floor and
should not be quoted as precise.

## Benchmarks

Debug profile, single run per size. Sink is accumulated value lengths,
`black_box`'d to prevent elision.

| n       | buckets | set       | get       | delete   |
|---------|---------|-----------|-----------|----------|
| 100     | 256     | 96.6 µs   | 4.6 µs    | 30.9 µs  |
| 1 000   | 2 048   | 751.5 µs  | 46.7 µs   | 112.7 µs |
| 10 000  | 16 384  | 7.30 ms   | 808.3 µs  | 1.34 ms  |
| 100 000 | 262 144 | 101.03 ms | 16.57 ms  | 22.75 ms |

Per-op cost is flat within a small factor across four orders of magnitude —
amortized O(1) visible in real numbers. The ~3.6× rise in `get` per-op from
n=100 to n=100000 is cache residency, not algorithm. Numbers are debug-profile
single-run: shape is trustworthy, digits are not.

### Store vs `std::HashMap`

Same `set`-only workload, both sides. Single-run debug. Numbers from commit
`9806a91`; ratios reproduced across two runs to ~0.05.

| n       | Store    | HashMap  | ratio |
|---------|----------|----------|-------|
| 100     | 80.4 µs  | 53.6 µs  | 1.50  |
| 1 000   | 697.2 µs | 542.1 µs | 1.29  |
| 10 000  | 6.50 ms  | 5.94 ms  | 1.10  |
| 100 000 | 97.87 ms | 55.09 ms | 1.78  |

Store is 1.1–1.9× slower. Not monotonic: dips at n=10 000, rises at
n=100 000, where the run crosses the 131 072→262 144 resize — the largest
single rehash in the sequence — and `bucket_index` pays an integer `%` where
hashbrown masks against power-of-two capacity. The non-monotonicity is the
mechanism showing through, not noise, so it's left unsmoothed.

## Proofs

- **Amortized O(1) insertion** — potential-function argument with
  `Phi = c * max(0, len - B/2)`, `c = 3*kappa`. Shows every operation's
  amortized cost is bounded by `1 + 3*kappa`.
- **LRU eviction correctness** — invariant (ordering + structure + map/list
  consistency) preserved by every operation, proved by induction. `evict`
  removes the tail, which by the ordering invariant is the least recently used.

Both proofs are in NOTES.md, with the code facts they depend on.

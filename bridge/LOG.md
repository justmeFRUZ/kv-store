# bridge/LOG.md

## Week 13 — task (a) pace sizing

- Real average weekly hours, Weeks 9-12: 22
- Formula: 12 + ceil(435 / 22) = 12 + ceil(19.77) = 12 + 20 = 32
- Formula's Stage 2 end week: 32
- Declared Stage 2 end week: 42 (Weekday 3 wording: write the formula's value only if later than 42)

## Week 13 — task (b) outside reviewer

- Status: DEVIATION APPROVED. Using Claude (AI) as the named reviewer.
- Deadline: a named person or channel must exist before Week 16.
- Will approach: Claude (via direct prompting)
- Note: This is a deliberate deviation from the plan requirement for a human reviewer. Risk accepted.

## Week 13 — bridge slot

- Posting scan done: 10 rows in bridge/postings.md. Rows 1-2 real, 3-10 estimated — flagged in the file.
- Postgres 18.6 installed, kvlab database created and confirmed with winpty psql -lqt.
- Next: Week 14 bridge slot — SQL basics on a toy tasks table.

- Next: Week 15 bridge slot — schema design on paper first (tasks, tags, task_tags; PK, FK, NOT NULL, UNIQUE).

## Week 15 — bridge slot — schema design

- Schema designed and applied to kvlab: tasks, tags, task_tags (PK, FK, NOT NULL, UNIQUE, CHECK, CASCADE, RESTRICT). Schema at bridge/schema.sql; verified with \d tasks, \d tags, \d task_tags and \di task_tags_tag_id_idx.
- kvlab's prior tasks table (id/title/done/created_at, 2 rows) backed up to bridge/kvlab_tasks_backup.csv and dropped before the new schema was applied.
- Role discrepancy: Week 13's log recorded psql -lqt succeeding as user; today's cluster only has postgres. All schema work done as postgres. Dedicated-role setup deferred to Week 19 Django slot.
- Next: Week 16 bridge slot — joins and GROUP BY; five queries from English questions.

## Week 16 — bridge slot — joins and GROUP BY

- Five queries in bridge/queries.sql, each answering one English question: tasks-with-tags (7 rows), tasks-per-tag (work 3, study 2, urgent 2), tasks-without-tags (task 6 only), tags-on-more-than-2-tasks (work only), tasks-per-status-and-priority (Done/High 2, Pending/Low 2, Pending/Medium 2). All five matched predictions written before running.
- Seed: bridge/seed.sql — 6 tasks, 3 tags, 7 links, 16 rows. Explicit ids + ON CONFLICT DO NOTHING (idempotent); two setval calls advance the identity sequences to 6 and 3 so Week 19's id-less Django inserts do not collide.
- Known gap, not fixed: no tag currently has zero tasks, so Q2's LEFT JOIN "0" branch is structurally correct but unexercised by data.
- Login note: three scram auth failures this session were input-capture artifacts from read -rs, not a wrong password. The wrapper now prints `captured chars=N` before psql so a silent capture failure is visible in the same output.
- Next: Week 17 bridge slot — indexes and EXPLAIN ANALYZE (100k rows via generate_series).


## Week 17 — bridge slot — indexes and EXPLAIN ANALYZE

- Scratch table `bench_kv` in `kvlab`: 100k rows via `generate_series`, keys `k000001`–`k100000`, fixed-width through `lpad(n::text, 6, '0')`. Fixed-width is load-bearing, not cosmetic: verified before the fill that `'k000001' < 'k100000'` returns true on this cluster's Russian-locale collation, so Postgres text ordering agrees with Rust's byte-lexicographic `String` ordering. Without that check the comparison to the skip list would be quietly meaningless.
- Baseline, no index: point lookup 4.735 ms (`Seq Scan`, `Rows Removed by Filter: 99999`), range of 101 rows 22.856 ms (`Seq Scan`, 99899 removed). Both plans fit in shared buffers (`shared hit=541`).
- After `CREATE INDEX bench_kv_key_idx ON bench_kv (key)`: point 0.057 ms, range 0.078 ms, both `Index Scan using bench_kv_key_idx`. Ratio: ~83x point, ~293x range.
- Honest wrinkle: the indexed runs report `Buffers: shared hit=1 read=2` and `hit=2 read=1` — index pages were read from disk, not cache, because `CREATE INDEX` had just written them. That cuts against the index, so the measured speedup is a floor, not a ceiling. Worth one line rather than rounding off.
- Prediction I got wrong, recorded rather than buried: I expected the 101-row range walk to cost measurably more than the 1-row seek. It did not (0.078 vs 0.057 ms) — the index pages were already cached by then and 101 contiguous index entries are nearly free.
- Comparison to NOTES.md: NOT a speedup comparison. The hash table's ~166 ns/op at 100k (NOTES.md lines 67–72, in-process Rust) vs Postgres's 57 µs round-trip is ~343x, but those are different jobs — SQL parse, planner, transaction bookkeeping and wire framing on one side, a function call on the other. The comparable ratio is *within* Postgres: 4.735 → 0.057 ms.
- Flag for weekly check-in: the plan line says "compare with the skip-list numbers," but NOTES.md line 326 records skip-list *ratios* only (get ~1.8–2.0x vs `BTreeMap`), no absolute ms. That comparison cannot be made literally from disk. Options next week: rerun the skip list recording absolutes, accept ratio-only, or substitute the hash-table absolutes (what was actually done here).
- Files: `bridge/bench_index.sql` (create table), `bench_fill.sql` (100k insert), `bench_verify.sql` (empty/index-count/collation check), `bench_index_create.sql` (index + `\d`), `bench_baseline.sql` (ANALYZE + two EXPLAIN ANALYZE). Scratch table `bench_kv` and its index still exist in `kvlab`.
- Next: Week 18 bridge slot — transactions and normalization (BEGIN/ROLLBACK demonstrating atomicity; 3NF note).


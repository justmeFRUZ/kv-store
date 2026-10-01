# kv-store — plan

Status: skeleton only. `Store` type exists; nothing functional.

What this repo will become (Stage 2):
- In-memory hash table built from scratch — chaining for collisions, resize on load-factor threshold.
- get / set / delete, with tests around collision behavior and resizing.
- Later: WAL + crash recovery (Stage 2C). Not now.

Order of work:
1. Week 8 weekdays: read hashing — load factor, collision resolution, chaining vs open addressing.
2. Week 9 Day A: happy-path set/get, no collisions yet.
3. Week 9 Day B: force collisions, implement handling, write collision tests.
4. Week 9 weekdays: delete.
5. Week 10: resize + rehash, tests, benchmarks.
6. Week 11: LRU cache.
7. Week 12: proofs for resizing amortization and LRU eviction; README.

Deliberately out of scope for now:
- No std HashMap as the backing store — building it is the point.
- No persistence in this repo yet — WAL is a later Stage 2C addition.
- No CLI — this is a library-shaped experiment, not a tool.

Open questions to settle during Week 8 weekdays, not now:
- Hash function choice.
- Initial bucket count and load-factor threshold.
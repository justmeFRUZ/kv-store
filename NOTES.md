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

## Load factor 

Load factor is: **the ratio of entries to buckets — `n / k` — a measure of how full the table actually is relative to its allocated size.**

When it gets high: **each bucket is holding multiple entries on average, so lookups increasingly have to walk a chain (or probe past collisions) instead of landing directly — performance degrades toward O(n), which is exactly why a table resizes once load factor crosses its threshold.**

When it gets low: **most buckets sit empty — lookups are fast with almost no collisions, but memory is being spent on slots that aren't storing anything, which is why some implementations also shrink the table if load factor drops too far.**


## Chaining 

Why chains exist: **because a hash function can map two different keys to the same bucket, and you need somewhere to put both of them without overwriting one — a list per bucket lets a bucket hold any number of collided entries instead of just one.**

What lookup does when a chain has more than one entry: **it can't just grab what's in the bucket and return — it has to walk the list, comparing each entry's key against the one it's looking for, until it finds the match (or reaches the end and concludes the key isn't present).**

One thing chaining makes worse than a collision-free table: **lookup cost stops being a guaranteed single step — it becomes proportional to how many entries landed in that particular bucket, so a table with long chains (high load factor, or a poorly distributed hash function) degrades toward a linear scan instead of staying O(1).**

## Open addressing — my words (Week 8)

How it handles a collision: **instead of stacking collided entries in a list at one bucket, it puts the second entry in a different slot entirely — following a probe sequence (linear, quadratic, or double-hashed) from the original hashed slot until it finds one that's empty.**

What lookup does when the slot it checks holds a different key: **it doesn't stop and conclude "not found" — an occupied slot with a different key just means some other key landed there first, so lookup continues to the next slot in the same probe sequence.**

How you know a key is genuinely absent (vs just not in the slot you tried): **you keep probing until you hit a truly empty slot — since insertion always stops at the first empty slot it finds, a key can never sit past one, so reaching empty during a search is the real "not found" signal. This is also why deletion needs a separate "deleted" marker rather than just clearing the slot to empty — clearing it to empty would wrongly tell later searches to stop early, cutting off access to other keys whose probe sequence runs through that slot.**
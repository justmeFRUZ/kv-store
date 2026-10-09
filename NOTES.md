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

## Chaining vs open addressing

This project uses chaining, because: **it's the simpler structure to get right first — a `Vec` of key-value pairs per bucket means insert, get, and delete are all straightforward operations on a list, with no probe-sequence logic to get subtly wrong. It also tolerates a much higher load factor (1–3) before performance degrades, and degrades gradually rather than falling off a cliff near load factor 1.0 the way open addressing does.**

Open addressing would be the better pick here if: **cache performance mattered more than simplicity — everything lives in one contiguous array instead of scattered list nodes, so at low load factors it can genuinely be faster due to better locality. It'd also make sense if memory overhead per entry mattered a lot, since chaining pays for list/pointer overhead per bucket that open addressing avoids.**

One thing I'd have to solve if I switched to open addressing that chaining doesn't force me to solve: **deletion with tombstones — I'd need a third slot state (not just empty/occupied, but empty/occupied/deleted) so that removing an entry doesn't break the probe sequence for other keys that happen to probe past that slot. Chaining sidesteps this completely since removing from a bucket's list has no ordering dependency on other keys.**

## Benchmarks — Week 10 (set / get / delete)

Debug profile, single run per size (`cargo build`, run from `/tmp`). Same four sizes for all three benches. Sink = accumulated value lengths, `black_box`'d to prevent elision.

| n       | buckets | set       | get       | delete   |
|---------|---------|-----------|-----------|----------|
| 100     | 256     | 96.6 µs   | 4.6 µs    | 30.9 µs  |
| 1 000   | 2 048   | 751.5 µs  | 46.7 µs   | 112.7 µs |
| 10 000  | 16 384  | 7.30 ms   | 808.3 µs  | 1.34 ms  |
| 100 000 | 262 144 | 101.03 ms | 16.57 ms  | 22.75 ms |

Per-op: get 46 → 166 ns/op, set 730–1010 ns/op, delete 113–309 ns/op. Flat within a small factor across four orders of magnitude — amortized O(1) visible in real numbers, not just Big-O shorthand.

get's per-op rises ~3.6× from n=100 to n=100000. Cache residency, not algorithm: at n=100 the whole table fits in L1/L2, at n=100000 each lookup is a cold memory access.

Caveats: debug profile (release would be several× faster; debug chosen to match the Weekday 1 set table). Single run per size — shape is trustworthy, digits are not.

## LRU cache — Week 11 design decision

Representation: **index-based, not pointer-based.** Nodes live in a `Vec<Node>`, where `Node { key, value, prev: Option<usize>, next: Option<usize> }`. "Pointers" are indices into that Vec. Map is `HashMap<String, usize>` — key → node index.

Why not `Rc<RefCell<Node>>`: **the plan warns that linked-list-in-Rust with mutable shared references is a known pain point, and explicitly blesses a Vec-based or index-based sidestep. The index approach has no borrow-checker fight, no `Rc`/`Weak` cycle-leak risk, and the same O(1) get/put/evict as the pointer version.**


# Amortized cost of `Store::set` (resizing hash table, chaining)

Code facts: `set` on an existing key updates in place (len unchanged, no grow).
On a new key it pushes, does `len += 1`, then calls `grow` if `len / B > 0.75`,
i.e. when `len = 0.75·B + 1`. `grow` doubles B. Never shrinks.
Assumes B starts as a power of two >= 4 (code uses 16), so B is always a power
of two and 0.75·B is an integer. Initial len = 0.

## 1. Cost of a cheap `set` (new key, no grow)
O(1) expected: hash the key bytes, index a bucket, scan it, push one entry, bump
`len`, one load-factor check. Real work: two `to_string` allocations + hashing.
(Bucket scan is O(1) only in expectation, under roughly uniform hashing. See caveat A.)

## 2. Cost of a `set` that triggers `grow`
O(len), and at the trigger B < (4/3)·len, so equivalently O(B).
- `vec![Vec::new(); 2B]`: ONE heap allocation (the outer buffer). Each inner
  `Vec::new()` has zero capacity, so no per-bucket allocation. Writing the 2B
  empty Vec headers still costs O(B).
- Rehash: every existing entry is re-hashed and pushed into the new table. The
  Strings are moved, not copied. Each NON-EMPTY new bucket allocates once, on
  its first push ("one per non-empty bucket", not "one per bucket").
- The old buckets are dropped afterward.

## 3. How many `set`s between resizes
gap = 0.75·B, where B is the bucket count before the resize that opens the interval.
Check from 16: resizes fire at len 13, 25, 49, 97. Gaps 12, 24, 48 = 0.75 x 16, 32, 64.
Only new-key inserts count. Updates and deletes do not advance toward the next resize.
Each resize rehashes about 0.75·B entries, so the work per insert in the gap is O(1).

## 4. Is the expensive call rare?
Yes. Resize cost is proportional to the gap that earned it, and gaps double, so
total resize work over n inserts is a geometric series, at most about 2n
(aggregate argument). The potential method below gives the same bound and
handles mixed insert/update/delete sequences directly.

## 5. Potential-function proof
Let kappa be a constant with true cost of `grow` <= kappa·len (len after the push).
Any non-growing operation has true cost <= 1.

**Phi = c · max(0, len − B/2), with c = 3·kappa.**

Why this shape:
- Phi >= 0 by the clamp, and Phi(initial) = 0 since len = 0.
- Phi = 0 right after every resize: new B' = 2B, len = 0.75·B + 1 <= B'/2.
- Phi rises by c per new-key insert once len > B/2, reaching c·B/4 = 0.75·kappa·B
  just before the trigger, enough to pay for a rehash costing about 0.75·kappa·B.

Amortized cost c_hat = true cost + delta Phi, per operation:
- Insert, no grow: <= 1 + c = 1 + 3·kappa.
- Update existing key: <= 1 (delta Phi = 0).
- Delete: <= 1 (delta Phi <= 0, since len decreases).
- Insert that triggers grow: len before = 0.75·B.
  delta Phi = −c·B/4. True cost <= 1 + kappa·(0.75·B + 1).
  c_hat <= 1 + kappa + (0.75·kappa − c/4)·B = 1 + kappa  (B-term vanishes for c = 3·kappa).

Every operation, whatever its type, has c_hat <= C = 1 + 3·kappa, bounded
independently of the sequence it appears in. Summing over any n operations:
total true cost <= C·n + Phi(initial) − Phi(final) <= C·n,
since the Phi terms telescope, Phi(initial) = 0 and Phi(final) >= 0.
So amortized O(1) holds for ANY interleaving of insert/update/delete.
Sanity check, c = 2·kappa: the B-coefficient becomes +0.25·kappa, so the grow
insert costs Theta(B) amortized and the bound fails. c >= 3·kappa is required.

## Caveats
A. The proof bounds resize cost only. Per-op O(1) also needs short buckets
   (uniform hashing). Colliding or adversarial keys degrade `get`/`set`/`delete`
   to O(n) regardless of resizing.
B. A single `set` can still spike O(B). Amortized is not worst-case latency.
C. If shrink-on-delete is ever added, this Phi no longer works. It would need a
   symmetric low-load term (e.g. add c·max(0, B/4 − len)) and separated
   thresholds (e.g. grow at 0.75, shrink at 0.25) to avoid thrashing.
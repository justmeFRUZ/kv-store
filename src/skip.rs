/// A deterministic xorshift64 pseudorandom number generator.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Creates a generator seeded with `seed`. A `seed` of 0 is remapped to a
    /// fixed non-zero constant so the generator never gets stuck at 0.
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
        Self { state }
    }

    /// Advances the generator and returns the next 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

const MAX_LEVEL: usize = 24;

fn random_level(rng: &mut Rng) -> usize {
    let mut lvl = 1;
    while lvl < MAX_LEVEL && rng.next_u64() & 1 == 1 {
        lvl += 1;
    }
    lvl
}

#[derive(Debug, Clone)]
struct Node {
    key: String,
    value: String,
    next: Vec<Option<usize>>,
}

/// A probabilistic ordered map backed by a skip list.
///
/// Nodes are stored in an arena (`Vec<Node>`) and linked by index. The head
/// sentinel lives at index 0. Expected `O(log n)` search, insert and remove;
/// worst case is `O(n)`.
pub struct SkipList {
    nodes: Vec<Node>,
    head: usize,
    level: usize,
    rng: Rng,
    len: usize,
}

impl SkipList {
    /// Creates an empty skip list with the given RNG seed.
    ///
    /// The seed is passed through to [`Rng::new`]; two lists built with the
    /// same seed and the same operation sequence produce identical structure.
    pub fn new(seed: u64) -> Self {
        let head = Node {
            key: String::new(),
            value: String::new(),
            next: vec![None; MAX_LEVEL],
        };
        Self {
            nodes: vec![head],
            head: 0,
            level: 1,
            rng: Rng::new(seed),
            len: 0,
        }
    }

    /// Returns the number of entries in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the list contains no entries.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterates over entries in ascending key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        SkipListIter {
            list: self,
            cur: self.nodes[self.head].next[0],
        }
    }

    fn search(&self, key: &str) -> Option<usize> {
        let mut x = self.head;
        for i in (0..self.level).rev() {
            loop {
                match self.nodes[x].next[i] {
                    Some(n) if self.nodes[n].key.as_str() < key => x = n,
                    _ => break,
                }
            }
        }

        match self.nodes[x].next[0] {
            Some(n) if self.nodes[n].key == key => Some(n),
            _ => None,
        }
    }

    /// Returns the value associated with `key`, or `None` if absent.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.search(key).map(|i| self.nodes[i].value.as_str())
    }

    fn lower_bound(&self, key: &str) -> Option<usize> {
        let mut x = self.head;
        for i in (0..self.level).rev() {
            loop {
                match self.nodes[x].next[i] {
                    Some(n) if self.nodes[n].key.as_str() < key => x = n,
                    _ => break,
                }
            }
        }
        self.nodes[x].next[0]
    }

    /// Iterates over entries with `start <= key < end`, in ascending key order.
    ///
    /// The interval is half-open: `end` is excluded. If `start >= end`, the
    /// iterator yields nothing. The bounds are compared lexicographically as
    /// Rust `&str` (`String` `Ord`).
    pub fn range(&self, start: &str, end: &str) -> impl Iterator<Item = (&str, &str)> + '_ {
        SkipListRangeIter {
            list: self,
            cur: self.lower_bound(start),
            end: end.to_string(),
        }
    }

    /// Inserts `key` with `value`, or replaces the existing value if `key` is
    /// already present (in which case `len` is unchanged).
    pub fn insert(&mut self, key: &str, value: &str) {
        let mut update = [self.head; MAX_LEVEL];
        let mut x = self.head;
        for i in (0..self.level).rev() {
            loop {
                match self.nodes[x].next[i] {
                    Some(n) if self.nodes[n].key.as_str() < key => x = n,
                    _ => break,
                }
            }
            update[i] = x;
        }
        let existing = match self.nodes[x].next[0] {
            Some(n) if self.nodes[n].key == key => Some(n),
            _ => None,
        };
        if let Some(i) = existing {
            self.nodes[i].value = value.to_string();
            return;
        }
        let lvl = random_level(&mut self.rng);
        if lvl > self.level {
            for slot in update.iter_mut().take(lvl).skip(self.level) {
                *slot = self.head;
            }
            self.level = lvl;
        }
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            key: key.to_string(),
            value: value.to_string(),
            next: vec![None; lvl],
        });
        for (i, &prev) in update.iter().enumerate().take(lvl) {
            let next_of_prev = self.nodes[prev].next[i];
            self.nodes[new_idx].next[i] = next_of_prev;
            self.nodes[prev].next[i] = Some(new_idx);
        }
        self.len += 1;
    }

    /// Removes `key` and returns its previous value, or `None` if `key` is
    /// absent.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        let mut update = [self.head; MAX_LEVEL];
        let mut x = self.head;
        for i in (0..self.level).rev() {
            loop {
                match self.nodes[x].next[i] {
                    Some(n) if self.nodes[n].key.as_str() < key => x = n,
                    _ => break,
                }
            }
            update[i] = x;
        }
        let target = match self.nodes[x].next[0] {
            Some(n) if self.nodes[n].key == key => n,
            _ => return None,
        };

        let value = self.nodes[target].value.clone();
        for (i, &prev) in update.iter().enumerate().take(self.level) {
            if self.nodes[prev].next[i] == Some(target) {
                let next_of_target = self.nodes[target].next[i];
                self.nodes[prev].next[i] = next_of_target;
            }
        }
        self.len -= 1;
        Some(value)
    }

    #[cfg(test)]
    fn check_invariants(&self) {
        for i in 0..MAX_LEVEL {
            let mut cur = self.nodes[self.head].next[i];
            let mut prev: Option<&str> = None;
            while let Some(n) = cur {
                let k = self.nodes[n].key.as_str();
                if let Some(p) = prev {
                    assert!(p < k, "level {i} unsorted: {p:?} >= {k:?}");
                }
                prev = Some(k);
                cur = self.nodes[n].next[i];
            }
        }

        for i in 0..MAX_LEVEL - 1 {
            let hi = i + 1;
            let mut at_i = vec![false; self.nodes.len()];
            let mut cur = self.nodes[self.head].next[i];
            while let Some(n) = cur {
                at_i[n] = true;
                cur = self.nodes[n].next[i];
            }
            let mut cur = self.nodes[self.head].next[hi];
            while let Some(n) = cur {
                assert!(at_i[n], "node {n} at level {hi} missing at level {i}");
                cur = self.nodes[n].next[hi];
            }
        }

        // 3. the number of nodes reachable at level 0 matches len
        let mut count = 0usize;
        let mut cur = self.nodes[self.head].next[0];
        while let Some(n) = cur {
            count += 1;
            cur = self.nodes[n].next[0];
        }
        assert_eq!(count, self.len, "level-0 node count != len");
    }
}

/// Borrowing iterator over a [`SkipList`] in ascending key order.
///
/// Created by [`SkipList::iter`].
pub struct SkipListIter<'a> {
    list: &'a SkipList,
    cur: Option<usize>,
}

impl<'a> Iterator for SkipListIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.cur?;
        let next = self.list.nodes[i].next[0];
        self.cur = next;
        Some((&self.list.nodes[i].key, &self.list.nodes[i].value))
    }
}

/// Borrowing iterator over a half-open key range of a [`SkipList`].
///
/// Created by [`SkipList::range`]; yields entries with `start <= key < end`.
pub struct SkipListRangeIter<'a> {
    list: &'a SkipList,
    cur: Option<usize>,
    end: String,
}

impl<'a> Iterator for SkipListRangeIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.cur?;
        if self.list.nodes[i].key.as_str() >= self.end.as_str() {
            return None;
        }
        let next = self.list.nodes[i].next[0];
        self.cur = next;
        Some((&self.list.nodes[i].key, &self.list.nodes[i].value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xorshift64_is_deterministic_for_a_fixed_seed() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..8 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn xorshift64_differs_accross_seeds() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn zero_seed_is_remapped_to_a_usable_state() {
        let mut r = Rng::new(0);
        assert_ne!(r.next_u64(), 0);
    }

    #[test]
    fn random_level_is_bounded() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            let lvl = random_level(&mut rng);
            assert!(
                (1..=MAX_LEVEL).contains(&lvl),
                "level {} out of bounds",
                lvl
            )
        }
    }

    #[test]
    fn random_level_distribution_halves_per_level() {
        let mut rng = Rng::new(12345);
        let trials = 10_000;
        let mut counts = [0usize; MAX_LEVEL + 1];
        for _ in 0..trials {
            let lvl = random_level(&mut rng);
            counts[lvl] += 1;
        }
        assert!(
            (4000..=6000).contains(&counts[1]),
            "level 1 count {} outside tolerance",
            counts[1]
        );
        assert!(
            (2000..=3000).contains(&counts[2]),
            "level 2 count {} outside tolerance",
            counts[2]
        );
        assert!(
            (800..=1700).contains(&counts[3]),
            "level 3 count {} outside tolerance",
            counts[3]
        );
    }

    #[test]
    fn skiplist_empty_returns_none_and_is_empty() {
        let sl = SkipList::new(1);
        assert!(sl.is_empty());
        assert_eq!(sl.len(), 0);
        assert_eq!(sl.get("anything"), None);
        assert!(sl.iter().next().is_none());
    }

    #[test]
    fn skiplist_one_element_gets_and_iterates() {
        let mut sl = SkipList::new(1);
        sl.insert("only", "one");
        assert_eq!(sl.len(), 1);
        assert!(!sl.is_empty());
        assert_eq!(sl.get("only"), Some("one"));
        assert_eq!(sl.get("absent"), None);
        let mut it = sl.iter();
        assert_eq!(it.next(), Some(("only", "one")));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn skiplist_insert_at_max_level_keeps_state_consistent() {
        // White-box: force level to MAX_LEVEL so insert's growth branch cannot
        // fire (lvl > self.level is impossible when self.level == MAX_LEVEL).
        // Proves the boundary state stays consistent; not that the fixup runs.
        let mut sl = SkipList::new(1);
        sl.level = MAX_LEVEL;
        for k in ["a", "b", "c"] {
            sl.insert(k, "x");
        }
        assert_eq!(sl.len(), 3);
        assert_eq!(sl.level, MAX_LEVEL);
        assert_eq!(sl.get("b"), Some("x"));
        let got: Vec<&str> = sl.iter().map(|(k, _)| k).collect();
        assert_eq!(got, vec!["a", "b", "c"]);
    }

    #[test]
    fn skiplist_insert_then_get_returns_value() {
        let mut sl = SkipList::new(1);
        sl.insert("name", "Alice");
        assert_eq!(sl.get("name"), Some("Alice"));
    }

    #[test]
    fn skiplist_get_missing_return_none() {
        let mut sl = SkipList::new(1);
        sl.insert("name", "Alice");
        assert_eq!(sl.get("aaa"), None);
        assert_eq!(sl.get("zzz"), None);
    }

    #[test]
    fn skiplist_insert_duplicate_replaces_value() {
        let mut sl = SkipList::new(1);
        sl.insert("name", "Alice");
        sl.insert("name", "Bob");
        assert_eq!(sl.get("name"), Some("Bob"));
        assert_eq!(sl.len(), 1);
    }

    #[test]
    fn skiplist_len_tracks_insert() {
        let mut sl = SkipList::new(1);
        assert!(sl.is_empty());
        sl.insert("a", "1");
        sl.insert("b", "2");
        sl.insert("c", "3");
        assert_eq!(sl.len(), 3);
        assert!(!sl.is_empty());
    }

    #[test]
    fn skiplist_matches_btreemap_on_random_ops() {
        use std::collections::BTreeMap;

        let seed: u64 = 99;
        let mut rng = Rng::new(seed);
        let mut sl = SkipList::new(seed);
        let mut oracle: BTreeMap<String, String> = BTreeMap::new();

        for step in 0..10_00u64 {
            let key = format!("key{}", rng.next_u64() % 100);
            if rng.next_u64().is_multiple_of(2) {
                let value = format!("value{}", step);
                sl.insert(&key, &value);
                oracle.insert(key, value);
            } else {
                let got = sl.get(&key).map(|s| s.to_string());
                let want = oracle.get(&key).cloned();
                assert_eq!(
                    got, want,
                    "seed={} step={} key={}: skiplist vs BTreeMap",
                    seed, step, key
                );
            }
            sl.check_invariants();
        }
        assert_eq!(sl.len(), oracle.len(), "len mismatch, seed={}", seed);
    }

    #[test]
    fn skiplist_iter_returns_keys_in_sorted_order() {
        let mut sl = SkipList::new(1);
        for k in ["m", "a", "z", "c"] {
            sl.insert(k, "x");
        }
        let got: Vec<&str> = sl.iter().map(|(k, _)| k).collect();
        assert_eq!(got, vec!["a", "c", "m", "z"]);
    }

    #[test]
    fn skiplist_iter_matches_btreemap_on_random_ops() {
        use std::collections::BTreeMap;
        let seed = 4242;
        let mut rng = Rng::new(seed);
        let mut sl = SkipList::new(seed);
        let mut bt = BTreeMap::new();
        for _ in 0..10_000 {
            let key = format!("key{:03}", rng.next_u64() % 100);
            sl.insert(&key, "x");
            bt.insert(key, String::from("x"));
        }
        let sl_keys: Vec<&str> = sl.iter().map(|(k, _)| k).collect();
        let bt_keys: Vec<&str> = bt.keys().map(|k| k.as_str()).collect();
        assert_eq!(sl_keys, bt_keys, "seed {seed}");
    }

    #[test]
    fn skiplist_remove_absent_returns_none() {
        let mut sl = SkipList::new(99);
        sl.insert("a", "1");
        assert_eq!(sl.remove("zzz"), None);
        assert_eq!(sl.len(), 1);
        assert_eq!(sl.get("a"), Some("1"));
    }

    #[test]
    fn skiplist_remove_only_element_leaves_empty() {
        let mut sl = SkipList::new(99);
        sl.insert("a", "1");
        assert_eq!(sl.remove("a"), Some(String::from("1")));
        assert!(sl.is_empty());
        assert_eq!(sl.get("a"), None);
        assert_eq!(sl.iter().count(), 0);
    }

    #[test]
    fn skiplist_remove_then_reinsert_works() {
        let mut sl = SkipList::new(99);
        sl.insert("a", "1");
        assert_eq!(sl.remove("a"), Some(String::from("1")));
        sl.insert("a", "2");
        assert_eq!(sl.get("a"), Some("2"));
        assert_eq!(sl.len(), 1);
    }

    #[test]
    fn skiplist_range_is_half_open() {
        let mut sl = SkipList::new(99);
        sl.insert("a", "1");
        sl.insert("b", "2");
        sl.insert("c", "3");
        sl.insert("d", "4");
        let got: Vec<(&str, &str)> = sl.range("b", "d").collect();
        assert_eq!(got, vec![("b", "2"), ("c", "3")]);
    }

    #[test]
    fn skiplist_range_start_ge_end_is_empty() {
        let mut sl = SkipList::new(99);
        sl.insert("a", "1");
        sl.insert("b", "2");
        assert_eq!(sl.range("b", "a").count(), 0);
        assert_eq!(sl.range("a", "a").count(), 0);
    }

    #[test]
    fn skiplist_range_on_empty_is_empty() {
        let sl = SkipList::new(99);
        assert_eq!(sl.range("a", "z").count(), 0);
    }

    #[test]
    fn skiplist_range_matches_btreemap_on_random_ops() {
        use std::collections::BTreeMap;

        let seed: u64 = 4242;
        let mut rng = Rng::new(seed);
        let mut sl = SkipList::new(seed);
        let mut oracle: BTreeMap<String, String> = BTreeMap::new();

        for step in 0..10_000u64 {
            let key = format!("key{}", rng.next_u64() % 100);
            match rng.next_u64() % 3 {
                0 => {
                    let value = format!("value{}", step);
                    sl.insert(&key, &value);
                    oracle.insert(key, value);
                }
                1 => {
                    let got = sl.remove(&key);
                    let want = oracle.remove(&key);
                    assert_eq!(got, want, "seed={} step={} remove key={}", seed, step, key);
                }
                _ => {
                    let start = format!("key{}", rng.next_u64() % 100);
                    let end = format!("key{}", rng.next_u64() % 100);
                    let got: Vec<(&str, &str)> = sl.range(&start, &end).collect();
                    if start >= end {
                        assert!(
                            got.is_empty(),
                            "seed={} step={} start={} end={}",
                            seed,
                            step,
                            start,
                            end
                        );
                    } else {
                        let want: Vec<(&str, &str)> = oracle
                            .range(start.clone()..end.clone())
                            .map(|(k, v)| (k.as_str(), v.as_str()))
                            .collect();
                        assert_eq!(
                            got, want,
                            "seed={} step={} start={} end={}",
                            seed, step, start, end
                        )
                    }
                }
            }
            assert_eq!(
                sl.len(),
                oracle.len(),
                "len mismatch seed={} step={}",
                seed,
                step
            );
        }
    }
}

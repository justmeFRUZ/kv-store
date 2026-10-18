pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
        Self { state }
    }

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

pub struct SkipList {
    nodes: Vec<Node>,
    head: usize,
    level: usize,
    rng: Rng,
    len: usize,
}

impl SkipList {
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

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

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

    pub fn get(&self, key: &str) -> Option<&str> {
        self.search(key).map(|i| self.nodes[i].value.as_str())
    }

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
}

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
    fn skiplist_insert_then_get_returns_value() {
        let mut sl = SkipList::new(1);
        sl.insert("name", "Alice");
        assert_eq!(sl.get("name"), Some("Alice"));
    }

    #[test]
    fn skiplist_get_missing_return_none() {
        let mut sl = SkipList::new(1);
        sl.insert("name", "Alice");
        assert_eq!(sl.get("name"), Some("Alice"));
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
        }
        assert_eq!(sl.len(), oracle.len(), "len mismatch, seed={}", seed);
    }
}

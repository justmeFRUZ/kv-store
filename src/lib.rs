use std::collections::HashMap;

pub mod skip;

pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
pub struct Store {
    buckets: Vec<Vec<(String, String)>>,
    len: usize,
    index: skip::SkipList,
}

impl Store {
    pub fn new() -> Self {
        Self {
            buckets: vec![Vec::new(); 16],
            len: 0,
            index: skip::SkipList::new(4242),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    fn bucket_index(&self, key: &str) -> usize {
        (fnv1a(key.as_bytes()) % self.buckets.len() as u64) as usize
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let idx = self.bucket_index(key);
        let bucket = &mut self.buckets[idx];
        for entry in bucket.iter_mut() {
            if entry.0 == key {
                entry.1 = value.to_string();
                return;
            }
        }
        bucket.push((key.to_string(), value.to_string()));
        self.len += 1;
        self.index.insert(key, key);
        if self.load_factor() > 0.75 {
            self.grow();
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        let idx = self.bucket_index(key);
        let bucket = &self.buckets[idx];
        for entry in bucket.iter() {
            if entry.0 == key {
                return Some(&entry.1);
            }
        }
        None
    }

    pub fn delete(&mut self, key: &str) -> Option<String> {
        let idx = self.bucket_index(key);
        let bucket = &mut self.buckets[idx];
        let pos = bucket.iter().position(|entry| entry.0 == key)?;
        let (_, value) = bucket.remove(pos);
        self.len -= 1;
        let _ = self.index.remove(key);
        Some(value)
    }

    pub fn range(&self, start: &str, end: &str) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.index
            .range(start, end)
            .filter_map(|(key, _)| self.get(key).map(|value| (key, value)))
    }

    pub fn load_factor(&self) -> f64 {
        self.len as f64 / self.buckets.len() as f64
    }

    fn grow(&mut self) {
        let new_len = self.buckets.len() * 2;
        let old = std::mem::replace(&mut self.buckets, vec![Vec::new(); new_len]);
        for bucket in old {
            for (key, value) in bucket {
                let idx = self.bucket_index(&key);
                self.buckets[idx].push((key, value));
            }
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

struct LruNode {
    key: String,
    value: String,
    prev: Option<usize>,
    next: Option<usize>,
}

pub struct LruCache {
    nodes: Vec<LruNode>,
    free: Vec<usize>,
    map: HashMap<String, usize>,
    head: Option<usize>,
    tail: Option<usize>,
    capacity: usize,
}

impl LruCache {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "LruCache capacity must be > 0");
        LruCache {
            nodes: Vec::new(),
            free: Vec::new(),
            map: HashMap::new(),
            head: None,
            tail: None,
            capacity,
        }
    }

    fn unlink(&mut self, idx: usize) {
        let prev = self.nodes[idx].prev;
        let next = self.nodes[idx].next;
        match prev {
            Some(p) => self.nodes[p].next = next,
            None => self.head = next,
        }
        match next {
            Some(n) => self.nodes[n].prev = prev,
            None => self.tail = prev,
        }
    }

    fn push_front(&mut self, idx: usize) {
        self.nodes[idx].prev = None;
        self.nodes[idx].next = self.head;
        match self.head {
            Some(h) => self.nodes[h].prev = Some(idx),
            None => self.tail = Some(idx),
        }
        self.head = Some(idx);
    }

    fn move_to_front(&mut self, idx: usize) {
        if self.head == Some(idx) {
            return;
        }
        self.unlink(idx);
        self.push_front(idx);
    }

    fn alloc(&mut self, key: &str, value: &str) -> usize {
        let node = LruNode {
            key: key.to_string(),
            value: value.to_string(),
            prev: None,
            next: None,
        };
        if let Some(i) = self.free.pop() {
            self.nodes[i] = node;
            i
        } else {
            self.nodes.push(node);
            self.nodes.len() - 1
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&str> {
        let idx = *self.map.get(key)?;
        self.move_to_front(idx);
        Some(&self.nodes[idx].value)
    }

    pub fn put(&mut self, key: &str, value: &str) {
        if let Some(&idx) = self.map.get(key) {
            self.nodes[idx].value = value.to_string();
            self.move_to_front(idx);
        } else {
            let idx = self.alloc(key, value);
            self.map.insert(key.to_string(), idx);
            self.push_front(idx);
            if self.map.len() > self.capacity {
                self.evict();
            }
        }
    }

    fn evict(&mut self) {
        if let Some(t) = self.tail {
            self.unlink(t);
            let key = self.nodes[t].key.clone();
            self.map.remove(&key);
            self.free.push(t);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn set_then_get_returns_value() {
        let mut store = Store {
            buckets: vec![Vec::new(); 16],
            len: 0,
            index: skip::SkipList::new(4242),
        };
        store.set("name", "Alice");
        assert_eq!(store.get("name"), Some("Alice"));
    }

    #[test]
    fn two_keys_in_same_bucket_are_both_retrievable() {
        let mut store = Store {
            buckets: vec![Vec::new(); 1],
            len: 0,
            index: skip::SkipList::new(4242),
        };
        store.set("name", "Alice");
        store.set("city", "Tashkent");
        assert_eq!(store.get("name"), Some("Alice"));
        assert_eq!(store.get("city"), Some("Tashkent"));
    }

    #[test]
    fn updating_a_key_in_a_bucket_replaces_the_value() {
        let mut store = Store {
            buckets: vec![Vec::new(); 1],
            len: 0,
            index: skip::SkipList::new(4242),
        };
        store.set("name", "Alice");
        store.set("name", "Bob");
        store.set("city", "Tashkent");
        assert_eq!(store.get("name"), Some("Bob"));
        assert_eq!(store.get("city"), Some("Tashkent"));
    }

    #[test]
    fn delete_nonexistent_key_returns_none() {
        let mut store = Store {
            buckets: vec![Vec::new(); 16],
            len: 0,
            index: skip::SkipList::new(4242),
        };
        store.set("name", "Alice");
        assert_eq!(store.delete("missing"), None);
        assert_eq!(store.get("name"), Some("Alice"));
    }

    #[test]
    fn delete_from_empty_bucket_returns_none() {
        let mut store = Store {
            buckets: vec![Vec::new(); 16],
            len: 0,
            index: skip::SkipList::new(4242),
        };
        assert_eq!(store.delete("anything"), None);
    }

    #[test]
    fn delete_only_item_in_bucket_leaves_bucket_empty() {
        let mut store = Store {
            buckets: vec![Vec::new(); 1],
            len: 0,
            index: skip::SkipList::new(4242),
        };
        store.set("name", "Alice");
        assert_eq!(store.delete("name"), Some("Alice".to_string()));
        assert_eq!(store.len, 0);
        assert_eq!(store.get("name"), None);
    }

    #[test]
    fn resize_preserves_all_keys() {
        let mut store = Store {
            buckets: vec![Vec::new(); 16],
            len: 0,
            index: skip::SkipList::new(4242),
        };
        for i in 0..20 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            store.set(&key, &value);
        }
        assert_eq!(store.len, 20);
        assert!(store.buckets.len() > 16, "resize did not fire");
        for i in 0..20 {
            let key = format!("key{}", i);
            let expected = format!("value{}", i);
            assert_eq!(store.get(&key), Some(expected.as_str()));
        }
    }

    #[test]
    fn lru_evicts_least_recently_used() {
        let mut lru = LruCache::new(2);
        lru.put("a", "1");
        lru.put("b", "2");
        assert_eq!(lru.get("a"), Some("1"));
        lru.put("c", "3");
        assert_eq!(lru.get("b"), None);
        assert_eq!(lru.get("a"), Some("1"));
        assert_eq!(lru.get("c"), Some("3"));
    }

    #[test]
    fn lru_get_on_middle_node_rewires_neighbors() {
        let mut lru = LruCache::new(3);
        lru.put("a", "1");
        lru.put("b", "2");
        lru.put("c", "3");
        assert_eq!(lru.get("b"), Some("2"));
        lru.put("d", "4");
        assert_eq!(lru.get("a"), None);
        assert_eq!(lru.get("b"), Some("2"));
        assert_eq!(lru.get("c"), Some("3"));
        assert_eq!(lru.get("d"), Some("4"));
    }

    #[test]
    fn lru_put_existing_key_updates_value_and_promotes() {
        let mut lru = LruCache::new(3);
        lru.put("a", "1");
        lru.put("b", "2");
        lru.put("c", "3");
        lru.put("a", "updated");
        assert_eq!(lru.get("a"), Some("updated"));
        lru.put("d", "4");
        assert_eq!(lru.get("b"), None);
        assert_eq!(lru.get("a"), Some("updated"));
        assert_eq!(lru.get("c"), Some("3"));
        assert_eq!(lru.get("d"), Some("4"));
    }

    #[test]
    fn lru_capacity_boundary_evicts_only_past_capacity() {
        let mut lru = LruCache::new(3);
        lru.put("a", "1");
        lru.put("b", "2");
        lru.put("c", "3");
        assert_eq!(lru.get("a"), Some("1"));
        assert_eq!(lru.get("b"), Some("2"));
        assert_eq!(lru.get("c"), Some("3"));
        lru.put("d", "4");
        assert_eq!(lru.get("a"), None);
        assert_eq!(lru.get("d"), Some("4"));
    }

    #[test]
    fn lru_reuses_free_slot_after_eviction() {
        let mut lru = LruCache::new(2);
        lru.put("a", "1");
        lru.put("b", "2");
        lru.put("c", "3");
        assert_eq!(lru.get("a"), None);
        lru.put("d", "4");
        assert_eq!(lru.get("b"), None);
        assert_eq!(lru.get("c"), Some("3"));
        assert_eq!(lru.get("d"), Some("4"));
    }

    #[test]
    #[should_panic(expected = "LruCache capacity must be > 0")]
    fn lru_new_with_zero_capacity_panics() {
        let _ = LruCache::new(0);
    }

    #[test]
    fn store_matches_reference_model_on_random_ops() {
        let seed: u64 = 0xBEEF_1234_5678_9ABC;
        let mut rng = crate::skip::Rng::new(seed);
        let mut store = Store::new();
        let mut model: BTreeMap<String, String> = BTreeMap::new();

        for step in 0..2000u32 {
            let roll = rng.next_u64() % 100;
            let key = format!("k{}", rng.next_u64() % 200);
            let val = format!("v{}", rng.next_u64() % 1000);

            if roll < 55 {
                store.set(&key, &val);
                model.insert(key.clone(), val);
            } else if roll < 80 {
                assert_eq!(
                    store.get(&key),
                    model.get(&key).map(|v| v.as_str()),
                    "get mismatch at step {} key {} (seed {})",
                    step,
                    key,
                    seed
                );
            } else {
                assert_eq!(
                    store.delete(&key),
                    model.remove(&key),
                    "delete mismatch at step {} key {} (seed {})",
                    step,
                    key,
                    seed
                );
            }

            assert_eq!(
                store.len(),
                model.len(),
                "len mismatch at step {} (seed {})",
                step,
                seed
            );

            let from = format!("k{}", rng.next_u64() % 220);
            let to = format!("k{}", rng.next_u64() % 220);
            let got: Vec<(&str, &str)> = store.range(&from, &to).collect();
            if from < to {
                let want: Vec<(&str, &str)> = model
                    .range::<String, _>(&from..&to)
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                assert_eq!(
                    got, want,
                    "range mismatch at step {} from {} to {} (seed {})",
                    step, from, to, seed
                );
            } else {
                assert!(
                    got.is_empty(),
                    "range from >= to must be empty at step {} from {} to {} (seed {})",
                    step,
                    from,
                    to,
                    seed
                );
            }

            for (k, v) in &model {
                assert_eq!(
                    store.get(k),
                    Some(v.as_str()),
                    "content mismatch at step {} key {} (seed {})",
                    step,
                    k,
                    seed
                );
            }
        }
    }

    #[test]
    fn lru_matches_reference_model_on_random_ops() {
        let seed: u64 = 0xDEAD_9876_5432_1FED;
        let mut rng = crate::skip::Rng::new(seed);
        let capacity = 8usize;
        let mut lru = LruCache::new(capacity);
        // Model: index 0 = most recently used, last index = least recently used.
        let mut model: Vec<(String, String)> = Vec::new();

        for step in 0..2000u32 {
            let roll = rng.next_u64() % 100;
            let key = format!("k{}", rng.next_u64() % 20);
            let val = format!("v{}", rng.next_u64() % 1000);

            if roll < 55 {
                lru.put(&key, &val);
                if let Some(p) = model.iter().position(|(k, _)| *k == key) {
                    model.remove(p);
                }
                model.insert(0, (key, val));
                if model.len() > capacity {
                    model.pop();
                }
            } else {
                let pos = model.iter().position(|(k, _)| *k == key);
                let expected = pos.map(|p| {
                    let entry = model.remove(p);
                    let v = entry.1.clone();
                    model.insert(0, entry);
                    v
                });
                assert_eq!(
                    lru.get(&key),
                    expected.as_deref(),
                    "get mismatch at step {} key {} (seed {})",
                    step,
                    key,
                    seed
                );
            }
        }
    }
}

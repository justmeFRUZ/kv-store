use std::collections::HashMap;

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
struct Store {
    buckets: Vec<Vec<(String, String)>>,
    len: usize,
}

impl Store {
    fn bucket_index(&self, key: &str) -> usize {
        (fnv1a(key.as_bytes()) % self.buckets.len() as u64) as usize
    }

    fn set(&mut self, key: &str, value: &str) {
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
        if self.load_factor() > 0.75 {
            self.grow();
        }
    }

    fn get(&self, key: &str) -> Option<&str> {
        let idx = self.bucket_index(key);
        let bucket = &self.buckets[idx];
        for entry in bucket.iter() {
            if entry.0 == key {
                return Some(&entry.1);
            }
        }
        None
    }

    fn delete(&mut self, key: &str) -> Option<String> {
        let idx = self.bucket_index(key);
        let bucket = &mut self.buckets[idx];
        let pos = bucket.iter().position(|entry| entry.0 == key)?;
        let (_, value) = bucket.remove(pos);
        self.len -= 1;
        Some(value)
    }

    fn load_factor(&self) -> f64 {
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

struct LruNode {
    key: String,
    value: String,
    prev: Option<usize>,
    next: Option<usize>,
}

struct LruCache {
    nodes: Vec<LruNode>,
    free: Vec<usize>,
    map: HashMap<String, usize>,
    head: Option<usize>,
    tail: Option<usize>,
    capacity: usize,
}

impl LruCache {
    fn new(capacity: usize) -> Self {
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

    fn get(&mut self, key: &str) -> Option<&str> {
        let idx = *self.map.get(key)?;
        self.move_to_front(idx);
        Some(&self.nodes[idx].value)
    }

    fn put(&mut self, key: &str, value: &str) {
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

fn main() {
    let mut store = Store {
        buckets: vec![Vec::new(); 16],
        len: 0,
    };
    println!("buckets count: {}", store.buckets.len());
    println!("fnv1a(\"hello\") = {}", fnv1a(b"hello"));

    store.set("name", "Alice");
    store.set("name", "Bob");
    println!("get name = {:?}", store.get("name"));
    println!("get missing = {:?}", store.get("missing"));
    println!("get hello = {:?}", store.get("hello"));
    println!("load factor = {}", store.load_factor());

    println!("delete name = {:?}", store.delete("name"));
    println!("get name after delete = {:?}", store.get("name"));
    println!("delete missing = {:?}", store.delete("missing"));
    bench_set();
    bench_get();
    bench_delete();
    let mut lru = LruCache::new(2);
    lru.put("a", "1");
    lru.put("b", "2");
    let _ = lru.get("a");
    lru.put("c", "3");
    println!("lru a = {:?}", lru.get("a"));
    println!("lru b = {:?}", lru.get("b"));
    println!("lru c = {:?}", lru.get("c"));
}

fn bench_set() {
    for n in [100usize, 1_000, 10_000, 100_000] {
        let mut store = Store {
            buckets: vec![Vec::new(); 16],
            len: 0,
        };
        let start = std::time::Instant::now();
        for i in 0..n {
            let key = format!("key{}", i);
            store.set(&key, "v");
        }
        let elapsed = start.elapsed();
        println!(
            "set n={} buckets={} elapsed={:?}",
            n,
            store.buckets.len(),
            elapsed
        );
    }
}

fn bench_get() {
    for n in [100usize, 1_000, 10_000, 100_000] {
        let mut store = Store {
            buckets: vec![Vec::new(); 16],
            len: 0,
        };
        let keys: Vec<String> = (0..n).map(|i| format!("key{}", i)).collect();
        for key in &keys {
            store.set(key, "v");
        }
        let mut sink: usize = 0;
        let start = std::time::Instant::now();
        for key in &keys {
            if let Some(value) = store.get(key) {
                sink += value.len();
            }
        }
        let elapsed = start.elapsed();
        let sink = std::hint::black_box(sink);
        println!(
            "get n={} buckets={} elapsed={:?} sink={}",
            n,
            store.buckets.len(),
            elapsed,
            sink,
        );
    }
}

fn bench_delete() {
    for n in [100usize, 1_000, 10_000, 100_000] {
        let mut store = Store {
            buckets: vec![Vec::new(); 16],
            len: 0,
        };
        let keys: Vec<String> = (0..n).map(|i| format!("key{}", i)).collect();
        for key in &keys {
            store.set(key, "v");
        }
        let mut sink: usize = 0;
        let start = std::time::Instant::now();
        for key in &keys {
            if let Some(value) = store.delete(key) {
                sink += value.len();
            }
        }
        let elapsed = start.elapsed();
        let sink = std::hint::black_box(sink);
        println!(
            "delete n={} buckets={} elapsed={:?} sink={}",
            n,
            store.buckets.len(),
            elapsed,
            sink,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_then_get_returns_value() {
        let mut store = Store {
            buckets: vec![Vec::new(); 16],
            len: 0,
        };
        store.set("name", "Alice");
        assert_eq!(store.get("name"), Some("Alice"));
    }

    #[test]
    fn two_keys_in_same_bucket_are_both_retrievable() {
        let mut store = Store {
            buckets: vec![Vec::new(); 1],
            len: 0,
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
        };
        assert_eq!(store.delete("anything"), None);
    }

    #[test]
    fn delete_only_item_in_bucket_leaves_bucket_empty() {
        let mut store = Store {
            buckets: vec![Vec::new(); 1],
            len: 0,
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
}

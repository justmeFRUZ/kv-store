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
}

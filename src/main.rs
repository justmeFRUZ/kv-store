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
}

fn main() {
    let mut store = Store {
        buckets: vec![Vec::new(); 16],
    };
    println!("buckets count: {}", store.buckets.len());
    println!("fnv1a(\"hello\") = {}", fnv1a(b"hello"));

    store.set("name", "Alice");
    store.set("name", "Bob");
    println!("get name = {:?}", store.get("name"));
    println!("get missing = {:?}", store.get("missing"));
    println!("get hello = {:?}", store.get("hello"));
}

use kv_store::skip::SkipList;
use kv_store::{LruCache, Store};
use std::collections::BTreeMap;
use std::collections::HashMap;

fn main() {
    let mut store = Store::new();
    println!("buckets count: {}", store.bucket_count());
    println!("fnv1a(\"hello\") = {}", kv_store::fnv1a(b"hello"));

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
    bench_vs_hashmap();
    bench_skiplist_insert_vs_btreemap();
    bench_skiplist_get_vs_btreemap();
    bench_skiplist_range_vs_btreemap();
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
        let mut store = Store::new();
        let start = std::time::Instant::now();
        for i in 0..n {
            let key = format!("key{}", i);
            store.set(&key, "v");
        }
        let elapsed = start.elapsed();
        println!(
            "set n={} buckets={} elapsed={:?}",
            n,
            store.bucket_count(),
            elapsed
        );
    }
}

fn bench_get() {
    for n in [100usize, 1_000, 10_000, 100_000] {
        let mut store = Store::new();
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
            "get n={} buckets={} elapsed={:?} sink{}",
            n,
            store.bucket_count(),
            elapsed,
            sink,
        );
    }
}

fn bench_delete() {
    for n in [100usize, 1_000, 10_000, 100_000] {
        let mut store = Store::new();
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
            store.bucket_count(),
            elapsed,
            sink,
        );
    }
}

fn bench_vs_hashmap() {
    for n in [100usize, 1_000, 10_000, 100_000] {
        let mut store = Store::new();
        let start = std::time::Instant::now();
        for i in 0..n {
            let key = format!("key{}", i);
            store.set(&key, "v");
        }
        let store_elapsed = start.elapsed();

        let mut map: HashMap<String, String> = HashMap::new();
        let start = std::time::Instant::now();
        for i in 0..n {
            let key = format!("key{}", i);
            map.insert(key, "v".to_string());
        }
        let map_elapsed = start.elapsed();

        println!(
            "set-vs-hashmap n={} store_len={} map_len={} store={:?} hashmap{:?} ratio={:.2}",
            n,
            store.len(),
            map.len(),
            store_elapsed,
            map_elapsed,
            store_elapsed.as_secs_f64() / map_elapsed.as_secs_f64(),
        );
    }
}

fn bench_skiplist_insert_vs_btreemap() {
    for n in [1_000usize, 10_000, 100_000, 1_000_000] {
        let mut list = SkipList::new(99);
        let start = std::time::Instant::now();
        for i in 0..n {
            let key = format!("key{}", i);
            list.insert(&key, "v");
        }
        let list_elapsed = start.elapsed();

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        let start = std::time::Instant::now();
        for i in 0..n {
            let key = format!("key{}", i);
            map.insert(key, "v".to_string());
        }
        let map_elapsed = start.elapsed();

        println!(
            "insert-skiplist-vs-btreemap n={} list_len={} map_len={} skiplist={:?} btreemap={:?} ratio={:.2}",
            n,
            list.len(),
            map.len(),
            list_elapsed,
            map_elapsed,
            list_elapsed.as_secs_f64() / map_elapsed.as_secs_f64(),
        );
    }
}

fn bench_skiplist_get_vs_btreemap() {
    for n in [1_000usize, 10_000, 100_000, 1_000_000] {
        let keys: Vec<String> = (0..n).map(|i| format!("key{}", i)).collect();
        let mut list = SkipList::new(99);
        for key in &keys {
            list.insert(key, "v");
        }
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for key in &keys {
            map.insert(key.clone(), "v".to_string());
        }

        let mut sink: usize = 0;
        let start = std::time::Instant::now();
        for key in &keys {
            if let Some(v) = list.get(key) {
                sink += v.len();
            }
        }
        let list_elapsed = start.elapsed();
        let _ = std::hint::black_box(sink);

        let mut sink: usize = 0;
        let start = std::time::Instant::now();
        for key in &keys {
            if let Some(v) = map.get(key) {
                sink += v.len();
            }
        }
        let map_elapsed = start.elapsed();
        let _ = std::hint::black_box(sink);

        println!(
            "get-skiplist-vs-btreemap n={} list_len={} map_len={} skiplist={:?} btreemap={:?} ratio={:.2}",
            n,
            list.len(),
            map.len(),
            list_elapsed,
            map_elapsed,
            list_elapsed.as_secs_f64() / map_elapsed.as_secs_f64(),
        );
    }
}

fn bench_skiplist_range_vs_btreemap() {
    let start_string = String::from("key");
    let end_string = String::from("kez");
    for n in [1_000usize, 10_000, 100_000, 1_000_000] {
        let keys: Vec<String> = (0..n).map(|i| format!("key{}", i)).collect();
        let mut list = SkipList::new(99);
        for key in &keys {
            list.insert(key, "v");
        }
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for key in &keys {
            map.insert(key.clone(), "v".to_string());
        }

        let mut sink: usize = 0;
        let start = std::time::Instant::now();
        for (k, v) in list.range(start_string.as_str(), end_string.as_str()) {
            sink += k.len() + v.len();
        }
        let list_elapsed = start.elapsed();
        let _ = std::hint::black_box(sink);

        let mut sink: usize = 0;
        let map_start = start_string.clone();
        let map_end = end_string.clone();
        let start = std::time::Instant::now();
        for (k, v) in map.range(map_start..map_end) {
            sink += k.len() + v.len();
        }
        let map_elapsed = start.elapsed();
        let _ = std::hint::black_box(sink);

        println!(
            "range-skiplist-vs-btreemap n={} list_len={} map_len={} skiplist={:?} btreemap={:?} ratio={:.2}",
            n,
            list.len(),
            map.len(),
            list_elapsed,
            map_elapsed,
            list_elapsed.as_secs_f64() / map_elapsed.as_secs_f64(),
        );
    }
}

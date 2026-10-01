struct Store {
    buckets: Vec<Vec<(String, String)>>,
}

fn main() {
    let store = Store {
        buckets: Vec::new(),
    };
    println!("buckets count: {}", store.buckets.len());
}

use std::process;

fn main() {
    let addr = "127.0.0.1:7878";
    if let Err(e) = kv_store::server::run(addr) {
        eprint!("server error: {}", e);
        process::exit(1);
    }
}
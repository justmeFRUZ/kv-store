use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

pub fn run(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {}", addr);
    serve(listener)
}

pub fn serve(listener: TcpListener) -> std::io::Result<()> {
    loop {
        let Some((stream, peer_addr)) = next_or_log(listener.accept()) else {
            continue;
        };

        if let Err(e) = handle_client(stream, peer_addr) {
            eprintln!("connection error: {}", e);
        }
    }
}

fn handle_client(mut stream: TcpStream, peer_addr: SocketAddr) -> std::io::Result<()> {
    let mut buf = [0; 1024];
    let mut total: usize = 0;
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            // 0-byte read means the client closed the connection
            eprintln!("{} sent {} bytes", peer_addr, total);
            return Ok(());
        }
        total += bytes_read;
        stream.write_all(&buf[..bytes_read])?;
    }
}

fn next_or_log(
    accept_result: std::io::Result<(TcpStream, SocketAddr)>,
) -> Option<(TcpStream, SocketAddr)> {
    match accept_result {
        Ok(pair) => Some(pair),
        Err(e) => {
            eprintln!("accept error: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_or_log_returns_none_on_err() {
        let err = std::io::Error::other("synthetic accept failure");
        assert!(next_or_log(Err(err)).is_none());
    }

    #[test]
    fn next_or_log_returns_some_on_ok() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let addr = listener.local_addr().expect("local addr");
        let _client = TcpStream::connect(addr).expect("connect");
        let accepted = listener.accept();
        assert!(next_or_log(accepted).is_some());
    }
}

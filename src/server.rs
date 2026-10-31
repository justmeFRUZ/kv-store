use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

pub fn run(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {}", addr);
    serve(listener)
}

pub fn serve(listener: TcpListener) -> std::io::Result<()> {
    loop {
        match listener.accept() {
            Ok((stream, peer_addr)) => {
                if let Err(e) = handle_client(stream, peer_addr) {
                    eprintln!("connection error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("accept error: {}", e);
            }
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

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub fn run(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {}", addr);
    serve(listener)
}

pub fn serve(listener: TcpListener) -> std::io::Result<()> {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_client(stream) {
                    eprintln!("connection error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("accept error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = [0; 1024];
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            // 0-byte read means the client closed the connection
            return Ok(());
        }
        stream.write_all(&buf[..bytes_read])?;
    }
}
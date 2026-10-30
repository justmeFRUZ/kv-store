use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

#[test]
fn test_echo_round_trip() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        let _ = kv_store::server::serve(listener);
    });

    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let msg = b"hello";
    stream.write_all(msg).unwrap();

    let mut buf = [0; 5];
    stream.read_exact(&mut buf).unwrap();

    assert_eq!(&buf, msg);
}


#[test]
fn  test_echo_100kn() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        let _ = kv_store::server::serve(listener);
    });

    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let msg = vec![42u8; 100 * 1024];
    let mut stream_clone = stream.try_clone().unwrap();

    let msg_clone = msg.clone();
    let handle = thread::spawn(move || {
        stream_clone.write_all(&msg_clone).unwrap();
    });

    let mut buf = vec![0u8; 100 * 1024];
    stream.read_exact(&mut buf).unwrap();

    handle.join().unwrap();
    assert_eq!(buf, msg);
}


#[test]
fn test_abrupt_disconnect_does_not_keill_server() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        let _ = kv_store::server::serve(listener);
    });

    {
        let _stream = TcpStream::connect(addr).unwrap();
    }

    thread::sleep(Duration::from_millis(50));

    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let msg = b"still alive";
    stream.write_all(msg).unwrap();

    let mut buf = [0; 11];
    stream.read_exact(&mut buf).unwrap();

    assert_eq!(&buf, msg);
}
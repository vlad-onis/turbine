use std::io::Result;
#[allow(unused_imports)]
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;

fn greet(mut stream: TcpStream) {
    // // Set up reading
    // let _ = stream.set_read_timeout(Some(Duration::from_micros(10)));
    // let mut buf: Vec<u8> = Vec::new();
    // let _ = stream.read_to_end(&mut buf);

    let _ = stream.shutdown(std::net::Shutdown::Read);

    let response = "HTTP/1.1 200 OK\r\nConnection: Closed\r\n\r\n";
    let _ = stream.write_all(response.as_bytes());
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;

    for stream in listener.incoming() {
        println!("####New connection received");
        if let Ok(s) = stream {
            greet(s);
        }
    }

    Ok(())
}

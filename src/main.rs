mod http;

use http::{ParseError, Request as HttpRequest};

use std::fs;
use std::io::Result as IOResult;
#[allow(unused_imports)]
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::string::FromUtf8Error;

use thiserror::Error;


#[derive(Error, Debug)]
enum ServerError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Failed to convert byte stream to String: {0}")]
    Conversion(#[from] FromUtf8Error),

    #[error("Parsing the request failed because: {0}")]
    RequestParsing(#[from] http::ParseError),
}

// Static lifetime is infered here
const END_OF_CONTENT: &str = "\r\n\r\n";
const HEADER_STATUS: &str = "HTTP/1.1 200 OK\r\n";
const HEADER_CONTENT_TYPE: &str = "Content-Type: text/html; charset=UTF-8\r\n";
const NEW_LINE: &str = "\r\n";

fn read_stream_content_to_end(stream: &mut TcpStream) -> Result<http::Request, http::ParseError> {
    let mut buffer = [0; 1024]; // Adjust buffer size as needed
    let mut request = Vec::new();

    loop {
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read == 0 {
            break; // Connection was closed
        }

        request.extend_from_slice(&buffer[..bytes_read]);

        // Check if the end of the request is reached
        if request.ends_with(b"\r\n\r\n") {
            break;
        }
    }

    let request = String::from_utf8_lossy(&request).to_string();
    let request = HttpRequest::new(request)?;

    Ok(request)
}

fn parse_request(stream: &mut TcpStream) -> Result<String, ParseError> {
    let request = read_stream_content_to_end(stream)?;

    let resource = request.headers.resource;

    Ok(resource)
}

fn serve_file(mut stream: TcpStream) -> Result<(), ServerError> {
    let resource = parse_request(&mut stream)?;
    println!("Parsed Resource : {:?}", resource);

    let file_content = fs::read_to_string("index.html")?;
    let content_length = file_content.len() + END_OF_CONTENT.len();

    stream.write_all(HEADER_STATUS.as_bytes())?;
    stream.write_all(HEADER_CONTENT_TYPE.as_bytes())?;

    let content_length = format!("Content-Length: {}\r\n", content_length);
    stream.write_all(content_length.as_bytes())?;

    stream.write_all(NEW_LINE.as_bytes())?;

    stream.write_all(file_content.as_bytes())?;
    stream.write_all(END_OF_CONTENT.as_bytes())?;

    Ok(())
}

fn _greet(mut stream: TcpStream) {
    // // Set up reading
    // let _ = stream.set_read_timeout(Some(Duration::from_micros(10)));
    // let mut buf: Vec<u8> = Vec::new();
    // let _ = stream.read_to_end(&mut buf);

    let _ = stream.shutdown(std::net::Shutdown::Read);

    let response = "HTTP/1.1 200 OK\r\nConnection: Closed\r\n\r\n";
    let _ = stream.write_all(response.as_bytes());
}

fn main() -> IOResult<()> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;

    for stream in listener.incoming() {
        println!("#### New connection received");
        if let Ok(s) = stream {
            let res = serve_file(s);
            println!("Serving: {:?}", res);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse_headers_fail() {
        assert!(Headers::new(vec![]).is_err());
        assert!(Headers::new(vec!["GET", "/"]).is_err());
        assert!(Headers::new(vec!["GWET", "/", "HTTP/1.1"]).is_err());
    }

    #[test]
    pub fn test_parse_headers() {
        let header = Headers::new(vec!["GET", "/", "HTTP/1.1"]);
        println!("{header:?}");

        assert!(Headers::new(vec!["GET", "/", "HTTP/1.1"]).is_ok());
        assert!(Headers::new(vec!["POST", "/", "HTTP/1.1"]).is_ok());
        assert!(Headers::new(vec!["GET", "/foo", "HTTP/1.1"]).is_ok());
    }
}

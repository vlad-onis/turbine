use std::fs;
use std::io::Result as IOResult;
#[allow(unused_imports)]
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::string::FromUtf8Error;
use std::time::Duration;

use thiserror::Error;

#[derive(Error, Debug)]
enum ServerError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Failed to convert byte stream to String: {0}")]
    Conversion(#[from] FromUtf8Error),

    #[error("Parsing the request failed because: {0}")]
    RequestParsing(#[from] ParseError),
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("Http request cannot be empty")]
    EmptyRequest,

    #[error("Invalid request")]
    InvalidRequest,
}

// Static lifetime is infered here
const END_OF_CONTENT: &str = "\r\n\r\n";
const HEADER_STATUS: &str = "HTTP/1.1 200 OK\r\n";
const HEADER_CONTENT_TYPE: &str = "Content-Type: text/html; charset=UTF-8\r\n";
const NEW_LINE: &str = "\r\n";

fn parse_request(request: &str) -> Result<String, ParseError> {
    let lines: Vec<_> = request.split("\r\n").collect();

    let first_line = lines.first().ok_or(ParseError::EmptyRequest)?;

    let words = first_line.split_whitespace().collect::<Vec<_>>();

    if words.len() != 3 {
        return Err(ParseError::InvalidRequest);
    }

    let resource = words[1].clone();

    Ok(resource.to_string())
}

fn serve_file(mut stream: TcpStream) -> Result<(), ServerError> {
    stream.set_read_timeout(Some(Duration::from_micros(1000)))?;

    let mut buf: Vec<u8> = Vec::new();
    let mut buf = String::new();
    // this throws an error, when the timeout is reached.
    // because the connection is not close so the end is not reached.
    let _ = stream.read_to_string(&mut buf)?;

    // let request = String::from_utf8(buf)?;
    let request = buf;

    // todo: Remove printlns
    println!("Request: {}", request);
    let resource = parse_request(&request)?;
    println!("Parsed Resource : {}", resource);

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

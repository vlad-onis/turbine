use thiserror::Error;

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::string::FromUtf8Error;

use crate::config::Config;
use crate::http::{HttpPath, ParseError, Request as HttpRequest};
use crate::resolver::{ResolveError, Resolver};

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Failed to convert byte stream to String: {0}")]
    Conversion(#[from] FromUtf8Error),

    #[error("Parsing the request failed because: {0}")]
    RequestParsing(#[from] ParseError),

    #[error("Resolving the request failed because: {0}")]
    ResolverError(#[from] ResolveError),
}

// Static lifetime is infered here
const END_OF_CONTENT: &str = "\r\n\r\n";
const HEADER_STATUS: &str = "HTTP/1.1 200 OK\r\n";
const HEADER_CONTENT_TYPE: &str = "Content-Type: text/html; charset=UTF-8\r\n";
const NEW_LINE: &str = "\r\n";

pub struct Server {
    resolver: Resolver,
}

impl Server {
    pub fn new(config: Config) -> Result<Self, ServerError> {
        let document_root = config.document_root;
        let canonicalized_document_root = fs::canonicalize(document_root)?;
        Ok(Self {
            resolver: Resolver::new(canonicalized_document_root),
        })
    }

    pub fn run(&self) -> Result<(), ServerError> {
        println!("Starting turbine");

        let listener = TcpListener::bind("0.0.0.0:12345")?;

        for stream in listener.incoming() {
            println!("#### New connection received");
            if let Ok(s) = stream {
                let res = self.serve_file(s);
                println!("{:?}", res);
            }
        }

        Ok(())
    }

    /// Reads the content of the stream until the end of the request is reached
    /// Acts as a converter from [TcpStream] to [http::Request] to ensure a validated request
    /// and separation of concerns going forward
    fn read_stream_content_to_end(
        &self,
        stream: &mut TcpStream,
    ) -> Result<HttpRequest, ParseError> {
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

    /// Parses the request and returns the resource path
    /// Resource path is the path to the file that should be served
    /// The path is validated to ensure that it is a file inside the web_resources directory
    /// It defaults to index.html if the path is a directory
    fn parse_request(&self, request: &HttpRequest) -> Result<HttpPath, ResolveError> {
        self.resolver.resolve(request.headers.resource.clone())
    }

    /// Reads the content of the file specified by the resource path
    fn get_resource_content(&self, resource: &HttpPath) -> std::io::Result<String> {
        let file_content = fs::read_to_string(resource)?;
        Ok(file_content)
    }

    /// Serves the file specified by the resource path back to the client
    fn serve_file(&self, mut stream: TcpStream) -> Result<(), ServerError> {
        let request = self.read_stream_content_to_end(&mut stream)?;

        let resource = self.parse_request(&request)?;

        let resource_content = self.get_resource_content(&resource)?;
        let content_length = resource_content.len() + END_OF_CONTENT.len();

        stream.write_all(HEADER_STATUS.as_bytes())?;
        stream.write_all(HEADER_CONTENT_TYPE.as_bytes())?;

        let content_length = format!("Content-Length: {}\r\n", content_length);
        stream.write_all(content_length.as_bytes())?;

        stream.write_all(NEW_LINE.as_bytes())?;

        stream.write_all(resource_content.as_bytes())?;
        stream.write_all(END_OF_CONTENT.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::*;

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

    #[test]
    pub fn try_from_for_http_path() {
        let document_root = std::env::current_dir().unwrap().join("web_resources");

        let path = HttpPath::try_from("/index.html".to_string());
        assert!(path.is_ok());
        assert_eq!(
            path.unwrap().as_path(),
            Path::new(document_root.join("index.html").to_str().unwrap())
        );

        let path = HttpPath::try_from("/".to_string());
        assert!(path.is_ok());
        assert_eq!(
            path.unwrap().as_path(),
            Path::new(document_root.join("index.html").to_str().unwrap())
        );

        let path = HttpPath::try_from("/foo".to_string());
        assert!(path.is_ok());
        assert_eq!(
            path.unwrap().as_path(),
            Path::new(document_root.join("foo/index.html").to_str().unwrap())
        );

        let path = HttpPath::try_from("/foo/".to_string());
        assert!(path.is_ok());
        assert_eq!(
            path.unwrap().as_path(),
            Path::new(document_root.join("foo/index.html").to_str().unwrap())
        );

        let path = HttpPath::try_from("/foo/bar".to_string());
        println!("{:?}", path);
        assert!(path.is_ok());
        assert_eq!(
            path.unwrap().as_path(),
            Path::new(document_root.join("foo/bar/index.html").to_str().unwrap())
        );

        let path = HttpPath::try_from("".to_string());
        assert!(path.is_err());

        let path = HttpPath::try_from("../index.html".to_string());
        assert!(path.is_err());

        let path = HttpPath::try_from("/../index.html".to_string());
        assert!(path.is_err());

        let path = HttpPath::try_from("foo".to_string());
        assert!(path.is_err());
    }
}

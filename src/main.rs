mod http;

use http::{ParseError, Request as HttpRequest};

use std::fs;
use std::io::Result as IOResult;
#[allow(unused_imports)]
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
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

/// Reads the content of the stream until the end of the request is reached
/// Acts as a converter from [TcpStream] to [http::Request] to ensure a validated request
/// and separation of concerns going forward
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

/// Specifies a valid HTTP path after parsing
#[derive(Debug)]
struct HttpPath(PathBuf);

impl Deref for HttpPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Path> for HttpPath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

/// Usage:
/// ```rust
/// let path = HttpPath::try_from("/web_resources/index.html".to_string());
/// ```
impl TryFrom<String> for HttpPath {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        
        // Paths should start with a slash to represent the root of the resource hierarchy
        // Also the slash refers to the root directory, if it was not there the server might
        // try to resolve the path as the root directory leading to potential flawed results
        if !value.starts_with("/") {
            return Err(ParseError::PathShouldStartWithSlash(value));
        }

        // remove spaces up to the first slash
        let value = value.trim_start_matches("/").to_string();

        
        let path = if value.as_str() == "/" {
            PathBuf::from("web_resources")
        } else {
            PathBuf::from("web_resources").join(&value)
        };

        // From here on, this needs to be an absolute path
        // If path was -> "/../index.html" this would end up in the parent directory of 
        // the document root. This function returns the absolute path to this parent
        // directory
        let path = fs::canonicalize(path)?;

        // This path needs to be a file or a directory inside the web_resources directory
        // inside the current working directory
        let expected_prefix = std::env::current_dir()?.join("web_resources");
        if !path.starts_with(&expected_prefix) {
            println!("Path does not start with web_resources");
            return Err(ParseError::PathOutsideDocumentRoot(value));
        }

        if path.is_file() {
            return Ok(HttpPath(path));
        }

        // assume index.html as the default file to look for when the path is a directory
        if path.is_dir() {
            return Ok(HttpPath(path.join("index.html")));
        }

        
        Err(ParseError::InvalidPath(value))
    }
}

/// Parses the request and returns the resource path
/// Resource path is the path to the file that should be served
/// The path is validated to ensure that it is a file inside the web_resources directory
/// It defaults to index.html if the path is a directory
fn parse_request(request: &http::Request) -> Result<HttpPath, ParseError> {
    println!("Request: {:?}", request);
    let resource = request.headers.resource.clone();
    println!("Resource: {:?}", resource);

    let http_path = HttpPath::try_from(resource)?;
    println!("HttpPath: {:?}", http_path);

    Ok(http_path)
}

/// Reads the content of the file specified by the resource path
fn get_resource_content(resource: &HttpPath) -> std::io::Result<String> {
    let file_content = fs::read_to_string(resource)?;
    Ok(file_content)
}

/// Serves the file specified by the resource path back to the client
fn serve_file(mut stream: TcpStream) -> Result<(), ServerError> {
    let request = read_stream_content_to_end(&mut stream)?;

    let resource = parse_request(&request)?;
    println!("Parsed Resource : {:?}", resource);

    let resource_content = get_resource_content(&resource)?;
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

fn main() -> IOResult<()> {
    println!("Starting turbine");
    let listener = TcpListener::bind("0.0.0.0:12345")?;

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

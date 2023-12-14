use std::collections::HashMap;
#[allow(unused_imports)]
use std::io::{Read, Write};

use thiserror::Error;

/// Errors that can occur when parsing a http request
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Http request cannot be empty")]
    EmptyRequest,

    #[error("Headers must have method, resource, version")]
    InvalidHeaders,

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Unknown or unsupported http method : {0}")]
    InvalidMethod(String),

    #[error("Resource {0} could not be found because it points outside the document root")]
    PathOutsideDocumentRoot(String),

    #[error("Path {0} should start with a slash")]
    PathShouldStartWithSlash(String),

    #[error("Path {0} is invalid")]
    InvalidPath(String)
}

/// Supported HTTP methods
#[derive(Debug)]
pub enum Method {
    Get,
    Post,
}

/// Representation of HTTP headers
#[derive(Debug)]
pub struct Headers {
    pub method: Method,
    pub resource: String,
    pub version: String,

    // All the possible http headers will be stored here
    pub other_headers: HashMap<String, String>,
}

impl Headers {

    /// Creates a new [Headers] instance from a vector of strings
    pub fn new(headers: Vec<&str>) -> Result<Headers, ParseError> {

        // At least the method, resource and version should be present
        if headers.len() != 3 {
            return Err(ParseError::InvalidHeaders);
        }

        let method = match headers[0] {
            "GET" => Method::Get,
            "POST" => Method::Post,
            unknown => return Err(ParseError::InvalidMethod(unknown.to_string())),
        };

        let resource = headers[1].to_string();
        let version = headers[2].to_string();
        
        // TODO: Parse other headers
        let other_headers = HashMap::new();

        Ok(Headers {
            method,
            resource,
            version,
            other_headers,
        })
    }
}

/// Representation of a HTTP request
#[derive(Debug)]
pub struct Request {
    pub headers: Headers,
    pub body: Vec<u8>,
}

impl Request {
    pub fn new(request: String) -> Result<Request, ParseError> {
        let lines: Vec<_> = request.split("\r\n").collect();

        let first_line = lines.first().ok_or(ParseError::EmptyRequest)?;

        let words = first_line.split_whitespace().collect::<Vec<_>>();

        let headers = Headers::new(words)?;

        // todo: Extract the body when we're dealing with POST requests
        let body = Vec::new();

        Ok(Request { headers, body })
    }
}

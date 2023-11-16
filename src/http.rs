use std::collections::HashMap;
#[allow(unused_imports)]
use std::io::{Read, Write};

use thiserror::Error;

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
}

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug)]
pub struct Headers {
    pub method: Method,
    pub resource: String,
    pub version: String,
    pub other_headers: HashMap<String, String>,
}

impl Headers {
    pub fn new(headers: Vec<&str>) -> Result<Headers, ParseError> {
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
        let other_headers = HashMap::new();

        Ok(Headers {
            method,
            resource,
            version,
            other_headers,
        })
    }
}

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

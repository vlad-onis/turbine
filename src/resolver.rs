use thiserror::Error;

use std::path::PathBuf;

use crate::http::HttpPath;

/// Errors that can occur when parsing a http request
#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("Resource {0} could not be found because it points outside the document root")]
    PathOutsideDocumentRoot(HttpPath),

    #[error("Path {0} should start with a slash")]
    PathShouldStartWithSlash(String),

    #[error("HttpError: {0}")]
    HttpPathError(#[from] crate::http::ParseError),
}

pub struct Resolver {
    /// The canonicalized document root
    document_root: PathBuf,
}

impl Resolver {
    pub fn new(document_root: PathBuf) -> Self {
        Self { document_root }
    }

    /// Parses the request and returns the resource path as an absolute path
    ///
    /// The path is validated to ensure that it is a file inside the
    /// web_resources directory
    ///
    /// # Errors
    ///
    /// Returns an error if the path
    /// - cannot be converted to an `HttpPath`
    /// - is outside the document root
    pub fn resolve(&self, resource: String) -> Result<HttpPath, ResolveError> {
        if !resource.starts_with('/') {
            return Err(ResolveError::PathShouldStartWithSlash(resource));
        }

        // Absolute paths replace the document root
        // Therefore we need to remove the leading slash
        let trimmed = resource.trim_start_matches('/');
        let resource = self.document_root.join(trimmed);

        // this is an absolute path
        let http_path = HttpPath::try_from(resource)?;

        // check if the absolute path file is inside the document root
        if !http_path.starts_with(&self.document_root) {
            return Err(ResolveError::PathOutsideDocumentRoot(http_path));
        }

        Ok(http_path)
    }
}

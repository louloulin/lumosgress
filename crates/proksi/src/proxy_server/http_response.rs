use http::{HeaderMap, StatusCode};
use bytes::Bytes;

/// HTTP response structure for returning responses to clients
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code
    pub status_code: StatusCode,
    /// HTTP headers
    pub headers: HeaderMap,
    /// HTTP body
    pub body: Bytes,
}

impl HttpResponse {
    /// Create a new HTTP response
    pub fn new(status_code: StatusCode, headers: HeaderMap, body: Bytes) -> Self {
        Self {
            status_code,
            headers,
            body,
        }
    }
    
    /// Create a new text response
    pub fn text(status_code: StatusCode, text: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        
        Self {
            status_code,
            headers,
            body: Bytes::from(text.to_string()),
        }
    }
    
    /// Create a new JSON response
    pub fn json<T: serde::Serialize>(status_code: StatusCode, value: &T) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );
        
        let body = serde_json::to_string(value)?;
        
        Ok(Self {
            status_code,
            headers,
            body: Bytes::from(body),
        })
    }
} 
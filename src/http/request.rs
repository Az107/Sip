// Written by Alberto Ruiz 2025-01-01
//
// This module defines the HTTP request structure and a streaming builder for parsing raw input.
// While the core functionality is usable, there are known limitations:
// - No support for chunked transfer encoding
// - Partial header validation
// - No URI normalization or encoding
//

use super::{HttpMethod, HttpStatus};
use std::{collections::HashMap, default, net::TcpStream, str};

const MAX_HEADER_SIZE: usize = 1024 * 16;
const MAX_HEADER_COUNT: usize = 100;

/// Represents a parsed HTTP request.
///
/// Contains method, path, optional query arguments, headers, body, and a stream (for low-level access).
#[derive(Debug)]
pub struct HttpRequest {
    pub host: String,
    pub method: HttpMethod,
    pub path: String,
    pub args: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Creates a new HTTP request with the given method and path.
    pub fn new(method: HttpMethod, host: &str, path: &str) -> Self {
        return HttpRequest {
            method,
            host: host.to_string(),
            path: path.to_string(),
            args: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        };
    }

    pub fn parse(raw: String) -> Result<Self, &'static str> {
        let mut lines = raw.split('\n');
        let status_line = lines.next().ok_or("invalid status")?.to_string();
        let (raw_method, path) = status_line.split_once(" ").ok_or("Invalud status")?;
        let raw_method = raw_method.trim();
        let path = path.trim();
        let path = path
            .strip_prefix("http://")
            .unwrap_or(path.strip_prefix("https://").unwrap_or(path));
        let (host, path) = path.split_once('/').unwrap();
        let host = host.to_string();
        // let (path, args) = path.split_once("?").unwrap();
        let mut path = path.to_string();
        path.insert(0, '/');
        let method = HttpMethod::from_str(raw_method);
        let mut headers = HashMap::new();
        while let Some(line) = lines.next() {
            if !line.contains(':') || line.len() <= 1 {
                break;
            }
            let (k, v) = line.split_once(":").unwrap();
            let k = k.trim().to_lowercase().to_string();
            let v = v.trim().to_string();
            headers.insert(k, v);
        }
        if !headers.contains_key("host") {
            headers.insert("host".to_string(), host.clone());
        }
        let mut body = Vec::new();
        while let Some(line) = lines.next() {
            body.extend_from_slice(line.as_bytes());
        }
        if !headers.contains_key("content-length") {
            headers.insert("content-length".to_string(), body.len().to_string());
        }
        let request = HttpRequest {
            host,
            method,
            path,
            args: HashMap::new(),
            headers,
            body,
        };

        println!("{:?}", request);

        return Ok(request);
    }

    /// Returns a blank default request (empty method/path/headers).
    pub fn default() -> Self {
        HttpRequest {
            method: HttpMethod::Other(String::new()),
            path: String::new(),
            host: String::new(),
            args: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Returns a blank default request (empty method/path/headers).
    pub fn clone(&self) -> Self {
        return HttpRequest {
            method: self.method.clone(),
            host: self.host.clone(),
            path: self.path.clone(),
            args: self.args.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
        };
    }

    /// Attempts to decode the body as UTF-8 and return it as text.
    pub fn text(&self) -> Option<String> {
        if self.body.len() == 0 {
            return None;
        }
        let body = match str::from_utf8(self.body.as_slice()) {
            Ok(v) => Some(v.to_string()),
            Err(_e) => None,
        };
        return body;
    }
}

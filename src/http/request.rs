// Written by Alberto Ruiz 2025-01-01
//
// This module defines the HTTP request structure and a streaming builder for parsing raw input.
// While the core functionality is usable, there are known limitations:
// - No support for chunked transfer encoding
// - Partial header validation
// - No URI normalization or encoding
//

use crate::http::HttpHeaders;

use super::HttpMethod;
use std::{collections::HashMap, str};

/// Represents a parsed HTTP request.
///
/// Contains method, path, optional query arguments, headers, body, and a stream (for low-level access).
#[derive(Debug)]
pub struct HttpRequest {
    pub ssl: bool,
    pub host: String,
    pub method: HttpMethod,
    pub path: String,
    pub args: HashMap<String, String>,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Creates a new HTTP request with the given method and path.
    pub fn new(method: HttpMethod, host: &str, path: &str) -> Self {
        HttpRequest {
            ssl: false,
            method,
            host: host.to_string(),
            path: path.to_string(),
            args: HashMap::new(),
            headers: HttpHeaders::new(),
            body: Vec::new(),
        }
    }

    pub fn parse(raw: String) -> Result<Self, &'static str> {
        let mut ssl = false;
        let mut lines = raw.split('\n');
        let status_line = lines.next().ok_or("invalid status")?.to_string();
        let (raw_method, path) = status_line.split_once(" ").ok_or("Invalud status")?;
        let raw_method = raw_method.trim();
        let path = path.trim();
        if path.starts_with("https://") {
            ssl = true;
        }
        let path = path
            .strip_prefix("http://")
            .unwrap_or(path.strip_prefix("https://").unwrap_or(path));
        let (host, path) = path.split_once('/').unwrap_or((path, ""));
        let host = host.to_string();
        // let (path, args) = path.split_once("?").unwrap();
        let mut path = path.to_string();
        path.insert(0, '/');
        let method = HttpMethod::from_str(raw_method);
        let mut headers = HttpHeaders::new();
        while let Some(line) = lines.next() {
            if !line.contains(':') || line.len() <= 1 {
                break;
            }
            let (k, v) = line.split_once(":").unwrap();
            let k = k.trim().to_lowercase();
            let v = v.trim();
            headers.insert(&k, v);
        }
        if !headers.contains_key("host") {
            headers.insert("host", &host);
        }
        let mut body = Vec::new();
        while let Some(line) = lines.next() {
            body.extend_from_slice(line.as_bytes());
        }
        if !headers.contains_key("content-length") && body.is_empty() {
            headers.insert("content-length", &body.len().to_string());
        }
        let request = HttpRequest {
            host,
            ssl,
            method,
            path,
            args: HashMap::new(),
            headers,
            body,
        };

        Ok(request)
    }

    /// Returns a blank default request (empty method/path/headers).
    pub fn default() -> Self {
        HttpRequest {
            method: HttpMethod::Other(String::new()),
            ssl: false,
            path: String::new(),
            host: String::new(),
            args: HashMap::new(),
            headers: HttpHeaders::new(),
            body: Vec::new(),
        }
    }

    /// Returns a blank default request (empty method/path/headers).
    pub fn clone(&self) -> Self {
        HttpRequest {
            method: self.method.clone(),
            ssl: self.ssl,
            host: self.host.clone(),
            path: self.path.clone(),
            args: self.args.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
        }
    }

    /// Attempts to decode the body as UTF-8 and return it as text.
    pub fn text(&self) -> Option<String> {
        if self.body.is_empty() {
            return None;
        }
        match str::from_utf8(self.body.as_slice()) {
            Ok(v) => Some(v.to_string()),
            Err(_e) => None,
        }
    }
}

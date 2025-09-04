//! HTTP response types for HTeaPot.

use super::HttpStatus;
use std::collections::HashMap;

/// Basic HTTP status line + headers.
pub struct HttpResponse {
    pub status: HttpStatus,
    pub headers: HashMap<String, String>,
    pub content: Vec<u8>,
}

impl HttpResponse {
    pub fn new<B: AsRef<[u8]>>(
        status: HttpStatus,
        content: B,
        headers: Option<HashMap<String, String>>,
    ) -> Box<Self> {
        let content = content.as_ref();
        let headers = headers.unwrap_or_default();
        Box::new(Self {
            status,
            headers,
            content: content.to_owned(),
        })
    }
    /// Converts the status + headers into a properly formatted HTTP header block.
    pub fn to_bytes(&mut self) -> Vec<u8> {
        let mut headers_text = String::new();
        for (key, value) in self.headers.iter() {
            headers_text.push_str(&format!("{}: {}\r\n", key, value));
        }

        let response_header = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n",
            self.status as u16,
            self.status.to_string(),
            headers_text
        );

        let mut response = Vec::new();
        response.extend_from_slice(response_header.as_bytes());
        response
    }

    pub fn parse(raw: Vec<u8>) -> Result<HttpResponse, &'static str> {
        let mut status = HttpStatus::IAmATeapot;
        let mut headers = HashMap::new();
        let mut mode = 0; //0 -> status line, 1 -> headers, 2 -> body
        let mut index = 0;
        for raw_line in raw.split(|b| *b == b'\n') {
            index += raw_line.len() + 1;
            let line = str::from_utf8(raw_line).map_err(|_| "Error parsing line")?;
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.len() == 0 && mode == 1 {
                break;
            }
            match mode {
                0 => {
                    let raw_status = line.split(' ').skip(1).next().unwrap();
                    let raw_status = raw_status
                        .parse::<u16>()
                        .map_err(|_| "Error parsing status")?;
                    status = HttpStatus::from_u16(raw_status)?;
                    mode = 1;
                }
                1 => {
                    if !line.contains(':') {
                        continue;
                    }
                    let (k, v) = line.split_once(":").unwrap();
                    let k = k.trim().to_string();
                    let v = v.trim().to_string();
                    headers.insert(k, v);
                }
                _ => {}
            }
        }
        let mut content = Vec::new();
        content.extend_from_slice(&raw[index..raw.len()]);
        return Ok(HttpResponse {
            status,
            headers,
            content,
        });
    }
}

//! HTTP response types for HTeaPot.

use crate::http::HttpHeaders;

use super::HttpStatus;

/// Basic HTTP status line + headers.
pub struct HttpResponse {
    pub status: HttpStatus,
    pub headers: HttpHeaders,
    pub content: Vec<u8>,
}

impl HttpResponse {
    pub fn new<B: AsRef<[u8]>>(
        status: HttpStatus,
        content: B,
        headers: Option<HttpHeaders>,
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
}

use std::usize;

#[derive(Debug)]
pub enum State {
    Init,
    Headers,
    Body,
    Finish,
}

pub struct HttpResponseBuilder {
    pub body: Vec<u8>,
    pub headers: HttpHeaders,
    pub status: HttpStatus,
    buffer: Vec<u8>,
    pub state: State,
}

impl HttpResponseBuilder {
    pub fn new() -> HttpResponseBuilder {
        HttpResponseBuilder {
            status: HttpStatus::IAmATeapot,
            headers: HttpHeaders::new(),
            body: Vec::new(),
            buffer: Vec::new(),
            state: State::Init,
        }
    }

    pub fn get(&self) -> Option<HttpResponse> {
        match self.state {
            State::Finish => Some(HttpResponse {
                status: self.status,
                headers: self.headers.clone(),
                content: self.body.clone(),
            }),
            _ => None,
        }
    }

    pub fn append(&mut self, chunk: &[u8]) -> Result<bool, &'static str> {
        self.buffer.extend_from_slice(chunk);

        while !self.buffer.is_empty() {
            match self.state {
                State::Init => {
                    if let Some(line) = get_line(&mut self.buffer) {
                        let parts: Vec<&str> = line.split(" ").collect();
                        if parts.len() < 3 {
                            println!("parts: {:?}", parts);
                            return Err("Invalid response");
                        }
                        let status_str = parts.get(1).ok_or("Invalid status")?;
                        let status = status_str.parse::<u16>().map_err(|_| "Invalid status")?;
                        self.status = HttpStatus::from_u16(status).map_err(|_| "Invalid status")?;
                        self.state = State::Headers;
                    } else {
                        return Ok(false);
                    }
                }
                State::Headers => {
                    if let Some(line) = get_line(&mut self.buffer) {
                        if line.is_empty() {
                            self.state = State::Body;
                            continue;
                        }
                        let (k, v) = line.split_once(":").ok_or("Invalid header")?;
                        let k = k.trim().to_lowercase();
                        let v = v.trim();
                        self.headers.insert(&k, v);
                    } else {
                        return Ok(false);
                    }
                }
                State::Body => {
                    self.body.extend_from_slice(&mut self.buffer.as_slice());
                    self.buffer.clear();
                    if let Some(content_length) = self.headers.get("content-length") {
                        let content_length = content_length
                            .parse::<usize>()
                            .map_err(|_| "invalid content-length")?;
                        if self.body.len() >= content_length {
                            self.state = State::Finish;
                            return Ok(true);
                        } else {
                            return Ok(false);
                        }
                    } else {
                        //TODO: handle chunked
                        self.state = State::Finish;
                        return Ok(true);
                    }
                }
                State::Finish => {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

fn get_line(buffer: &mut Vec<u8>) -> Option<String> {
    if let Some(pos) = buffer.windows(2).position(|w| w == b"\r\n") {
        let line = buffer.drain(..pos).collect::<Vec<u8>>();
        buffer.drain(..2); // remove CRLF
        return match str::from_utf8(line.as_slice()) {
            Ok(v) => Some(v.to_string()),
            Err(_e) => None,
        };
    }
    None
}

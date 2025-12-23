//! HTTP response types for HTeaPot.

use crate::http::HttpHeaders;

use super::HttpStatus;

/// Basic HTTP status line + headers.
pub struct HttpResponse {
    pub status: HttpStatus,
    pub headers: HttpHeaders,
    pub content: Vec<u8>,
}

use std::cmp::min;

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
    chunked: bool,
    length: usize,
}

impl HttpResponseBuilder {
    pub fn new() -> HttpResponseBuilder {
        HttpResponseBuilder {
            status: HttpStatus::IAmATeapot,
            headers: HttpHeaders::new(),
            body: Vec::new(),
            buffer: Vec::new(),
            state: State::Init,
            length: 0,
            chunked: false,
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
                        let parts: Vec<&str> = line.split_whitespace().collect();
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
                        if k == "content-length" {
                            self.length = v.parse().unwrap_or(0);
                        }
                        if k == "transfer-encoding" && v.to_lowercase() == "chunked" {
                            self.chunked = true;
                            self.length = v.parse().unwrap_or(0);
                        }
                        self.headers.insert(&k, v);
                    } else {
                        return Ok(false);
                    }
                }
                State::Body => {
                    let body_left = self.length - self.body.len();
                    if body_left > 0 {
                        let to_take = min(body_left, self.buffer.len());
                        let to_append = self.buffer.drain(..to_take);
                        let to_append = to_append.as_slice();
                        self.body.extend_from_slice(to_append);
                        if to_append.len() < body_left {
                            return Ok(false);
                        }
                    }

                    if self.chunked {
                        let size = get_line(&mut self.buffer);
                        if size.is_none() {
                            self.length = 0;
                            return Ok(false);
                        }
                        let size = size.unwrap();
                        if size.is_empty() {
                            continue;
                        }
                        let size = size.strip_prefix("0x").unwrap_or(&size);
                        let size =
                            i64::from_str_radix(size, 16).map_err(|_| "Invalud chunk size")?;
                        if size == 0 {
                            self.state = State::Finish;
                            return Ok(true);
                        }

                        self.length += size as usize;
                    }

                    if self.body.len() >= self.length {
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

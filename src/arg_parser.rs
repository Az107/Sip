use std::{collections::HashMap, env};

use crate::http::{HttpMethod, HttpRequest};

#[derive(Debug)]
pub enum ArgKind {
    Flag,
    Value(String),
}

#[derive(Debug)]
pub struct RequestArgs {
    pub method: String,
    pub url: String,
    pub host: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub args: HashMap<String, ArgKind>,
}

impl RequestArgs {
    fn new() -> RequestArgs {
        RequestArgs {
            method: String::new(),
            url: String::new(),
            host: String::new(),
            body: String::new(),
            headers: HashMap::new(),
            args: HashMap::new(),
        }
    }

    pub fn to_request(&self) -> Result<HttpRequest, &'static str> {
        let method = HttpMethod::from_str(&self.method);
        let mut ssl = false;
        if self.url.starts_with("https://") {
            ssl = true;
        }
        let path = &self
            .url
            .strip_prefix("http://")
            .unwrap_or(self.url.strip_prefix("https://").unwrap_or(&self.url));
        let (host, path) = path.split_once('/').unwrap_or((path, ""));
        let host = host.to_string();
        let mut path = path.to_string();
        path.insert(0, '/');
        let mut request = HttpRequest::new(method, &host, &path);
        request.headers.insert("host", &host);
        request.ssl = ssl;
        request.body = self.body.as_bytes().to_vec();
        for (key, value) in self.headers.iter() {
            request.headers.insert(key, value);
        }

        Ok(request)
    }

    pub fn get(&self, key: &str) -> Option<String> {
        if let Some(value) = self.args.get(key) {
            match value {
                ArgKind::Flag => None,
                ArgKind::Value(v) => Some(v.clone()),
            }
        } else {
            None
        }
    }

    fn get_state(&self) -> State {
        if self.method.is_empty() {
            State::Method
        } else if self.url.is_empty() {
            State::Url
        } else {
            State::Body
        }
    }
}

fn host_from_url(url: &str) -> String {
    //remove the protocol from the url
    let url = url.strip_prefix("http://").unwrap_or(url);
    let url = url.strip_prefix("https://").unwrap_or(url);
    //remove the path from the url
    let url = url.split('/').next().unwrap();
    //remove the port from the url
    url.split(':').next().unwrap_or(url).to_string()
}

enum State {
    Method,
    Url,
    Body,
    Arg(String),
}

pub fn args_parser() -> RequestArgs {
    let mut request_args = RequestArgs::new();
    let mut state = State::Method;
    for arg in env::args().skip(1) {
        if arg.starts_with("-") {
            let key = arg.strip_prefix("-").unwrap().to_string();
            if key.starts_with("-") {
                let key = key.strip_prefix("-").unwrap_or(&key).to_string();
                state = State::Arg(key);
            } else {
                request_args.args.insert(key, ArgKind::Flag);
            }
            continue;
        }

        match state {
            State::Method => {
                request_args.method = arg;
                state = State::Url;
            }
            State::Url => {
                request_args.url = arg;
                request_args.host = host_from_url(&request_args.url);

                state = State::Body;
            }
            State::Body => {
                if arg.contains(":") {
                    let header_kv = arg.split_once(":");
                    if let Some((k, v)) = header_kv {
                        request_args.headers.insert(k.to_string(), v.to_string());
                        continue;
                    }
                }
                request_args.body.push_str(&arg);
            }
            State::Arg(k) => {
                request_args.args.insert(k, ArgKind::Value(arg));
                state = request_args.get_state();
            }
        };
    }
    request_args
}

// if arg.starts_with("-") {
//     k_arg = arg.strip_prefix("-").unwrap().to_string();
//     k_arg = k_arg.strip_prefix("-").unwrap_or(&k_arg).to_string();
// } else if !k_arg.is_empty() {
//     args.insert(k_arg.clone(), arg.clone());
//     k_arg = String::new();
// } else {
//     if !input.is_empty() && arg.contains('=') {
//         if body.is_empty() {
//             body.push('{');
//         }
//         let (k, v) = arg.split_once('=').unwrap();
//         body.push_str(&format!("\"{}\":\"{}\"", k, v));
//         body.push(',');
//         continue;
//     }
//     input.push_str(&arg);
// }

// TEST
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_from_url() {
        assert_eq!(host_from_url("http://example.com:8080/path"), "example.com");
        assert_eq!(host_from_url("https://example.com/path"), "example.com");
        assert_eq!(host_from_url("example.com:8080/path"), "example.com");
        assert_eq!(host_from_url("example.com/path"), "example.com");
        assert_eq!(host_from_url("example.com"), "example.com");
    }
}

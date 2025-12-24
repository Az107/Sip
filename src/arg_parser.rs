use std::{collections::HashMap, env};

use crate::http::{HttpMethod, HttpRequest};

#[derive(Debug)]
pub struct RequestArgs {
    pub method: String,
    pub url: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub args: HashMap<String, String>,
}

impl RequestArgs {
    fn new() -> RequestArgs {
        RequestArgs {
            method: String::new(),
            url: String::new(),
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
        request.ssl = ssl;
        request.body = self.body.as_bytes().to_vec();
        for (key, value) in self.headers.iter() {
            request.headers.insert(key, value);
        }

        Ok(request)
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
            let key = key.strip_prefix("-").unwrap_or(&key).to_string();
            state = State::Arg(key);
            continue;
        }

        match state {
            State::Method => {
                request_args.method = arg;
                state = State::Url;
            }
            State::Url => {
                request_args.url = arg;
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
                request_args.args.insert(k, arg);
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

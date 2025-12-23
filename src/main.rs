mod http;
mod utils;

use std::{collections::HashMap, env};
use utils::{print_response, save_file};

use http::{HttpRequest, HttpResponse};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "config_file")]
fn parse_http_file(content: String) -> Vec<(String, HttpRequest)> {
    let mut result: Vec<(String, HttpRequest)> = Vec::new();
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut raw_request = "".to_string();

    for line in content.split("\n") {
        if line.starts_with("###") && !raw_request.is_empty() {
            for (k, v) in vars.iter() {
                let k = format!("{{{}}}", k);
                raw_request = raw_request.replace(&k, v);
            }
            let request = HttpRequest::parse(raw_request.clone());
            if let Ok(request) = request {
                result.push(("".to_string(), request));
            }
            raw_request = String::new();
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        if line.starts_with("@") {
            let line = line.strip_prefix('@').unwrap();
            let (k, v) = line.split_once("=").unwrap();
            let k = k.trim().to_string();
            let v = v.trim().to_string();
            vars.insert(k, v);
            continue;
        }
        raw_request.push_str(line);
        raw_request.push('\n');
    }
    result
}

fn args_parser() -> (String, String, HashMap<String, String>) {
    let mut input = String::new();
    let mut body = String::new();
    let mut args: HashMap<String, String> = HashMap::new();
    let mut skip = 1;
    let mut k_arg = String::new();
    for arg in env::args().skip(1) {
        if arg.starts_with("-") {
            k_arg = arg.strip_prefix("-").unwrap().to_string();
            k_arg = k_arg.strip_prefix("-").unwrap_or(&k_arg).to_string();
            skip += 1;
        } else if !k_arg.is_empty() {
            args.insert(k_arg.clone(), arg.clone());
            k_arg = String::new();
            skip += 1;
        } else {
            break;
        }
    }
    // let mut body = String::new();
    for arg in env::args().skip(skip) {
        let is_empty = input.is_empty();

        if !is_empty && arg.contains('=') {
            if body.is_empty() {
                body.push('{');
            }
            let (k, v) = arg.split_once('=').unwrap();
            body.push_str(&format!("\"{}\":\"{}\"", k, v));
            body.push(',');
            continue;
        }
        input.push_str(&arg);
        if !is_empty {
            input.push('\n');
        } else {
            input.push(' ');
        }
    }
    if body.starts_with('{') && !body.ends_with('}') {
        body.pop();
        body.push('}');
    }

    (input, body, args)
}

fn main() {
    let (input, body, args) = args_parser();
    let request = HttpRequest::parse(input);
    if request.is_err() {
        println!("Error: {}", request.err().unwrap());
        return;
    }
    let mut request = request.unwrap();
    request
        .headers
        .insert("User-Agent", &format!("Sip/{}", VERSION));
    println!("{}", body);
    request.body = body.as_bytes().to_vec();
    if !body.is_empty() {
        request
            .headers
            .insert("Content-Legth", body.len().to_string().as_str());
    }

    let response = request.brew();
    if let Ok(response) = response {
        print_response(&response);
        if let Some(file) = args.get("O") {
            save_file(file, response);
        };
    } else {
        println!("{:?}", response.err())
    }
}

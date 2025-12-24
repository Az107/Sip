mod arg_parser;
mod http;
mod utils;

use arg_parser::args_parser;
use std::env;
use utils::{print_response, save_file};

use http::HttpRequest;

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

fn main() {
    let args = args_parser();
    println!("{:?}", args);
    let request = args.to_request();

    if request.is_err() {
        println!("Error: {}", request.err().unwrap());
        return;
    }
    let mut request = request.unwrap();
    request
        .headers
        .insert("User-Agent", &format!("Sip/{}", VERSION));
    if !request.body.is_empty() {
        request
            .headers
            .insert("Content-Legth", request.body.len().to_string().as_str());
    }

    let response = request.brew();
    if let Ok(response) = response {
        print_response(&response);
        if let Some(file) = args.args.get("O") {
            save_file(file, response);
        };
    } else {
        println!("{:?}", response.err())
    }
}

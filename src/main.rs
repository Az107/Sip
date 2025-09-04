mod http;
use std::{collections::HashMap, env};

use http::HttpRequest;

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
            if request.is_ok() {
                result.push(("".to_string(), request.unwrap()));
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
    return result;
}

fn main() {
    let mut input = String::new();
    // let mut body = String::new();
    for arg in env::args().skip(1) {
        println!("{}", arg);
        // if !input.is_empty() &&
        input.push_str(&arg);
        input.push('\n');
    }
    let request = HttpRequest::parse(input);
    if request.is_err() {
        println!("Error: {}", request.err().unwrap());
        return;
    }
    let request = request.unwrap();
    // request
    //     .headers
    //     .insert("host".to_string(), "example.org".to_string());

    let response = request.brew();
    if response.is_err() {
        println!("{:?}", response.err())
    } else {
        let response = response.unwrap();
        println!("{:?}", response.status.to_string());
        println!(
            "{:?}",
            str::from_utf8(response.content.as_slice()).unwrap_or("Error priting body")
        );
    }
}

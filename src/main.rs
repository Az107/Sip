mod http;

use std::{collections::HashMap, env, fs::File, io::Write, path::Path};

use http::{HttpRequest, HttpResponse};

use crate::http::HttpStatus;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "render_body")]
fn render_body(response: HttpResponse) {
    use html2text::config;
    use serde_json::{Value, to_string_pretty};
    if response.headers.contains_key("content-type") {
        let content_type = response.headers.get("content-type").unwrap();
        let content_type = content_type
            .split_once(';')
            .unwrap_or((content_type, content_type))
            .0;
        match content_type {
            "application/json" => {
                if let Ok(raw) = str::from_utf8(response.content.as_slice()) {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
                        let text = serde_json::to_string_pretty(&value).unwrap();
                        println!("{}", text);
                    } else {
                        println!("{}", raw);
                    }
                } else {
                    println!("Error priting body");
                };
            }
            "text/html" => {
                if let Ok(raw) = str::from_utf8(response.content.as_slice()) {
                    if let Ok(value) = config::rich()
                        .use_doc_css()
                        .string_from_read(raw.as_bytes(), 80)
                    {
                        println!("{}", value);
                    } else {
                        println!("{}", raw);
                    }
                } else {
                    println!("Error priting body");
                };
            }
            _ => {
                if response.content.len() > 1024 {
                    println!("<Binary {}>", response.content.len())
                } else {
                    println!(
                        "{}",
                        str::from_utf8(response.content.as_slice()).unwrap_or("Error priting body")
                    );
                }
            }
        }
    } else {
        if response.content.len() > 1024 {
            println!("<Binary {}>", response.content.len())
        } else {
            println!(
                "{}",
                str::from_utf8(response.content.as_slice()).unwrap_or("Error priting body")
            );
        }
    }
}

#[cfg(not(feature = "render_body"))]
fn render_body(response: &HttpResponse) {
    if response.content.len() > 1024 * 10 {
        println!("<Binary {}>", response.content.len())
    } else {
        println!(
            "{}",
            str::from_utf8(response.content.as_slice()).unwrap_or("Error priting body")
        );
    }
}

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
    let mut body = String::new();
    let mut args: HashMap<String, String> = HashMap::new();
    let mut skip = 1;
    let mut k_arg = String::new();
    for arg in env::args().skip(1) {
        if arg.starts_with("-") {
            k_arg = arg.strip_prefix("-").unwrap().to_string();
            k_arg = k_arg.strip_prefix("-").unwrap_or(&k_arg).to_string();
            skip += 1;
        } else {
            if !k_arg.is_empty() {
                args.insert(k_arg.clone(), arg.clone());
                k_arg = String::new();
                skip += 1;
            } else {
                break;
            }
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

    if !body.is_empty() {
        input.push_str("\r\n");
        input.push_str(&body);
    }

    let request = HttpRequest::parse(input);
    if request.is_err() {
        println!("Error: {}", request.err().unwrap());
        return;
    }
    let mut request = request.unwrap();
    request
        .headers
        .insert("User-Agent", &format!("Sip/{}", VERSION));
    // request
    //     .headers
    //     .insert("host".to_string(), "example.org".to_string());

    let response = request.brew();
    if response.is_err() {
        println!("{:?}", response.err())
    } else {
        let response = response.unwrap();
        println!("{:?}", response.status.to_string());
        for header in response.headers.iter() {
            println!("- {}: {}", header.0, header.1);
        }
        println!("\n");
        render_body(&response);
        if response.status.is_ok()
            && let Some(file) = args.get("O")
        {
            let path = Path::new(file);
            let mut file = if path.exists() {
                File::open(path)
            } else {
                File::create(path)
            }
            .unwrap();

            let _ = file.write_all(&response.content.as_slice());
        };
    }
}

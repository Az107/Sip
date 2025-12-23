use std::{fs::File, io::Write, path::Path};

use crate::http::HttpResponse;

pub fn save_file(file: &str, response: HttpResponse) {
    let path = Path::new(file);
    let mut file = if path.exists() {
        File::open(path)
    } else {
        File::create(path)
    }
    .unwrap();

    let _ = file.write_all(response.content.as_slice());
}

#[cfg(feature = "render_body")]
fn render_body(response: &HttpResponse) {
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
    if response.content.len() > 1024 * 100 {
        println!("<Binary {}>", response.content.len())
    } else {
        println!(
            "{}",
            str::from_utf8(response.content.as_slice()).unwrap_or("Error priting body")
        );
    }
}

pub fn print_response(response: &HttpResponse) {
    println!("{} {}", response.status.as_num(), response.status.as_str());
    for header in response.headers.iter() {
        println!("- {}: {}", header.0, header.1);
    }
    println!("\n");
    render_body(response);
}

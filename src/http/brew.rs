// Written by Alberto Ruiz 2024-04-08
//
// This module provides basic HTTP client functionality. It defines
// methods to compose and send HTTP requests and parse the resulting
// responses using a `TcpStream`.

use std::fmt::{self, Display, Formatter};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use native_tls::TlsConnector;

use super::request::HttpRequest;
use super::response::{HttpResponse, HttpResponseBuilder};

// use super::status::HttpStatus;
// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
//
trait StreamRW: Read + Write {}
impl<T: Read + Write> StreamRW for T {}

impl Display for HttpRequest {
    /// Converts the request into a raw HTTP/1.1-compliant string.
    ///
    /// This includes method, path with optional query args, headers, and optional body.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Add query parameters to the path if needed
        let path = if self.args.is_empty() {
            self.path.clone()
        } else {
            let mut path = format!("{}?", self.path);
            for (k, v) in &self.args {
                path.push_str(&format!("{}={}&", k, v));
            }
            if path.ends_with('&') {
                path.pop();
            }
            path
        };

        let path = if path.is_empty() {
            "/".to_string()
        } else {
            path
        };

        write!(f, "{} {} HTTP/1.1\r\n", self.method.to_str(), path)?;
        for (k, v) in &self.headers {
            write!(f, "{}: {}\r\n", k, v)?;
        }
        write!(f, "\r\n")?;
        write!(f, "{}", &self.text().unwrap_or_default())?;
        write!(f, "\r\n")
    }
}

impl HttpRequest {
    /// Sends the request to a remote server and returns a parsed response.
    ///
    /// Supports only `http://` (not `https://`). Attempts to resolve the domain
    /// and open a TCP connection. Times out after 5 seconds.
    pub fn brew(&self) -> Result<HttpResponse, &'static str> {
        let mut addr = self.host.clone();
        let mut ssl = self.ssl;
        // Strip protocol prefix
        if let Some(stripped) = addr.strip_prefix("http://") {
            addr = stripped.to_string();
        } else if addr.starts_with("https://") {
            ssl = true;
        }

        // Add port if missing
        if !addr.contains(':') {
            if ssl {
                addr.push_str(":443");
            } else {
                addr.push_str(":80");
            }
        }

        let addr = addr.split("/").next().unwrap();
        // Resolve address
        let addr = if addr.starts_with("localhost") {
            addr.replace("localhost", "127.0.0.1").to_string()
        } else {
            addr.to_string()
        };

        let resolved_addrs: Vec<_> = addr
            .to_socket_addrs()
            .map_err(|_| "Unable to resolve domain")?
            .collect();

        let socket_addr = resolved_addrs
            .into_iter()
            .find(|addr| addr.port() != 0 && !addr.ip().is_unspecified())
            .ok_or("No valid address found")?;
        if socket_addr.port() == 443 {
            ssl = true;
        }
        // Connect to server
        let stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))
            .map_err(|_| "Error connecting to server")?;
        let _ = stream.set_read_timeout(Some(Duration::from_secs(20)));
        let mut stream: Box<dyn StreamRW> = if ssl {
            // 1. Crear el conector TLS
            let connector = TlsConnector::new().map_err(|_| "SLL error")?;
            let (hostname, _) = self
                .host
                .split_once(':')
                .unwrap_or((&self.host, &self.host));
            let stream = connector.connect(hostname, stream).map_err(|e| {
                println!("host: {} \n error: {:?}", hostname, e);
                "SLL error"
            })?;
            Box::new(stream)
        } else {
            Box::new(stream)
        };

        let _ = stream.write_all(self.to_string().as_bytes());
        let _ = stream.flush();
        let mut builder = HttpResponseBuilder::new();
        let mut buffer = [0u8; 4096];
        let mut zero_counter = 10;

        loop {
            if zero_counter == 0 {
                return Err("Connexion closed");
            }
            match stream.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        zero_counter -= 1;
                    } else {
                        zero_counter = 10;
                    }
                    let r = builder.append(&buffer[..n])?;
                    if r {
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    println!("Read timeout");
                    println!("state: {:?}", builder.state);
                    println!("Headers: {:?}", builder.headers);
                    println!("body_len: {:?}", builder.body.len());
                    break;
                }
                Err(e) => {
                    println!("{:?}", e);
                    println!("state: {:?}", builder.state);
                    println!("body_len: {:?}", builder.body.len());
                    return Err("Error reading");
                }
            }
        }

        Ok(builder.get().unwrap())
    }
}

// pub fn brew_url(url: &str) -> Result<HttpResponse, &'static str> {
//     todo!()
// }

#[cfg(test)]
mod tests {
    use super::super::methods::HttpMethod;
    use super::*;
    #[test]
    fn test_http_request_new() {
        let request = HttpRequest::new(HttpMethod::Get, "localhost", "/example");
        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.path, "/example");
        assert!(request.args.is_empty());
        assert!(request.headers.is_empty());
        assert_eq!(request.text(), None);
    }

    #[test]
    fn test_http_request_arg() {
        let mut request = HttpRequest::new(HttpMethod::Post, "localhost", "/submit");
        request.args.insert("key".to_string(), "value".to_string());
        assert_eq!(request.args.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_http_request_header() {
        let mut request = HttpRequest::new(HttpMethod::Get, "localhost", "/data");
        request.headers.insert("Content-Type", "application/json");
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    // #[test]
    // fn test_http_request_body() {
    //     let mut request = HttpRequest::new(HttpMethod::POST, "/upload");
    //     request.body("Test body content".to_string());
    //     assert_eq!(request.body, "Test body content");
    // }

    #[test]
    fn test_http_request_to_string() {
        let mut request = HttpRequest::new(HttpMethod::Post, "localhost", "/resource");
        request.headers.insert("Content-Type", "application/json");
        //.body("{\"data\":\"test\"}".to_string());

        let request_string = request.to_string();
        assert!(request_string.contains("POST /resource HTTP/1.1"));
        assert!(request_string.contains("Content-Type: application/json"));
        //assert!(request_string.contains("{\"data\":\"test\"}"));
    }

    // #[test]
    // fn test_http_request_to_string_with_args() {
    //     let mut request = HttpRequest::new(HttpMethod::POST, "/resource");
    //     let _ = request
    //         .header("Content-Type", "application/json")
    //         .arg("key", "value")
    //         .body("{\"data\":\"test\"}".to_string())
    //         .brew("localhost:8080");

    //     let request_string = request.to_string();
    //     assert!(request_string.contains("POST /resource?key=value HTTP/1.1"));
    //     assert!(request_string.contains("Content-Type: application/json"));
    //     assert!(request_string.contains("{\"data\":\"test\"}"));
    // }

    #[test]
    fn test_http_request() {
        let r = HttpRequest::new(HttpMethod::Get, "example.org:80", "/").brew();
        assert!(r.is_ok());
    }

    #[test]
    fn test_http_request_time_out() {
        let r = HttpRequest::new(HttpMethod::Get, "example.org:8080", "/").brew();
        assert!(r.is_err());
    }
}

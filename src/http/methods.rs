/// Represents an HTTP method (verb).
///
/// Includes standard HTTP/1.1 methods such as `GET`, `POST`, `PUT`, etc.,
/// and a catch-all variant `Other(String)` for unknown or non-standard methods.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Trace,
    Connect,
    Other(String),
}
impl HttpMethod {
    /// Creates an `HttpMethod` from a raw string (case-sensitive).
    ///
    /// If the method is not one of the standard HTTP methods,
    /// it will be returned as `HttpMethod::Other(method.to_string())`.
    ///
    /// # Examples
    /// ```
    /// use hteapot::HttpMethod;
    ///
    /// let m = HttpMethod::from_str("GET");
    /// assert_eq!(m, HttpMethod::GET);
    ///
    /// let custom = HttpMethod::from_str("CUSTOM");
    /// assert_eq!(custom, HttpMethod::Other("CUSTOM".into()));
    /// ```
    pub fn from_str(method: &str) -> HttpMethod {
        let method = method.to_uppercase();
        match method.as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            "TRACE" => HttpMethod::Trace,
            "CONNECT" => HttpMethod::Connect,
            _ => Self::Other(method.to_string()),
        }
    }

    /// Returns the string representation of the HTTP method.
    ///
    /// If the method is non-standard (`Other`), it returns the inner string as-is.
    ///
    /// # Examples
    /// ```
    /// use hteapot::HttpMethod;
    ///
    /// let method = HttpMethod::GET;
    /// assert_eq!(method.to_str(), "GET");
    /// ```
    pub fn to_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Other(method) => method.as_str(),
        }
    }
}

// #[derive(Clone, Copy)]
// pub enum Protocol {
//     HTTP,
//     HTTPS,
// }

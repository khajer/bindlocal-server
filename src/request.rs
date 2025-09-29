pub struct HttpRequest {}

impl HttpRequest {
    pub fn get_subdomain(request: &str) -> String {
        for line in request.lines() {
            if line.to_lowercase().starts_with("host:") {
                let host = line.splitn(2, ':').nth(1).unwrap_or("").trim();

                if let Some((subdomain, _rest)) = host.split_once('.') {
                    return subdomain.to_string();
                }
            }
        }
        "".to_string()
    }
    pub fn parse_content_length(headers: String) -> Option<usize> {
        for line in headers.lines() {
            if line.to_lowercase().starts_with("content-length:") {
                if let Some(value) = line.split(':').nth(1) {
                    if let Ok(length) = value.trim().parse::<usize>() {
                        return Some(length);
                    }
                }
            }
        }
        None
    }
    pub fn parse_connection(headers: String) -> Option<String> {
        for line in headers.lines() {
            if line.to_lowercase().starts_with("connection:") {
                if let Some(value) = line.split(':').nth(1) {
                    return Some(value.to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_request() {
        let result = HttpRequest::get_subdomain("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_non_empty_request() {
        let request = "GET / HTTP/1.1\r\nHost: test.example.com\r\n\r\n";
        let result = HttpRequest::get_subdomain(request);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_parse_content_length() {
        let headers = "Content-Length: 123\r\n".to_string();
        let result = HttpRequest::parse_content_length(headers);
        assert_eq!(result, Some(123));
    }

    #[test]
    fn test_parse_content_length_invalid() {
        let headers = "Content-Length: abc\r\n".to_string();
        let result = HttpRequest::parse_content_length(headers);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_content_length_empty() {
        let headers = "Content-Length:\r\n".to_string();
        let result = HttpRequest::parse_content_length(headers);
        assert_eq!(result, None);
    }
    #[test]
    fn test_parse_connection() {
        let headers = "Connection: keep-alive\r\n".to_string();
        let result = HttpRequest::parse_connection(headers);
        assert_eq!(result, Some("keep-alive".to_string()));
    }
    #[test]
    fn test_parse_connection_empty() {
        let headers = "Connection:\r\n".to_string();
        let result = HttpRequest::parse_connection(headers);
        assert_eq!(result, None);
    }
}

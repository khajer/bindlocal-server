pub struct HttpRequest {
    // pub sub_domain: String,
}

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
}

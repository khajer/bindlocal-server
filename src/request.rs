use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    pub fn parse(request: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = request.lines().collect();

        if lines.is_empty() {
            return Err("Empty request".into());
        }

        // Parse request line (GET /path HTTP/1.1)
        let request_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line_parts.len() < 3 {
            return Err("Invalid request line".into());
        }

        let method = request_line_parts[0].to_string();
        let path = request_line_parts[1].to_string();

        // Parse headers
        let mut headers = HashMap::new();
        let mut i = 1;

        while i < lines.len() && !lines[i].is_empty() {
            if let Some((key, value)) = lines[i].split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
            i += 1;
        }

        // Parse body (everything after empty line)
        let body = if i + 1 < lines.len() {
            lines[i + 1..].join("\n")
        } else {
            String::new()
        };

        Ok(HttpRequest {
            method,
            path,
            headers,
            body,
        })
    }
}

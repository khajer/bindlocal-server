pub struct HttpResponse {
    status_code: u16,
    status_text: String,
    content_type: String,
    body: String,
}

impl HttpResponse {
    pub fn new(status_code: u16, status_text: &str, content_type: &str, body: &str) -> Self {
        Self {
            status_code,
            status_text: status_text.to_string(),
            content_type: content_type.to_string(),
            body: body.to_string(),
        }
    }

    pub fn service_unavailable() -> Self {
        let body = "service unavailable";
        Self::new(503, "error", "text/plain", body)
    }

    pub fn not_found() -> Self {
        let body = r#"<!DOCTYPE html>
<html>
<head>
    <title>404 Not Found</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; text-align: center; }
        h1 { color: #d32f2f; }
    </style>
</head>
<body>
    <h1>404 - Not Found</h1>
    <p>The requested resource was not found on this server.</p>
    <a href="/">← Back to home</a>
</body>
</html>"#;
        Self::new(404, "Not Found", "text/html", body)
    }

    pub fn client_app_call_local_refused() -> Self {
        let body = r#"<!DOCTYPE html>
<html>
<head>
    <title>503 Service Unavailable , The Client Application Cannot Connect to the Local Server</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; text-align: center; }
        h1 { color: #d32f2f; }
    </style>
</head>
<body>
    <h1>503 Service Unavailable</h1>
    <p>The Client Application Cannot Connect to the Local Server</p>
    <a href="/">← Back to home</a>
</body>
</html>"#;
        Self::new(503, "Service Unavailable", "text/html", body)
    }

    pub fn to_string(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             Server: Tokio-HTTP/1.0\r\n\
             \r\n\
             {}",
            self.status_code,
            self.status_text,
            self.content_type,
            self.body.len(),
            self.body
        )
    }
}

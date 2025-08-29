use crate::request::HttpRequest;
use crate::response::HttpResponse;

pub async fn route_request(request: &HttpRequest) -> String {
    let response = match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => handle_home().await,
        ("GET", "/hello") => handle_hello().await,
        ("GET", "/json") => handle_json().await,
        ("POST", "/echo") => handle_echo(request).await,
        ("GET", "/status") => handle_status().await,
        _ => handle_not_found().await,
    };

    response.to_string()
}

async fn handle_home() -> HttpResponse {
    let body = r#"<!DOCTYPE html>
<html>
<head>
    <title>Tokio HTTP Server</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .endpoint { background: #f0f0f0; padding: 10px; margin: 10px 0; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>Welcome to Tokio HTTP Server!</h1>
    <p>This server is built with Tokio and raw TCP sockets.</p>

    <h2>Available endpoints:</h2>
    <div class="endpoint"><strong>GET /</strong> - This page</div>
    <div class="endpoint"><strong>GET /hello</strong> - Hello world response</div>
    <div class="endpoint"><strong>GET /json</strong> - JSON response</div>
    <div class="endpoint"><strong>POST /echo</strong> - Echo back request body</div>
    <div class="endpoint"><strong>GET /status</strong> - Server status</div>
</body>
</html>"#;

    HttpResponse::ok_html(body)
}

async fn handle_hello() -> HttpResponse {
    HttpResponse::ok_text("Hello, World from Tokio HTTP Server!")
}

async fn handle_json() -> HttpResponse {
    let json_body =
        r#"{"message": "Hello from JSON endpoint", "server": "Tokio HTTP", "status": "running"}"#;
    HttpResponse::ok_json(json_body)
}

async fn handle_echo(request: &HttpRequest) -> HttpResponse {
    let response_body = format!("Echo: {}", request.body);
    HttpResponse::ok_text(&response_body)
}

async fn handle_status() -> HttpResponse {
    let status_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Server Status</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .status { color: green; font-weight: bold; }
        .info { background: #e8f4fd; padding: 15px; border-radius: 5px; margin: 10px 0; }
    </style>
</head>
<body>
    <h1>Server Status</h1>
    <p class="status">✓ Server is running</p>
    <div class="info">
        <h3>Server Information:</h3>
        <p><strong>Framework:</strong> Tokio async runtime</p>
        <p><strong>Transport:</strong> Raw TCP sockets</p>
        <p><strong>Protocol:</strong> HTTP/1.1</p>
        <p><strong>Address:</strong> 127.0.0.1:8080</p>
    </div>
</body>
</html>"#;

    HttpResponse::ok_html(status_html)
}

async fn handle_not_found() -> HttpResponse {
    HttpResponse::not_found()
}

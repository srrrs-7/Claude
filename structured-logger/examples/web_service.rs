use log::{debug, error, info, LevelFilter};
use std::time::{Duration, Instant};
use structured_logger::{init_with_config, logger::LogContext, LoggerConfig, OutputFormat};

// Mock structures to simulate a web service
struct Request {
    id: String,
    method: String,
    path: String,
    ip: String,
    user_agent: String,
}

struct Response {
    status: u16,
    duration_ms: u64,
}

struct User {
    id: String,
    role: String,
}

// Example web service handler
fn handle_request(req: Request) -> Result<Response, String> {
    let start = Instant::now();

    // Create request context for logging
    let ctx = LogContext::new()
        .with_str("request_id", &req.id)
        .with_str("method", &req.method)
        .with_str("path", &req.path)
        .with_str("ip", &req.ip);

    ctx.info("Request received");

    // Authentication (simulated)
    debug!("Authenticating request request_id={}", req.id);

    let user = authenticate(&req)?;

    let auth_ctx = LogContext::new()
        .with_str("request_id", &req.id)
        .with_str("user_id", &user.id)
        .with_str("role", &user.role);

    auth_ctx.info("User authenticated");

    // Business logic (simulated)
    if req.path == "/api/users" {
        if req.method == "GET" {
            // Simulate database operation
            let db_ctx = LogContext::new()
                .with_str("request_id", &req.id)
                .with_str("user_id", &user.id)
                .with_str("operation", "query")
                .with_str("table", "users");

            db_ctx.debug("Executing database query");

            // Simulate slow query
            std::thread::sleep(Duration::from_millis(200));

            if req.path.contains("?limit=high") {
                db_ctx.warn("Large result set requested rows=5000");
            }

            db_ctx.debug("Database query completed");
        } else if req.method == "POST" {
            // Simulate user creation
            let db_ctx = LogContext::new()
                .with_str("request_id", &req.id)
                .with_str("user_id", &user.id)
                .with_str("operation", "insert")
                .with_str("table", "users");

            db_ctx.debug("Creating new user");

            // Simulate operation
            std::thread::sleep(Duration::from_millis(150));

            db_ctx.info("User created");
        }
    } else if req.path == "/api/error" {
        // Simulate error condition
        let err_ctx = LogContext::new()
            .with_str("request_id", &req.id)
            .with_str("user_id", &user.id)
            .with_str("path", &req.path);

        err_ctx.error("Internal server error");

        return Err("Internal server error".to_string());
    }

    // Calculate duration
    let duration = start.elapsed();

    // Log response
    let resp_ctx = LogContext::new()
        .with_str("request_id", &req.id)
        .with_number("status", 200)
        .with_number("duration_ms", duration.as_millis() as f64);

    resp_ctx.info("Request completed");

    Ok(Response {
        status: 200,
        duration_ms: duration.as_millis() as u64,
    })
}

// Authentication simulation
fn authenticate(req: &Request) -> Result<User, String> {
    // Simulate authentication logic
    std::thread::sleep(Duration::from_millis(50));

    if req.path == "/api/error" && req.method == "POST" {
        let auth_ctx = LogContext::new()
            .with_str("request_id", &req.id)
            .with_str("ip", &req.ip)
            .with_str("path", &req.path);

        auth_ctx.error("Authentication failed");
        return Err("Authentication failed".to_string());
    }

    Ok(User {
        id: "user-123".to_string(),
        role: if req.path.contains("admin") {
            "admin"
        } else {
            "user"
        }
        .to_string(),
    })
}

fn main() {
    // Initialize logger with JSON format for production
    let config = LoggerConfig::new()
        .with_level(LevelFilter::Info)
        .with_format(OutputFormat::Json)
        .with_metadata("service", "user-api")
        .with_metadata("env", "production")
        .with_metadata("version", "1.2.0");

    init_with_config(config).expect("Failed to initialize logger");

    info!("Service started");

    // Simulate a few requests
    let requests = vec![
        Request {
            id: "req-001".to_string(),
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            ip: "192.168.1.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
        },
        Request {
            id: "req-002".to_string(),
            method: "POST".to_string(),
            path: "/api/users".to_string(),
            ip: "192.168.1.2".to_string(),
            user_agent: "PostmanRuntime/7.29.0".to_string(),
        },
        Request {
            id: "req-003".to_string(),
            method: "GET".to_string(),
            path: "/api/error".to_string(),
            ip: "192.168.1.3".to_string(),
            user_agent: "curl/7.79.1".to_string(),
        },
    ];

    // Process requests
    for req in requests {
        let res = handle_request(req);
        match res {
            Ok(_) => {} // Already logged in handle_request
            Err(e) => error!("Request handler error: {}", e),
        }
    }

    info!("Service stopping");
}

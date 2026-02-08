/// Actix-Web middleware benchmark demonstrating ACL integration in a web application.
///
/// This benchmark creates a realistic web server with ACL-based authorization middleware
/// and simulates concurrent HTTP requests to measure real-world performance.
///
/// Run with: `cargo run --release --example benchmark_actix_middleware`

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    middleware, web, App, Error, HttpResponse, HttpServer,
};
use std::convert::TryFrom;
use std::fs::File;
use std::future::{ready, Ready};
use std::sync::Arc;
use std::time::Instant;
use walrs_acl::simple::{Acl, AclData};

/// Shared application state containing the ACL
#[derive(Clone)]
struct AppState {
    acl: Arc<Acl>,
}

/// Middleware factory for ACL-based authorization
pub struct AclMiddleware {
    acl: Arc<Acl>,
}

impl AclMiddleware {
    pub fn new(acl: Arc<Acl>) -> Self {
        Self { acl }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AclMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AclMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AclMiddlewareService {
            service,
            acl: self.acl.clone(),
        }))
    }
}

/// Middleware service that performs ACL checks on each request
pub struct AclMiddlewareService<S> {
    service: S,
    acl: Arc<Acl>,
}

impl<S, B> Service<ServiceRequest> for AclMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract authorization info from headers
        let role = req
            .headers()
            .get("X-User-Role")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("guest");

        let resource = req
            .headers()
            .get("X-Resource")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("homepage");

        let privilege = req
            .headers()
            .get("X-Privilege")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("read");

        // Perform ACL check (this is what we're benchmarking)
        let allowed = self.acl.is_allowed(
            Some(role),
            Some(resource),
            Some(privilege),
        );

        // For benchmark purposes, we still process the request
        // In production, you'd return 403 Forbidden if not allowed
        if !allowed {
            // Could track denied requests here
        }

        self.service.call(req)
    }
}

// Request handlers
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Welcome!")
}

async fn api_endpoint(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "roles": data.acl.role_count(),
        "resources": data.acl.resource_count()
    }))
}

async fn protected_resource() -> HttpResponse {
    HttpResponse::Ok().body("Protected resource accessed")
}

async fn admin_panel() -> HttpResponse {
    HttpResponse::Ok().body("Admin panel")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("=== Actix-Web ACL Middleware Benchmark ===\n");

    // Load ACL
    println!("Loading ACL from JSON...");
    let start = Instant::now();

    let file = File::open("test-fixtures/example-extensive-acl-array.json")
        .expect("Failed to open ACL file");

    let acl_data: AclData = serde_json::from_reader(file)
        .expect("Failed to parse ACL JSON");

    let acl = Acl::try_from(&acl_data)
        .expect("Failed to create ACL");

    let acl = Arc::new(acl);
    let load_duration = start.elapsed();

    println!("✓ ACL loaded in {:?}", load_duration);
    println!("  - Roles: {}", acl.role_count());
    println!("  - Resources: {}", acl.resource_count());
    println!();

    // Create app state
    let app_state = AppState { acl: acl.clone() };

    println!("Starting web server on http://127.0.0.1:8080");
    println!();
    println!("The server will run for 30 seconds to allow benchmark testing.");
    println!();
    println!("Test endpoints:");
    println!("  GET  /               - Public homepage");
    println!("  GET  /api/status     - API endpoint");
    println!("  GET  /protected      - Protected resource");
    println!("  GET  /admin          - Admin panel");
    println!();
    println!("Send requests with headers:");
    println!("  X-User-Role: <role>      (e.g., guest, user, admin)");
    println!("  X-Resource: <resource>   (e.g., blog, forum, admin_panel)");
    println!("  X-Privilege: <privilege> (e.g., read, write, delete)");
    println!();
    println!("Example benchmark command:");
    println!("  wrk -t4 -c100 -d10s \\");
    println!("    -H 'X-User-Role: user' \\");
    println!("    -H 'X-Resource: blog' \\");
    println!("    -H 'X-Privilege: read' \\");
    println!("    http://127.0.0.1:8080/");
    println!();
    println!("Alternative with curl:");
    println!("  curl -H 'X-User-Role: admin' \\");
    println!("       -H 'X-Resource: admin_panel' \\");
    println!("       -H 'X-Privilege: read' \\");
    println!("       http://127.0.0.1:8080/admin");
    println!();

    // Start web server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(AclMiddleware::new(acl.clone()))
            .route("/", web::get().to(index))
            .route("/api/status", web::get().to(api_endpoint))
            .route("/protected", web::get().to(protected_resource))
            .route("/admin", web::get().to(admin_panel))
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    server.await
}

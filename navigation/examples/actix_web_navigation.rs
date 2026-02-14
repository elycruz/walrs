use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use walrs_navigation::{Container, Page, view};

/// Shared navigation state
struct AppState {
    navigation: Container,
}

/// Home page handler
#[get("/")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let mut nav = data.navigation.clone();
    nav.set_active_by_uri("/");
    let html = render_page(&nav, "/");
    HttpResponse::Ok().content_type("text/html").body(html)
}

/// About page handler
#[get("/about")]
async fn about(data: web::Data<AppState>) -> impl Responder {
    let mut nav = data.navigation.clone();
    nav.set_active_by_uri("/about");
    let html = render_page(&nav, "/about");
    HttpResponse::Ok().content_type("text/html").body(html)
}

/// Products page handler
#[get("/products")]
async fn products(data: web::Data<AppState>) -> impl Responder {
    let mut nav = data.navigation.clone();
    nav.set_active_by_uri("/products");
    let html = render_page(&nav, "/products");
    HttpResponse::Ok().content_type("text/html").body(html)
}

/// API endpoint that returns navigation as JSON
#[get("/api/navigation")]
async fn api_navigation(data: web::Data<AppState>) -> impl Responder {
    match data.navigation.to_json() {
        Ok(json) => HttpResponse::Ok().content_type("application/json").body(json),
        Err(_) => HttpResponse::InternalServerError().body("Failed to serialize navigation"),
    }
}

/// Renders an HTML page with navigation menu, breadcrumbs, and content
fn render_page(nav: &Container, current_uri: &str) -> String {
    let menu_html = view::render_menu_with_class(nav, "nav-menu", "active");
    let breadcrumb_html = view::render_breadcrumbs(nav, " &gt; ");
    let escaped_uri = view::html_escape(current_uri);

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Navigation Example - {uri}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }}
        .nav-menu {{
            list-style-type: none;
            padding: 0;
            background-color: #333;
            overflow: hidden;
        }}
        .nav-menu > li {{
            float: left;
            position: relative;
        }}
        .nav-menu a {{
            display: block;
            color: white;
            text-align: center;
            padding: 14px 16px;
            text-decoration: none;
        }}
        .nav-menu a:hover {{
            background-color: #111;
        }}
        .nav-menu .active a {{
            background-color: #4CAF50;
        }}
        .nav-menu ul {{
            display: none;
            position: absolute;
            background-color: #444;
            min-width: 160px;
            box-shadow: 0px 8px 16px 0px rgba(0,0,0,0.2);
            z-index: 1;
            list-style-type: none;
            padding: 0;
        }}
        .nav-menu > li:hover ul {{
            display: block;
        }}
        .nav-menu ul li {{
            float: none;
        }}
        .nav-menu ul a {{
            text-align: left;
        }}
        .breadcrumbs {{
            padding: 10px 0;
            color: #666;
            clear: both;
        }}
        .breadcrumbs a {{
            color: #4CAF50;
            text-decoration: none;
        }}
        .breadcrumbs .active {{
            font-weight: bold;
            color: #333;
        }}
        .content {{
            clear: both;
            padding-top: 20px;
        }}
        h1 {{
            color: #333;
        }}
        .info {{
            background-color: #f0f0f0;
            padding: 15px;
            border-radius: 5px;
            margin-top: 20px;
        }}
    </style>
</head>
<body>
    <nav>
        {menu}
    </nav>
    <div class="breadcrumbs">
        {breadcrumbs}
    </div>
    <div class="content">
        <h1>Page: {uri}</h1>
        <p>This is a demonstration of the walrs_navigation component integrated with Actix Web.</p>
        <div class="info">
            <h3>Navigation Features:</h3>
            <ul>
                <li>Hierarchical menu structure</li>
                <li>Active page highlighting</li>
                <li>Dropdown sub-menus</li>
                <li>Breadcrumb trail</li>
                <li>JSON API endpoint at <a href="/api/navigation">/api/navigation</a></li>
            </ul>
            <h3>Available Pages:</h3>
            <ul>
                <li><a href="/">Home</a></li>
                <li><a href="/about">About</a></li>
                <li><a href="/products">Products</a></li>
            </ul>
        </div>
    </div>
</body>
</html>"#,
        uri = escaped_uri,
        menu = menu_html,
        breadcrumbs = breadcrumb_html
    )
}

/// Initialize navigation structure
fn create_navigation() -> Container {
    let mut nav = Container::new();

    // Products section with sub-pages
    let mut products_page = Page::builder()
        .label("Products")
        .uri("/products")
        .order(2)
        .build();

    products_page.add_page(
        Page::builder()
            .label("Books")
            .uri("/products/books")
            .order(1)
            .build(),
    );

    products_page.add_page(
        Page::builder()
            .label("Electronics")
            .uri("/products/electronics")
            .order(2)
            .build(),
    );

    // About section with sub-pages
    let mut about_page = Page::builder()
        .label("About")
        .uri("/about")
        .order(3)
        .build();

    about_page.add_page(
        Page::builder()
            .label("Team")
            .uri("/about/team")
            .build(),
    );

    about_page.add_page(
        Page::builder()
            .label("Contact")
            .uri("/about/contact")
            .build(),
    );

    // Use fluent interface to add all pages
    nav.add_page(
            Page::builder()
                .label("Home")
                .uri("/")
                .order(1)
                .build(),
        )
        .add_page(products_page)
        .add_page(about_page);

    nav
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Actix Web server with Navigation component...");
    println!("Server running at http://127.0.0.1:8080");
    println!();
    println!("Available endpoints:");
    println!("  - http://127.0.0.1:8080/");
    println!("  - http://127.0.0.1:8080/about");
    println!("  - http://127.0.0.1:8080/products");
    println!("  - http://127.0.0.1:8080/api/navigation (JSON API)");
    println!();

    // Create navigation and wrap in application state
    let navigation = create_navigation();
    let app_state = web::Data::new(AppState { navigation });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(index)
            .service(about)
            .service(products)
            .service(api_navigation)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

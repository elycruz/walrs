use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use walrs_navigation::{Container, Page};

/// Shared navigation state
struct AppState {
    navigation: Container,
}

/// Home page handler
#[get("/")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let nav = &data.navigation;
    let html = render_html_menu(nav, "/");
    HttpResponse::Ok().content_type("text/html").body(html)
}

/// About page handler
#[get("/about")]
async fn about(data: web::Data<AppState>) -> impl Responder {
    let nav = &data.navigation;
    let html = render_html_menu(nav, "/about");
    HttpResponse::Ok().content_type("text/html").body(html)
}

/// Products page handler
#[get("/products")]
async fn products(data: web::Data<AppState>) -> impl Responder {
    let nav = &data.navigation;
    let html = render_html_menu(nav, "/products");
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

/// Renders an HTML page with navigation menu
fn render_html_menu(nav: &Container, current_uri: &str) -> String {
    let mut menu_html = String::new();
    menu_html.push_str("<ul class=\"nav-menu\">\n");

    for page in nav.pages() {
        let active_class = if page.uri() == Some(current_uri) {
            " class=\"active\""
        } else {
            ""
        };

        menu_html.push_str(&format!(
            "  <li{}><a href=\"{}\">{}</a>",
            active_class,
            page.uri().unwrap_or("#"),
            page.label().unwrap_or("(no label)")
        ));

        // Render child pages if any
        if page.has_pages() {
            menu_html.push_str("\n    <ul class=\"sub-menu\">\n");
            for child in page.pages() {
                menu_html.push_str(&format!(
                    "      <li><a href=\"{}\">{}</a></li>\n",
                    child.uri().unwrap_or("#"),
                    child.label().unwrap_or("(no label)")
                ));
            }
            menu_html.push_str("    </ul>\n  ");
        }

        menu_html.push_str("</li>\n");
    }

    menu_html.push_str("</ul>");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Navigation Example</title>
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
        .sub-menu {{
            display: none;
            position: absolute;
            background-color: #444;
            min-width: 160px;
            box-shadow: 0px 8px 16px 0px rgba(0,0,0,0.2);
            z-index: 1;
            list-style-type: none;
            padding: 0;
        }}
        .nav-menu > li:hover .sub-menu {{
            display: block;
        }}
        .sub-menu li {{
            float: none;
        }}
        .sub-menu a {{
            text-align: left;
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
        {}
    </nav>
    <div class="content">
        <h1>Page: {}</h1>
        <p>This is a demonstration of the walrs_navigation component integrated with Actix Web.</p>
        <div class="info">
            <h3>Navigation Features:</h3>
            <ul>
                <li>Hierarchical menu structure</li>
                <li>Active page highlighting</li>
                <li>Dropdown sub-menus</li>
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
        menu_html, current_uri
    )
}

/// Initialize navigation structure
fn create_navigation() -> Container {
    let mut nav = Container::new();

    // Home page
    nav.add_page(
        Page::builder()
            .label("Home")
            .uri("/")
            .order(1)
            .build(),
    );

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

    nav.add_page(products_page);

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

    nav.add_page(about_page);

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

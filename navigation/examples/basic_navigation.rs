use walrs_navigation::{Container, Page, view};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Navigation Component Example ===\n");

    // Create a navigation container
    let mut nav = Container::new();

    // Add root-level pages
    nav.add_page(
        Page::builder()
            .label("Home")
            .uri("/")
            .title("Home Page")
            .order(1)
            .build(),
    );

    // Create a products section with nested pages
    let mut products = Page::builder()
        .label("Products")
        .uri("/products")
        .title("Our Products")
        .route("products")
        .order(2)
        .build();

    products.add_page(
        Page::builder()
            .label("Books")
            .uri("/products/books")
            .order(1)
            .class("nav-item")
            .build(),
    );

    products.add_page(
        Page::builder()
            .label("Electronics")
            .uri("/products/electronics")
            .order(2)
            .build(),
    );

    products.add_page(
        Page::builder()
            .label("Clothing")
            .uri("/products/clothing")
            .order(3)
            .build(),
    );

    nav.add_page(products);

    // Add more root-level pages
    let mut about = Page::builder()
        .label("About")
        .uri("/about")
        .order(3)
        .build();

    about.add_page(
        Page::builder()
            .label("Team")
            .uri("/about/team")
            .build(),
    );

    about.add_page(
        Page::builder()
            .label("Contact")
            .uri("/about/contact")
            .build(),
    );

    nav.add_page(about);

    nav.add_page(
        Page::builder()
            .label("Blog")
            .uri("/blog")
            .order(4)
            .target("_blank")
            .build(),
    );

    // Display navigation structure
    println!("Navigation Structure:");
    println!("Total pages: {}", nav.count());
    println!();

    // Traverse with depth information
    println!("All pages (depth-first with indentation):");
    nav.traverse_with_depth(&mut |page, depth| {
        let indent = "  ".repeat(depth);
        println!(
            "{}{} - {}",
            indent,
            page.label.as_deref().unwrap_or("(no label)"),
            page.uri.as_deref().unwrap_or("(no URI)")
        );
    });
    println!();

    // Find specific pages
    println!("Finding pages:");
    if let Some(books) = nav.find_by_uri("/products/books") {
        println!("Found by URI: {} at /products/books", books.label.as_deref().unwrap());
    }

    if let Some(team) = nav.find_by_label("Team") {
        println!("Found by label: Team at {}", team.uri.as_deref().unwrap());
    }

    if let Some(products_page) = nav.find_by_route("products") {
        println!("Found by route: {}", products_page.label.as_deref().unwrap());
    }

    // Find pages using find_page with a custom predicate
    if let Some(page) = nav.find_page(|p| p.label.as_deref() == Some("Blog")) {
        println!("Found by predicate: {} (target={})", page.label.as_deref().unwrap(), page.target.as_deref().unwrap_or("none"));
    }

    // Find all pages with a specific class using find_page
    let nav_items: Vec<_> = nav.pages().iter()
        .flat_map(|p| p.find_all_pages(|p| p.class.as_deref() == Some("nav-item")))
        .collect();
    println!("Pages with class 'nav-item': {}", nav_items.len());
    println!();

    // Only visible pages
    println!("Visible root pages: {}", nav.visible_pages().len());

    // Check if a page exists
    println!("Has /products/books? {}", nav.has_page(|p| p.uri.as_deref() == Some("/products/books"), true));
    println!();

    // Demonstrate breadcrumbs
    nav.set_active_by_uri("/products/books");
    let crumbs = nav.breadcrumbs();
    println!("Breadcrumb trail:");
    for (i, crumb) in crumbs.iter().enumerate() {
        if i > 0 {
            print!(" > ");
        }
        print!("{}", crumb.label.as_deref().unwrap_or(""));
    }
    println!("\n");

    // Render breadcrumbs as HTML
    println!("Breadcrumbs HTML:");
    println!("{}", view::render_breadcrumbs(&nav, " &gt; "));
    println!();

    // Render as HTML menu
    println!("Menu HTML:");
    println!("{}", view::render_menu(&nav));
    println!();

    // Render menu with custom classes
    println!("Menu with custom classes:");
    println!("{}", view::render_menu_with_class(&nav, "navbar-nav", "current"));
    println!();

    // Render as sitemap
    println!("Sitemap HTML (flat):");
    println!("{}", view::render_sitemap(&nav));
    println!();

    println!("Sitemap HTML (hierarchical):");
    println!("{}", view::render_sitemap_hierarchical(&nav));
    println!();

    // Demonstrate JSON serialization
    println!("JSON representation:");
    let json = nav.to_json_pretty()?;
    println!("{}", json);
    println!();

    // Demonstrate YAML serialization
    println!("YAML representation:");
    let yaml = nav.to_yaml()?;
    println!("{}", yaml);
    println!();

    // Demonstrate loading from JSON
    let json_input = r#"[
        {
            "label": "Help",
            "uri": "/help",
            "order": 100
        },
        {
            "label": "FAQ",
            "uri": "/faq",
            "order": 101
        }
    ]"#;

    println!("Loading from JSON:");
    let nav2 = Container::from_json(json_input)?;
    println!("Loaded {} pages from JSON", nav2.count());
    for page in nav2.iter() {
        println!("  - {} ({})", page.label.as_deref().unwrap(), page.uri.as_deref().unwrap());
    }
    println!();

    // Demonstrate direct field access
    println!("Direct field access:");
    let mut page = Page::new();
    page.label = Some("Dynamic".to_string());
    page.uri = Some("/dynamic".to_string());
    page.title = Some("Dynamically Created".to_string());
    println!(
        "  label={}, uri={}, title={}",
        page.label.as_deref().unwrap(),
        page.uri.as_deref().unwrap(),
        page.title.as_deref().unwrap()
    );
    println!();

    // Demonstrate href with fragment
    let page_with_fragment = Page::builder()
        .label("Section Link")
        .uri("/docs")
        .fragment("installation")
        .build();
    println!("Page with fragment:");
    println!("  href = {}", page_with_fragment.href().unwrap());

    // Demonstrate add_pages
    let mut nav3 = Container::new();
    nav3.add_pages(vec![
        Page::builder().label("Page A").order(2).build(),
        Page::builder().label("Page B").order(1).build(),
        Page::builder().label("Page C").order(3).build(),
    ]);
    println!("\nBulk add (sorted by order):");
    for page in nav3.iter() {
        println!("  {} (order {})", page.label.as_deref().unwrap(), page.order);
    }

    Ok(())
}

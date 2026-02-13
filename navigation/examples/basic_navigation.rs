use walrs_navigation::{Container, Page};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Navigation Component Example ===\n");

    // Create a navigation container
    let mut nav = Container::new();

    // Add root-level pages
    nav.add_page(
        Page::builder()
            .label("Home")
            .uri("/")
            .order(1)
            .build(),
    );

    // Create a products section with nested pages
    let mut products = Page::builder()
        .label("Products")
        .uri("/products")
        .order(2)
        .build();

    products.add_page(
        Page::builder()
            .label("Books")
            .uri("/products/books")
            .order(1)
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
            .build(),
    );

    // Display navigation structure
    println!("Navigation Structure:");
    println!("Total pages: {}", nav.count());
    println!();

    // Traverse and display all pages
    println!("All pages (depth-first):");
    nav.traverse(&mut |page| {
        let indent = if page.uri().map(|u| u.contains("/products/")).unwrap_or(false)
            || page.uri().map(|u| u.contains("/about/")).unwrap_or(false)
        {
            "  "
        } else {
            ""
        };

        println!(
            "{}{} - {}",
            indent,
            page.label().unwrap_or("(no label)"),
            page.uri().unwrap_or("(no URI)")
        );
    });
    println!();

    // Find specific pages
    println!("Finding pages:");
    if let Some(books) = nav.find_by_uri("/products/books") {
        println!("Found by URI: {} at /products/books", books.label().unwrap());
    }

    if let Some(team) = nav.find_by_label("Team") {
        println!("Found by label: Team at {}", team.uri().unwrap());
    }
    println!();

    // Demonstrate JSON serialization
    println!("JSON representation (pretty):");
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
        println!("  - {} ({})", page.label().unwrap(), page.uri().unwrap());
    }
    println!();

    // Demonstrate active page setting
    let mut nav3 = Container::new();
    nav3.add_page(Page::builder().label("Home").uri("/").build());
    nav3.add_page(Page::builder().label("About").uri("/about").build());
    nav3.add_page(Page::builder().label("Contact").uri("/contact").build());

    nav3.set_active_by_uri("/about");

    println!("Active page demonstration:");
    for page in nav3.iter() {
        let active_marker = if page.is_active() { " [ACTIVE]" } else { "" };
        println!("  {} - {}{}", page.label().unwrap(), page.uri().unwrap(), active_marker);
    }

    Ok(())
}

use walrs_navigation::{Container, Page};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("=== YAML Navigation Example ===\n");

  // Create a navigation container
  let mut nav = Container::new();

  nav.add_page(
    Page::builder()
      .label("Home")
      .uri("/")
      .title("Home Page")
      .order(1)
      .build(),
  );

  let mut products = Page::builder()
    .label("Products")
    .uri("/products")
    .title("Our Products")
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

  nav.add_page(products);

  nav.add_page(
    Page::builder()
      .label("About")
      .uri("/about")
      .order(3)
      .build(),
  );

  // Serialize to YAML
  println!("YAML representation:");
  let yaml = nav.to_yaml()?;
  println!("{}", yaml);
  println!();

  // Deserialize from YAML
  let yaml_input = r#"
- label: Help
  uri: /help
  order: 100
- label: FAQ
  uri: /faq
  order: 101
"#;

  println!("Loading from YAML:");
  let nav2 = Container::from_yaml(yaml_input)?;
  println!("Loaded {} pages from YAML", nav2.count());
  for page in nav2.iter() {
    println!(
      "  - {} ({})",
      page.label.as_deref().unwrap(),
      page.uri.as_deref().unwrap()
    );
  }
  println!();

  // Round-trip: serialize and deserialize
  println!("Round-trip test:");
  let yaml_out = nav.to_yaml()?;
  let nav3 = Container::from_yaml(&yaml_out)?;
  println!(
    "Original pages: {}, Round-tripped pages: {}",
    nav.count(),
    nav3.count()
  );

  Ok(())
}

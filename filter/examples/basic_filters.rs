//! Example: Basic filter usage
//!
//! Run with: `cargo run --example basic_filters`

use std::borrow::Cow;
use walrs_filter::{Filter, SlugFilter, StripTagsFilter, XmlEntitiesFilter};

fn main() {
    println!("=== walrs_filter Examples ===\n");

    // SlugFilter example
    println!("--- SlugFilter ---");
    let slug_filter = SlugFilter::new(200, false); // max_length=200, no duplicate dashes

    let titles = [
        "Hello World!",
        "My Blog Post Title",
        "Special Characters: @#$%^&*()",
        "Multiple   Spaces   Here",
        "Ça fait du café très bon!",
    ];

    for title in titles {
        let slug = slug_filter.filter(Cow::Borrowed(title));
        println!("  \"{}\" -> \"{}\"", title, slug);
    }

    println!();

    // SlugFilter with duplicate dashes allowed
    println!("--- SlugFilter (allow duplicate dashes) ---");
    let slug_filter_dashes = SlugFilter::new(200, true);

    let title = "Hello---World   Test";
    let slug = slug_filter_dashes.filter(Cow::Borrowed(title));
    println!("  \"{}\" -> \"{}\"", title, slug);

    println!();

    // StripTagsFilter example
    println!("--- StripTagsFilter ---");
    let strip_filter = StripTagsFilter::new();

    let html_samples = [
        "<p>Hello World</p>",
        "<script>alert('xss')</script>Safe content",
        "<b>Bold</b> and <i>italic</i> text",
        "<style>body { color: red; }</style>Styled text",
        "<a href=\"https://example.com\">Link</a>",
    ];

    for html in html_samples {
        let clean = strip_filter.filter(Cow::Borrowed(html));
        println!("  \"{}\"", html);
        println!("    -> \"{}\"", clean);
    }

    println!();

    // XmlEntitiesFilter example
    println!("--- XmlEntitiesFilter ---");
    let xml_filter = XmlEntitiesFilter::new();

    let strings = [
        "Hello & Goodbye",
        "5 < 10 and 10 > 5",
        "He said \"Hello\"",
        "It's a test",
        "<script>alert('xss')</script>",
    ];

    for s in strings {
        let encoded = xml_filter.filter(Cow::Borrowed(s));
        println!("  \"{}\"", s);
        println!("    -> \"{}\"", encoded);
    }

    println!();
    println!("=== Examples Complete ===");
}


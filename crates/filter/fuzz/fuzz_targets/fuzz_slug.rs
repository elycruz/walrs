#![no_main]

use libfuzzer_sys::fuzz_target;
use std::borrow::Cow;
use walrs_filter::{slug::to_pretty_slug, slug::to_slug, Filter, SlugFilter};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = to_slug(Cow::Borrowed(s));
        let _ = to_pretty_slug(Cow::Borrowed(s));

        let filter = SlugFilter::default();
        let _ = filter.filter(Cow::Borrowed(s));
    }
});

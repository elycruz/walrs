#![no_main]

use libfuzzer_sys::fuzz_target;
use std::borrow::Cow;
use walrs_filter::Filter;
use walrs_filter::StripTagsFilter;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let filter = StripTagsFilter::new();
        let _ = filter.filter(Cow::Borrowed(s));
    }
});

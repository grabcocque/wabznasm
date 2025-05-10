#![no_main]
use libfuzzer_sys::fuzz_target;
use wabznasm::parser::parse_expression;

fuzz_target!(|data: &[u8]| {
    // Interpret bytes as UTF-8 string, skip invalid sequences
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_expression(s);
    }
});

#![no_main]
use libfuzzer_sys::fuzz_target;
use wabznasm::codegen::compile_and_run;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = compile_and_run(s);
    }
});

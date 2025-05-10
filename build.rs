use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=grammar/grammar.js");
    // Generate the parser if needed
    let status = Command::new("tree-sitter")
        .arg("generate")
        .current_dir("grammar")
        .status()
        .expect("Failed to run tree-sitter generate");
    if !status.success() {
        panic!("tree-sitter generate failed");
    }
    // Compile the generated parser
    let mut build = cc::Build::new();
    build.include("grammar/src");
    build.file("grammar/src/parser.c");
    let scanner = Path::new("grammar/src/scanner.c");
    if scanner.exists() {
        build.file(scanner);
    }
    // Configure C debug/optimization flags based on cargo profile
    let profile = env::var("PROFILE").expect("PROFILE not set");
    if profile == "debug" {
        build.debug(true);
        build.opt_level(0);
    } else {
        build.debug(false);
        build.opt_level(3);
    }
    build.compile("calc");
    // Write Rust bindings for the generated parser
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("calc.rs");
    let bindings = concat!(
        "use tree_sitter::Language;\n",
        "unsafe extern \"C\" { fn tree_sitter_calc() -> Language; }\n",
        "pub fn language() -> Language { unsafe { tree_sitter_calc() } }\n"
    );
    fs::write(dest_path, bindings).expect("Failed to write calc bindings");
}

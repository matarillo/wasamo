//! Compiles `examples/bool-demo/bool-demo.ui` to Wasamo IR text at build time
//! and exposes the resulting path through `WASAMO_BOOL_DEMO_IR`.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ui_path = manifest_dir
        .join("..")
        .join("bool-demo")
        .join("bool-demo.ui")
        .canonicalize()
        .expect("examples/bool-demo/bool-demo.ui must exist");

    println!("cargo:rerun-if-changed={}", ui_path.display());

    let src =
        fs::read_to_string(&ui_path).unwrap_or_else(|e| panic!("read {}: {e}", ui_path.display()));
    let path_str = ui_path.to_string_lossy();

    let tokens = wasamoc::lexer::tokenize(&src, &path_str)
        .unwrap_or_else(|d| panic!("bool-demo.ui tokenize:\n{}", d.render(&src)));
    let ast = wasamoc::parser::parse(&tokens, &path_str)
        .unwrap_or_else(|d| panic!("bool-demo.ui parse:\n{}", d.render(&src)));
    let result = wasamoc::check::check(&ast, &path_str);
    if result.has_errors() {
        let rendered: Vec<String> = result.diagnostics.iter().map(|d| d.render(&src)).collect();
        panic!("bool-demo.ui check failed:\n{}", rendered.join("\n"));
    }
    let comp = wasamoc::lower::lower(&ast, &result.namespace);
    let ir_text = wasamoc::emit::emit(&comp);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_dir.join("bool-demo.uic");
    fs::write(&out_path, &ir_text).unwrap_or_else(|e| panic!("write {}: {e}", out_path.display()));

    println!("cargo:rustc-env=WASAMO_BOOL_DEMO_IR={}", out_path.display());
}

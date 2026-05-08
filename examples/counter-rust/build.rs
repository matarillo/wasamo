//! Compiles `examples/counter/counter.ui` to Wasamo IR text at build time
//! and exposes the absolute path of the resulting `counter.uic` to the
//! crate via the `WASAMO_COUNTER_IR` env var (consumed in `main.rs` via
//! `env!`). This is the counter-rust side of DD-M2-P6-008.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ui_path = manifest_dir
        .join("..")
        .join("counter")
        .join("counter.ui")
        .canonicalize()
        .expect("examples/counter/counter.ui must exist");

    println!("cargo:rerun-if-changed={}", ui_path.display());

    let src = fs::read_to_string(&ui_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", ui_path.display()));
    let path_str = ui_path.to_string_lossy();

    let tokens = wasamoc::lexer::tokenize(&src, &path_str)
        .unwrap_or_else(|d| panic!("counter.ui tokenize:\n{}", d.render(&src)));
    let ast = wasamoc::parser::parse(&tokens, &path_str)
        .unwrap_or_else(|d| panic!("counter.ui parse:\n{}", d.render(&src)));
    let result = wasamoc::check::check(&ast, &path_str);
    if result.has_errors() {
        let rendered: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| d.render(&src))
            .collect();
        panic!("counter.ui check failed:\n{}", rendered.join("\n"));
    }
    let comp = wasamoc::lower::lower(&ast, &result.namespace);
    let ir_text = wasamoc::emit::emit(&comp);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_dir.join("counter.uic");
    fs::write(&out_path, &ir_text)
        .unwrap_or_else(|e| panic!("write {}: {e}", out_path.display()));

    println!("cargo:rustc-env=WASAMO_COUNTER_IR={}", out_path.display());
}

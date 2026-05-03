// Build script for wasamo-sys.
//
// `wasamo.dll.lib` (the import library for `wasamo.dll`) is produced by
// the `wasamo-dll` cdylib shim crate into `<target>/<profile>/`. This
// script tells the linker where to find it. The path is derived from
// OUT_DIR, which cargo sets to `<target>/<profile>/build/wasamo-sys-<hash>/out`.
//
// Build-order is enforced by the `wasamo-dll` dependency entry in this
// crate's Cargo.toml (DD-M2-P1-006): cargo will not start building any
// binary that depends on `wasamo-sys` until `wasamo-dll` has finished
// producing `wasamo.dll` and its import library.

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("walking up to <target>/<profile>")
        .to_path_buf();

    let import_lib = profile_dir.join("wasamo.dll.lib");
    if !import_lib.exists() {
        println!(
            "cargo:warning=wasamo.dll.lib not found at {}. \
             Build wasamo-runtime first (use `cargo build --workspace` \
             or `cargo build -p wasamo-runtime`) so its import library \
             exists before wasamo-sys is linked.",
            import_lib.display()
        );
    }

    // rustc emits the cdylib's import library as `wasamo.dll.lib` (the
    // Windows-MSVC convention for Rust cdylibs). `dylib=wasamo` would
    // make link.exe look for `wasamo.lib` instead, so we use `+verbatim`
    // to pass the actual filename through unchanged.
    println!("cargo:rustc-link-search=native={}", profile_dir.display());
    println!("cargo:rustc-link-lib=dylib:+verbatim=wasamo.dll.lib");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", import_lib.display());
}

// Build script for wasamo-dll.
//
// Forces all symbols from `wasamo-runtime`'s rlib into the cdylib output
// via MSVC `/WHOLEARCHIVE`. Without this, the linker would only include
// symbols that are referenced from within this crate, which is none
// (the shim's lib.rs is intentionally empty). The `#[no_mangle] pub extern "C"`
// symbols defined in `wasamo-runtime` must be visible in `wasamo.dll`.
//
// DD-M2-P1-005: whole-archive via build.rs is the agreed mechanism.

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    // OUT_DIR is <target>/<profile>/build/wasamo-dll-<hash>/out;
    // walk up 3 levels to reach <target>/<profile>/.
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("walking up to <target>/<profile>")
        .to_path_buf();

    // MSVC only; this project is Windows-only (WinRT/Visual Layer).
    // The rlib is placed at <profile>/libwasamo_runtime.rlib by cargo.
    let rlib = profile_dir.join("libwasamo_runtime.rlib");
    println!("cargo:rustc-link-arg=/WHOLEARCHIVE:{}", rlib.display());
    println!("cargo:rerun-if-changed=build.rs");
}

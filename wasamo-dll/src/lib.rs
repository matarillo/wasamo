// wasamo-dll: cdylib shim that re-exports all `wasamo-runtime` C ABI symbols
// into `wasamo.dll` via whole-archive linking (build.rs DD-M2-P1-005).
// This crate has no Rust-library surface of its own.
extern crate wasamo_runtime;

//! Shared runtime bootstrap for the mock-free, Windows-only integration
//! tests that drive a live Compositor.
//!
//! This comment is intentionally self-contained: you should not need to open
//! any `.md` to understand why the helper exists or when to use it. Full
//! provenance / evidence is `docs/notes/verification-environments.md`
//! Observation 5, but it is not required reading.
//!
//! # Why this helper exists (rationale)
//!
//! The runtime caches the Compositor in a process-global `static`
//! (`OnceLock<Runtime>` in `runtime.rs`). A WinRT Compositor has **STA
//! apartment affinity**: it belongs to the thread that first called
//! `wasamo_init`, and the in-proc COM server that implements it
//! (`dcomp.dll`) is loaded into that thread's apartment.
//!
//! libtest runs **each `#[test]` on its own freshly spawned thread**, even
//! under `--test-threads=1` (the flag caps concurrency, it does not stop the
//! per-test thread spawn). So if a test initializes the runtime itself, the
//! Compositor ends up owned by *that test's* thread. When the test finishes,
//! its thread — and therefore its STA apartment — is torn down, and
//! `dcomp.dll` is unloaded. The Compositor pointer still cached in the
//! `static` is now dangling, because its vtable lived inside `dcomp.dll`.
//! The **next** test in the same binary fetches that cached Compositor and
//! calls a method on it (e.g. `CreateSpriteVisual` while building its widget
//! tree), dispatches through the freed vtable, and the process dies with
//! `STATUS_ACCESS_VIOLATION` (`0xC0000005`) — *after* the first test already
//! printed `ok`.
//!
//! This helper instead initializes the runtime on a **dedicated thread that
//! parks forever and never exits**, so the Compositor's apartment (and
//! `dcomp.dll`) stay resident for the entire test binary. The cached
//! Compositor therefore remains valid across every test in the binary.
//! Individual tests still build and drive their widget trees on their own
//! libtest threads; those calls are technically cross-apartment, but they do
//! not fault while `dcomp.dll` is held resident by the parked thread.
//!
//! # Which tests MUST use this helper
//!
//! Any single test binary (one `tests/<name>.rs` file) that contains **two
//! or more tests that build a live widget tree / touch the Compositor**.
//! With two such tests, the first test's thread death unloads `dcomp.dll`
//! and the second test faults. Route *every* Compositor test in such a file
//! through [`init_runtime_or_skip`] so the apartment is owned by the parked
//! thread rather than by any one test thread.
//!
//! # Which tests do NOT need this helper
//!
//! - **Binaries with at most one Compositor test.** A lone test initializes
//!   and uses the Compositor on its own thread, then the process exits;
//!   there is no *second* test to reuse a stale Compositor, so the crash
//!   cannot occur. Such tests may keep their own inline `RoInitialize` +
//!   `wasamo_init`. (Cargo compiles each `tests/<name>.rs` into its own
//!   binary and runs it as a separate process, so "one Compositor test per
//!   file" is what matters, not the total across the suite.)
//! - **Tests that never stand up the Compositor** (pure IR / parser /
//!   layout-math tests) — they do not call `wasamo_init` at all.
//!
//! # When this helper can be deleted
//!
//! Remove it once *either* condition holds:
//!
//! 1. **Remediation step 1 lands.** When all Compositor operations are
//!    marshalled onto the single owning thread (tests dispatch closures to
//!    the runtime thread instead of calling the Compositor from their own
//!    threads), no test touches the Compositor cross-thread and the parked
//!    thread becomes the marshalling thread — this keep-alive-only form is
//!    then superseded by that harness.
//! 2. **The harness stops creating the precondition.** e.g. the suite moves
//!    to a process-per-test runner (each `#[test]` in its own process, as
//!    `cargo nextest` does), or libtest stops spawning a fresh thread per
//!    test. Either removes the "second test reuses a torn-down apartment"
//!    sequence, making per-test inline init safe again.

#![cfg(windows)]

use std::ffi::CStr;
use std::sync::mpsc;
use std::sync::OnceLock;

use wasamo_runtime::ffi;

fn last_error() -> Option<String> {
    let ptr = ffi::wasamo_last_error_message();
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_str()
                .expect("last-error must be UTF-8")
                .to_owned(),
        )
    }
}

fn runtime_compositor_unavailable(msg: Option<&str>) -> bool {
    msg.is_some_and(|m| m.contains("0x80070005"))
}

fn github_actions() -> bool {
    std::env::var_os("GITHUB_ACTIONS").is_some()
}

#[derive(Clone, Copy)]
enum InitOutcome {
    Ready,
    CompositorUnavailable,
}

/// Initialize the runtime exactly once per process, on a parked keep-alive
/// thread that owns the Compositor's apartment for the whole binary.
fn ensure_runtime() -> InitOutcome {
    static INIT: OnceLock<InitOutcome> = OnceLock::new();
    *INIT.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("wasamo-test-runtime-keepalive".to_owned())
            .spawn(move || {
                let _ = unsafe {
                    windows::Win32::System::WinRT::RoInitialize(
                        windows::Win32::System::WinRT::RO_INIT_SINGLETHREADED,
                    )
                };
                let status = ffi::wasamo_init();
                let outcome = if status == ffi::WASAMO_ERR_RUNTIME
                    && runtime_compositor_unavailable(last_error().as_deref())
                {
                    InitOutcome::CompositorUnavailable
                } else {
                    assert_eq!(
                        status,
                        ffi::WASAMO_OK,
                        "wasamo_init failed: {:?}",
                        last_error()
                    );
                    InitOutcome::Ready
                };
                tx.send(outcome).expect("send runtime init outcome");
                // Park forever: keep this thread's STA apartment (and the
                // `dcomp.dll` server backing the Compositor) resident for the
                // lifetime of the test process.
                loop {
                    std::thread::park();
                }
            })
            .expect("spawn keep-alive runtime thread");
        rx.recv().expect("receive runtime init outcome")
    })
}

/// Initialize the runtime (once per process, on the parked keep-alive
/// thread) and prepare the calling test thread to issue COM calls against
/// the Compositor.
///
/// Returns `Some(())` when the runtime is ready, or `None` when the
/// Compositor is locally unavailable (the caller should `return`, skipping
/// the test). Fails — does not skip — on GitHub Actions, where a missing
/// Compositor must surface as a failure per CLAUDE.md §Testing rules; the
/// skip is a developer-laptop convenience only.
pub fn init_runtime_or_skip(test_name: &str) -> Option<()> {
    // The calling libtest thread issues the Compositor / Visual calls, so it
    // needs COM initialized too. The Compositor objects live in the
    // keep-alive apartment; these calls are cross-apartment but safe while
    // `dcomp.dll` stays resident (Observation 5, remediation step 2).
    let _ = unsafe {
        windows::Win32::System::WinRT::RoInitialize(
            windows::Win32::System::WinRT::RO_INIT_SINGLETHREADED,
        )
    };
    match ensure_runtime() {
        InitOutcome::Ready => Some(()),
        InitOutcome::CompositorUnavailable => {
            assert!(
                !github_actions(),
                "{test_name} cannot skip on GitHub Actions: \
                 runtime compositor unavailable"
            );
            eprintln!("skipping {test_name}: runtime compositor unavailable");
            None
        }
    }
}

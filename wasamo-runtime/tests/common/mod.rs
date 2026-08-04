//! Shared runtime bootstrap + executor for the mock-free, Windows-only
//! integration tests that drive a live Compositor.
//!
//! This comment is intentionally self-contained: you should not need to open
//! any `.md` to understand why the helper exists or how to use it. Full
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
//! This helper instead runs the runtime on a **single dedicated owning
//! thread that never exits**. That thread initializes the Compositor and
//! then serves as a work-queue executor: every test body that touches the
//! Compositor is shipped to it via [`run_on_owning_runtime_thread_or_skip`]
//! and runs *there*, one at a time. So the Compositor is created and used on
//! one and the same thread for the entire test binary — its apartment (and
//! `dcomp.dll`) stay resident, and no test ever touches it cross-apartment.
//! This mirrors production, where a host owns the Compositor on its single
//! UI thread for the whole process lifetime.
//!
//! (Historically this was a two-step remediation. Step 2 merely *parked* the
//! owning thread to keep the apartment resident while each test still called
//! the Compositor from its own libtest thread — crash-free, but technically
//! cross-apartment and safe only while `dcomp.dll` stayed loaded. Step 1,
//! implemented here, turns the parked thread into the executor above, so the
//! cross-apartment calls are gone, not merely tolerated.)
//!
//! # Which tests MUST use this helper
//!
//! Any single test binary (one `tests/<name>.rs` file) that contains **two
//! or more tests that build a live widget tree / touch the Compositor**.
//! With two such tests, per-test inline init would let the first test's
//! thread death unload `dcomp.dll` and the second test fault. Route *every*
//! Compositor test body in such a file through
//! [`run_on_owning_runtime_thread_or_skip`] so the Compositor is owned and
//! used by the executor thread rather than by any one test thread.
//!
//! # Which tests do NOT need this helper
//!
//! - **Binaries with at most one Compositor test.** A lone test initializes
//!   *and* uses the Compositor on its own thread, then the process exits:
//!   one apartment throughout (so no cross-apartment access), and no
//!   *second* test to reuse a torn-down one (so no crash). Such tests may
//!   keep their own inline `RoInitialize` + `wasamo_init`. (Cargo compiles
//!   each `tests/<name>.rs` into its own binary and runs it as a separate
//!   process, so "one Compositor test per file" is what matters, not the
//!   total across the suite.)
//! - **Tests that never stand up the Compositor** (pure IR / parser /
//!   layout-math tests) — they do not call `wasamo_init` at all.
//!
//! # When this helper can be deleted
//!
//! Remove it once **the harness stops creating the precondition** — e.g. the
//! suite moves to a process-per-test runner (each `#[test]` in its own
//! process, as `cargo nextest` does), or libtest stops spawning a fresh
//! thread per test. Either removes the "second test reuses a torn-down
//! apartment" sequence, making per-test inline init safe again. (The earlier
//! "delete once remediation step 1 lands" condition is discharged: step 1
//! *is* this executor.)

#![cfg(windows)]

use std::any::Any;
use std::ffi::CStr;
use std::panic::{catch_unwind, AssertUnwindSafe};
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

/// Convert a panic payload into a message that can be panicked again on the
/// calling libtest thread. `resume_unwind` intentionally bypasses that
/// thread's panic hook, which hides an assertion's values from CI output when
/// the originating owner thread's output is captured separately.
pub(crate) fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else {
        "non-string panic payload from runtime-owning thread".to_owned()
    }
}

/// A test body, boxed for delivery to the owning runtime thread.
type RuntimeJob = Box<dyn FnOnce() + Send>;

/// The process-global runtime state, established once on the owning thread.
enum Runtime {
    /// The Compositor is live; send jobs here to run them on the owning
    /// thread.
    Ready(mpsc::Sender<RuntimeJob>),
    /// The Compositor is unavailable on this machine (a dev laptop without a
    /// usable session); Compositor tests skip locally and fail on CI.
    CompositorUnavailable,
}

/// Bring the runtime up exactly once per process, on a dedicated owning
/// thread that initializes the Compositor and then runs queued test bodies
/// forever. Returns a handle to that thread (or the unavailable verdict).
fn ensure_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        let (outcome_tx, outcome_rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("wasamo-test-runtime-owner".to_owned())
            .spawn(move || {
                let _ = unsafe {
                    windows::Win32::System::WinRT::RoInitialize(
                        windows::Win32::System::WinRT::RO_INIT_SINGLETHREADED,
                    )
                };
                let status = ffi::wasamo_init();
                if status == ffi::WASAMO_ERR_RUNTIME
                    && runtime_compositor_unavailable(last_error().as_deref())
                {
                    outcome_tx.send(None).expect("send runtime init outcome");
                    return;
                }
                assert_eq!(
                    status,
                    ffi::WASAMO_OK,
                    "wasamo_init failed: {:?}",
                    last_error()
                );
                let (job_tx, job_rx) = mpsc::channel::<RuntimeJob>();
                outcome_tx
                    .send(Some(job_tx))
                    .expect("send runtime init outcome");
                // Own the Compositor's apartment for the whole process and
                // act as the marshalling executor: run each test body that
                // touches the Compositor here, one at a time, and never exit.
                // `recv` blocks between tests and ends only when every sender
                // (every test binary thread) is gone, i.e. at process exit.
                while let Ok(job) = job_rx.recv() {
                    job();
                }
            })
            .expect("spawn owning runtime thread");
        match outcome_rx.recv().expect("receive runtime init outcome") {
            Some(job_tx) => Runtime::Ready(job_tx),
            None => Runtime::CompositorUnavailable,
        }
    })
}

/// Run a Compositor-touching test body on the single runtime-owning thread,
/// or skip it when the Compositor is locally unavailable.
///
/// # Why one entry point bundles init + skip + marshalling + panic relay
///
/// This deliberately folds four things together: (1) one-time runtime init
/// on the owning thread, (2) the skip policy (skip on a dev laptop, *fail*
/// on GitHub Actions — CLAUDE.md §Testing rules), (3) marshalling the test
/// body onto that owning thread, and (4) relaying a panic (a failed
/// assertion) back across the thread boundary so `#[test]` still reports it.
///
/// These are *not* one responsibility in the strict SRP sense: their change
/// drivers differ — the skip rule answers to CI/testing policy, the
/// marshalling to the COM apartment model, the panic relay to test-framework
/// integration. The justification for a single entry point is not "one
/// responsibility" but **shared change/deletion locality plus coupling
/// avoidance**:
///
/// - All four exist solely to run a Compositor test body under libtest's
///   per-test-thread spawning + the Compositor's STA-apartment affinity.
///   They are added, changed, and *deleted together*: the moment that
///   precondition goes away (a process-per-test runner like `cargo nextest`,
///   or libtest no longer spawning a thread per test) the whole helper is
///   removed at once — see "When this helper can be deleted" above.
/// - Splitting the skip check back out to each caller re-introduces a real
///   coupling: the caller's skip check and this function both depend on the
///   same process-global init outcome, so the two calls would have to agree
///   to be correct — and the skip branch must live here anyway, since the
///   body cannot run when the Compositor is unavailable. Splitting buys no
///   independence and would leave a now-vestigial COM init on the test
///   thread, which step 1 exists to eliminate.
///
/// On panic the body's payload is caught on the owning thread and emitted as
/// a fresh panic on the calling libtest thread. This preserves the assertion
/// message in libtest / CI output while allowing the owning executor to serve
/// later tests.
pub fn run_on_owning_runtime_thread_or_skip<F>(test_name: &str, body: F)
where
    F: FnOnce() + Send + 'static,
{
    match ensure_runtime() {
        Runtime::Ready(job_tx) => {
            let (done_tx, done_rx) = mpsc::channel();
            job_tx
                .send(Box::new(move || {
                    // Catch the body's panic on the owning thread so the
                    // executor loop survives it, then hand its message back
                    // to the waiting test thread for a hook-visible panic.
                    let outcome = catch_unwind(AssertUnwindSafe(body))
                        .map_err(|payload| panic_payload_message(payload.as_ref()));
                    let _ = done_tx.send(outcome);
                }))
                .expect("owning runtime thread has stopped accepting jobs");
            match done_rx
                .recv()
                .expect("owning runtime thread dropped the test body without reporting")
            {
                Ok(()) => {}
                Err(message) => panic!("{test_name}: owner-thread test body panicked: {message}"),
            }
        }
        Runtime::CompositorUnavailable => {
            assert!(
                !github_actions(),
                "{test_name} cannot skip on GitHub Actions: \
                 runtime compositor unavailable"
            );
            eprintln!("skipping {test_name}: runtime compositor unavailable");
        }
    }
}

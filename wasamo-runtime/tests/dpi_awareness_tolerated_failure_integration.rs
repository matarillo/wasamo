//! Mock-free Windows integration evidence for M4-Phase 1 T9 — DD-M4-P1-001's
//! **failure handling**, option F2, fired end-to-end.
//!
//! This is the trap-#4 artifact for the one authored branch the task adds: the
//! path taken when the runtime's Per-Monitor-Aware V2 declaration does **not**
//! take effect because the host already set the process's awareness.
//!
//! # How the branch is reached, and why that is not a trick
//!
//! [plan.md](../../process/milestone-4/phase-1/implementation/plan.md) §T9 was
//! pre-authorised to record "cannot be fired, because process DPI awareness is
//! one-shot per process" as a stated limit. It is one-shot per *process*, and
//! a test binary is a process — so this file simply **is** the host
//! DD-M4-P1-001 describes. It declares its own awareness before `wasamo_init`
//! runs anywhere in this process, exactly as a host with an application
//! manifest or its own `SetProcessDpiAwarenessContext` call would, and the
//! runtime then takes the real `ERROR_ACCESS_DENIED` on the shipped path. No
//! seam, no injection, no mocked failure.
//!
//! `DPI_AWARENESS_CONTEXT_SYSTEM_AWARE` rather than V2 is deliberate: it makes
//! the assertions below discriminate. If the pre-declaration were V2, the
//! effective-level check could not tell "the host's declaration survived" from
//! "the runtime overrode it", which is the whole point.
//!
//! # What this pins that prose cannot
//!
//! Three properties DD-M4-P1-001 §Failure handling argues for, as behaviour:
//!
//! 1. `wasamo_init` returns `WASAMO_OK` — option F1 (hard error) would turn a
//!    host that did the right thing into a runtime that refuses to start.
//! 2. The outcome is **disclosed**, through the thread-local last-error string
//!    (abi_spec §4.1). Without this the host has no way to learn that AC7's
//!    crispness guarantee does not hold in its process.
//! 3. The host's level survives. The runtime defers rather than overriding.
//!
//! Property 2 is also the regression pin for the defect that made the
//! disclosure unreachable: `wasamo_init` used to clear the thread-local on its
//! success arm, *after* `runtime::init` had written the diagnostic into it.
//!
//! # Why this is its own binary
//!
//! Its sibling `dpi_awareness_declaration_integration.rs` asserts that the
//! level in force is V2. That is only evidence in a process where nothing but
//! the runtime declared anything — which this file, by construction, is not.
//! The two facts cannot share a process.
//!
//! # Stated limit
//!
//! One binary can observe one outcome. A process that has watched the runtime
//! declare successfully can never watch it fail, and this one can never watch
//! it succeed, so the success and failure halves of the branch are two
//! artifacts and no run asserts both. The pure-logic selection between them —
//! `runtime::declaration_diagnostic` — is unit-tested in both directions in
//! one binary precisely because that is the part which does not have to
//! inherit this limit.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::ffi::CStr;

use wasamo_runtime::ffi;

use windows::Win32::UI::HiDpi::{
    AreDpiAwarenessContextsEqual, GetThreadDpiAwarenessContext, SetProcessDpiAwarenessContext,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, DPI_AWARENESS_CONTEXT_SYSTEM_AWARE,
};

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

/// A host that declared its own awareness keeps it, is not failed, and is told.
#[test]
fn a_host_that_already_declared_keeps_its_level_and_is_told_ours_did_not_take() {
    // Before `wasamo_init` runs anywhere in this process — `ensure_runtime`
    // inside the helper below is what first calls it, and this binary holds
    // exactly one test, so the ordering is not a race but a sequence.
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_SYSTEM_AWARE) }
        .expect("the test host declares its own awareness first; nothing has set it yet");

    run_on_owning_runtime_thread_or_skip("tolerated declaration failure", move || {
        // Read the diagnostic before anything else: the helper has just run
        // `wasamo_init` on this thread, the string is thread-local, and every
        // other ABI entry point clears it on success (abi_spec §4.1 — "valid
        // until the next ABI call on that thread").
        let diagnostic = last_error();

        let context = unsafe { GetThreadDpiAwarenessContext() };
        let is_system =
            unsafe { AreDpiAwarenessContextsEqual(context, DPI_AWARENESS_CONTEXT_SYSTEM_AWARE) }
                .as_bool();
        let is_v2 = unsafe {
            AreDpiAwarenessContextsEqual(context, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
        }
        .as_bool();

        // Property 1 — `wasamo_init` returned WASAMO_OK. Asserted by arriving
        // here at all: `run_on_owning_runtime_thread_or_skip` asserts the
        // status itself and would have failed the test before the body ran,
        // and the Compositor-unavailable path skips instead of running this.
        // Recorded rather than left implicit, because "the body ran" is
        // evidence only once it is said what makes it so.

        // Property 3 — the host's level survived; the runtime deferred.
        assert!(
            is_system && !is_v2,
            "the runtime must defer to a host that declared its own awareness, \
             not override it (system-aware: {is_system}, V2: {is_v2})"
        );

        // Property 2 — the outcome was disclosed. This is the assertion that
        // goes red if `wasamo_init` clears the thread-local on its success arm
        // instead of on entry.
        let diagnostic = diagnostic.expect(
            "a declaration that did not take effect must be disclosed through \
             wasamo_last_error_message (DD-M4-P1-001 §Failure handling, \
             abi_spec §4.1). Reading None here means either the diagnostic was \
             never recorded or wasamo_init cleared it after runtime::init \
             wrote it",
        );
        assert!(
            diagnostic.contains("did not take effect"),
            "the disclosure must say what did not happen: {diagnostic}"
        );
        assert!(
            diagnostic.contains("0x80070005"),
            "the disclosure must name the HRESULT, which is the one thing a \
             developer can grep for: {diagnostic}"
        );
        assert!(
            diagnostic.contains("crispness"),
            "the disclosure must say what is given up, or a host cannot tell \
             that AC7's guarantee does not hold in its process: {diagnostic}"
        );
    });
}

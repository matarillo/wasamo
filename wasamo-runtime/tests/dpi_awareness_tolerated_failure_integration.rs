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
//! DD-M4-P1-001 describes. It sets the process's awareness before
//! `wasamo_init` runs anywhere in this process, exactly as a host with an
//! application manifest or its own `SetProcessDpiAwarenessContext` call would,
//! and the runtime then takes the real `ERROR_ACCESS_DENIED` on the shipped
//! path. No seam, no injection, no mocked failure.
//!
//! On a session where something *else* got there first, that is still true and
//! the test still holds — see the section below on doing OS work before the
//! guard. The host in the story stops being this file and becomes whatever set
//! the awareness; the runtime's behaviour under test is identical.
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
//! # This file does OS work before the skip guard, and that cost a defect
//!
//! The pre-declaration has to precede `wasamo_init`, and `wasamo_init` is
//! called by the shared helper — so this is the one binary in the suite that
//! touches the OS *before* the Compositor-unavailable skip decision. The first
//! version `expect`ed that call to succeed, on the reasoning that it needs no
//! Compositor. **Measured on the owner's guard-verification run: it returned
//! `ERROR_ACCESS_DENIED`**, because the process's awareness had already been
//! set before any test code ran, and this binary *failed* on the environment
//! where every other binary skips.
//!
//! The premise had been written as a claim about code ordering — one test per
//! binary, so no race — which is true and beside the point: it establishes that
//! no *test* set the awareness and says nothing about the OS, the loader, or a
//! compatibility shim. So the pre-declaration's result is now discarded, the
//! level actually in force is read back, and every assertion sits behind the
//! guard. What the test needs is not that *its* call won, but that the process
//! entered `wasamo_init` at some level other than V2 — which either outcome
//! establishes.
//!
//! # Stated limits
//!
//! 1. One binary can observe one outcome. A process that has watched the
//!    runtime declare successfully can never watch it fail, and this one can
//!    never watch it succeed, so the success and failure halves of the branch
//!    are two artifacts and no run asserts both. The pure-logic selection
//!    between them — `runtime::declaration_diagnostic` — is unit-tested in both
//!    directions in one binary precisely because that is the part which does
//!    not have to inherit this limit.
//! 2. **Why the process's awareness was already set on the owner's session is
//!    not identified**, and nothing here claims it. What is established is that
//!    it is environmental: the same on-disk executable passes locally and
//!    failed there. The test no longer depends on the answer.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::ffi::CStr;

use wasamo_runtime::ffi;

use windows::Win32::UI::HiDpi::{
    AreDpiAwarenessContextsEqual, GetThreadDpiAwarenessContext, SetProcessDpiAwarenessContext,
    DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, DPI_AWARENESS_CONTEXT_SYSTEM_AWARE,
    DPI_AWARENESS_CONTEXT_UNAWARE,
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

/// The awareness level in force, as a name that can cross a thread boundary.
///
/// A `DPI_AWARENESS_CONTEXT` is an opaque handle and is not `Send`; the test
/// body runs on the runtime's owning thread, so the level is resolved to a
/// name on each side and the names are compared.
fn level_name(context: DPI_AWARENESS_CONTEXT) -> &'static str {
    let eq = |other| unsafe { AreDpiAwarenessContextsEqual(context, other) }.as_bool();
    if eq(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) {
        "PER_MONITOR_AWARE_V2"
    } else if eq(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE) {
        "PER_MONITOR_AWARE_V1"
    } else if eq(DPI_AWARENESS_CONTEXT_SYSTEM_AWARE) {
        "SYSTEM_AWARE"
    } else if eq(DPI_AWARENESS_CONTEXT_UNAWARE) {
        "UNAWARE"
    } else {
        "UNRECOGNISED"
    }
}

/// A host that declared its own awareness keeps it, is not failed, and is told.
#[test]
fn a_host_that_already_declared_keeps_its_level_and_is_told_ours_did_not_take() {
    // Try to be the host that declares first. `wasamo_init` has not run
    // anywhere in this process yet — `ensure_runtime` inside the helper below
    // is what first calls it, and this binary holds exactly one test, so that
    // much is a sequence rather than a race.
    //
    // **Deliberately not `expect`, and the first version of this test got that
    // wrong** (measured on the owner's guard-verification run). This call sits
    // *before* the Compositor-unavailable skip decision, because it has to
    // precede `wasamo_init` — and it is not guaranteed to succeed. On the
    // owner's session it returned `ERROR_ACCESS_DENIED`: the process's
    // awareness had already been set by something outside this test, before a
    // line of test code ran. Panicking there made this binary *fail* on
    // precisely the environment where every other binary skips, which is the
    // defect the guard-verification rule exists to catch, and it caught it.
    //
    // The mechanism is not identified and is not claimed. The correction does
    // not depend on identifying it: what the assertions below need is not that
    // *this call* won, but that the process's awareness was set to something
    // other than V2 before `wasamo_init` ran. Either outcome establishes that
    // — ours succeeding, or ours losing to whatever got there first — so the
    // premise is **read back rather than assumed**, and every assertion sits
    // behind the skip guard where it belongs.
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_SYSTEM_AWARE) };
    let before = level_name(unsafe { GetThreadDpiAwarenessContext() });

    run_on_owning_runtime_thread_or_skip("tolerated declaration failure", move || {
        // Read the diagnostic before anything else: the helper has just run
        // `wasamo_init` on this thread, the string is thread-local, and every
        // other ABI entry point clears it on success (abi_spec §4.1 — "valid
        // until the next ABI call on that thread").
        let diagnostic = last_error();
        let after = level_name(unsafe { GetThreadDpiAwarenessContext() });

        // Property 1 — `wasamo_init` returned WASAMO_OK. Asserted by arriving
        // here at all: `run_on_owning_runtime_thread_or_skip` asserts the
        // status itself and would have failed the test before the body ran,
        // and the Compositor-unavailable path skips instead of running this.
        // Recorded rather than left implicit, because "the body ran" is
        // evidence only once it is said what makes it so.

        // The premise, read back rather than assumed. If this fires, the
        // fixture has stopped discriminating: with V2 already in force the
        // runtime's declaration fails and the diagnostic is still recorded,
        // so the branch is exercised — but "deferred" and "overrode" become
        // the same picture and property 3 below means nothing.
        assert_ne!(
            before, "PER_MONITOR_AWARE_V2",
            "this test needs the process to enter wasamo_init at some level \
             *other* than V2, so that deferring and overriding look different. \
             Something set V2 before the test ran"
        );

        // Property 3 — the pre-existing level survived; the runtime deferred
        // rather than overriding. Stated against whatever was actually in
        // force, not against SYSTEM_AWARE specifically, because who won the
        // race above is not what is under test.
        assert_eq!(
            after, before,
            "the runtime must defer to a process whose awareness was already \
             set, not override it"
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

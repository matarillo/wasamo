//! Mock-free Windows integration evidence for M4-Phase 1 T9 — DD-M4-P1-001's
//! first verification line: **the awareness level actually in force**.
//!
//! A real process, a real window, and the OS asked what that window's DPI
//! awareness context is. Nothing is mocked and nothing is stubbed.
//!
//! # What this binary asserts, and why it is not "we called the function"
//!
//! The failure this exists to catch is the declaration *silently not taking
//! effect* — because something locked the process's awareness before
//! `runtime::init` reached its call, because the call was moved above the
//! one-shot guard and re-declared into `ERROR_ACCESS_DENIED`, or because a
//! future edit inserted OS work ahead of it. Every one of those leaves the
//! call site present and correct-looking, so a test that checks the call's
//! return value, or that a particular function was called, catches none of
//! them. `GetWindowDpiAwarenessContext` + `AreDpiAwarenessContextsEqual` reads
//! the state instead of the code.
//!
//! # Why this is its own binary
//!
//! Process DPI awareness is one-shot **per process**, and its sibling
//! `dpi_awareness_tolerated_failure_integration.rs` works by setting it
//! *before* `wasamo_init` runs. In a shared process that pre-declaration would
//! make the assertion below pass without the runtime having declared anything
//! — the exact false green this file exists to prevent. The two facts
//! DD-M4-P1-001 names are therefore two processes.
//!
//! It also carries the readback for a second claim that had none: that the
//! declaration sitting *below* `runtime::init`'s existing one-shot is why no
//! new guard is needed (T1 finding F-10). A second `wasamo_init` must leave no
//! diagnostic, because it must not re-declare.
//!
//! # Stated limit
//!
//! This says the level is in force; it does not say the **runtime** is what
//! put it there. In this process nothing else declares anything, so the
//! runtime is the only candidate — but that is a property of the fixture, and
//! a host that pre-declared V2 itself would produce the same reading. The
//! sibling binary is what distinguishes "declared" from "found already
//! declared", by observing the case where the runtime's call loses.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::ffi::CStr;
use std::ptr;

use wasamo_runtime::ffi;

use windows::Win32::UI::HiDpi::{
    AreDpiAwarenessContextsEqual, GetWindowDpiAwarenessContext,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, DPI_AWARENESS_CONTEXT_SYSTEM_AWARE,
    DPI_AWARENESS_CONTEXT_UNAWARE,
};

/// The declared level is Per-Monitor-Aware V2, read off a live window.
#[test]
fn a_windows_effective_awareness_context_is_per_monitor_aware_v2() {
    run_on_owning_runtime_thread_or_skip("effective DPI awareness level", move || {
        unsafe {
            let title = "T9 awareness";
            let mut window: *mut ffi::WasamoWindow = ptr::null_mut();
            let status = ffi::wasamo_window_create(
                title.as_ptr() as *const std::ffi::c_char,
                title.len(),
                800,
                600,
                &mut window as *mut *mut ffi::WasamoWindow,
            );
            assert_eq!(status, ffi::WASAMO_OK, "wasamo_window_create");
            assert!(!window.is_null());

            let context = GetWindowDpiAwarenessContext((*window).hwnd);
            let is_v2 =
                AreDpiAwarenessContextsEqual(context, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)
                    .as_bool();
            // The two levels a defect would land on instead, read in the same
            // breath so a failure says *what* is in force rather than only
            // that V2 is not. `UNAWARE` is the pre-T9 status quo — the state a
            // declaration that never ran leaves behind — and `SYSTEM_AWARE` is
            // what DD-M4-P1-001 rejected as option L3.
            let is_unaware =
                AreDpiAwarenessContextsEqual(context, DPI_AWARENESS_CONTEXT_UNAWARE).as_bool();
            let is_system =
                AreDpiAwarenessContextsEqual(context, DPI_AWARENESS_CONTEXT_SYSTEM_AWARE).as_bool();

            ffi::wasamo_window_destroy(window);

            // A second `wasamo_init` must not re-declare. This is the readback
            // for T1's finding F-10 — "the existing one-shot is sufficient; T9
            // adds no new guard" — which until now was a structural claim with
            // nothing observing it. Placing the declaration *above* the
            // `RUNTIME.get().is_some()` early return leaves the level in force
            // correct and this call still returning WASAMO_OK, so the only
            // reachable symptom is the spurious diagnostic: the process would
            // take ERROR_ACCESS_DENIED against its own earlier, correct
            // declaration and report it to a host that did nothing wrong.
            let second = ffi::wasamo_init();
            let second_diagnostic = {
                let ptr = ffi::wasamo_last_error_message();
                if ptr.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
                }
            };

            assert_eq!(
                second,
                ffi::WASAMO_OK,
                "a second wasamo_init is a no-op that succeeds"
            );
            assert_eq!(
                second_diagnostic, None,
                "a second wasamo_init must not re-declare: the declaration sits \
                 below the runtime's existing one-shot, so there is nothing to \
                 report. A diagnostic here means the call was moved above the \
                 early return and the process denied its own re-declaration"
            );
            assert!(
                is_v2,
                "the level in force must be Per-Monitor-Aware V2 (unaware: \
                 {is_unaware}, system-aware: {is_system}). DD-M4-P1-001 puts \
                 the declaration in runtime::init so no host ships a manifest; \
                 if this reads unaware, the declaration did not run or did not \
                 take effect, and every scale factor in the process is 1 while \
                 DWM stretches the finished window"
            );
        }
    });
}

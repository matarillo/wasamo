//! Pure-logic integration tests for DD-M2-P6-005:
//!   - `WASAMO_ERR_WRONG_THREAD` for cross-thread calls
//!   - `wasamo_load_ui` resource-resolution dispatch (PATH and MEMORY)
//!   - argument validation (null pointers, zero length, unknown type)
//!
//! These exercise the guards and dispatch logic without standing up a live
//! Compositor / DispatcherQueueController. The owning thread is installed
//! via the test-only `__install_owning_thread_for_test` seam in the
//! runtime module (see `runtime.rs`), so the tests do not depend on a
//! running OS message pump or COM apartment state — those concerns are
//! covered by the GUI checkpoint (CLAUDE.md testing rules).
//!
//! All assertions live in a single `#[test]` so the OnceLock-based
//! owning-thread state is set once on the test runner thread; a child
//! thread spawned mid-test then exercises the cross-thread path.

#![cfg(windows)]

use std::ffi::c_void;
use std::ptr;

use wasamo_runtime::ffi;

#[test]
fn dd_m2_p6_005_wasamo_load_ui_and_thread_affinity() {
    // Establish this thread as the runtime's owning UI thread without
    // pulling in DispatcherQueueController / Compositor.
    ffi::__install_owning_thread_for_test();

    // ── 1. Cross-thread call returns WASAMO_ERR_WRONG_THREAD ──────────────
    let join = std::thread::spawn(|| {
        let mut count: usize = 7; // sentinel; must NOT be touched on error
        let status =
            unsafe { ffi::wasamo_widget_child_count(ptr::null_mut(), &mut count as *mut usize) };
        assert_eq!(status, ffi::WASAMO_ERR_WRONG_THREAD);
        assert_eq!(count, 7, "out_count must not be touched on wrong-thread");

        // wasamo_load_ui from the wrong thread must also fail without
        // dispatching into PATH / MEMORY logic.
        let mut out: *mut ffi::WasamoWindow = ptr::null_mut();
        let status = unsafe {
            ffi::wasamo_load_ui(
                ffi::WASAMO_LOAD_PATH,
                b"x".as_ptr() as *const c_void,
                1,
                &mut out as *mut *mut ffi::WasamoWindow,
            )
        };
        assert_eq!(status, ffi::WASAMO_ERR_WRONG_THREAD);
        assert!(out.is_null(), "out_root must remain null on wrong-thread");
    });
    join.join().expect("child thread panicked");

    // ── 2. Argument validation on the owning thread ───────────────────────
    let mut out: *mut ffi::WasamoWindow = ptr::null_mut();

    // null data
    let status = unsafe {
        ffi::wasamo_load_ui(
            ffi::WASAMO_LOAD_PATH,
            ptr::null(),
            16,
            &mut out as *mut *mut ffi::WasamoWindow,
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_INVALID_ARG);
    assert!(out.is_null());

    // zero data_len
    let dummy: u8 = 0;
    let status = unsafe {
        ffi::wasamo_load_ui(
            ffi::WASAMO_LOAD_MEMORY,
            &dummy as *const u8 as *const c_void,
            0,
            &mut out as *mut *mut ffi::WasamoWindow,
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_INVALID_ARG);

    // unknown WasamoLoadType
    let bytes = b"x";
    let status = unsafe {
        ffi::wasamo_load_ui(
            42,
            bytes.as_ptr() as *const c_void,
            bytes.len(),
            &mut out as *mut *mut ffi::WasamoWindow,
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_INVALID_ARG);

    // null out_root
    let status = unsafe {
        ffi::wasamo_load_ui(
            ffi::WASAMO_LOAD_MEMORY,
            bytes.as_ptr() as *const c_void,
            bytes.len(),
            ptr::null_mut(),
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_INVALID_ARG);

    // ── 3. PATH-mode dispatch: missing file → RUNTIME ─────────────────────
    let path = b"this_path_does_not_exist_for_wasamo_load_ui_test.ui";
    let status = unsafe {
        ffi::wasamo_load_ui(
            ffi::WASAMO_LOAD_PATH,
            path.as_ptr() as *const c_void,
            path.len(),
            &mut out as *mut *mut ffi::WasamoWindow,
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_RUNTIME);
    assert!(out.is_null());

    // PATH-mode rejects non-UTF-8 path bytes as INVALID_ARG.
    let bad = [0xff_u8, 0xfe, 0xfd];
    let status = unsafe {
        ffi::wasamo_load_ui(
            ffi::WASAMO_LOAD_PATH,
            bad.as_ptr() as *const c_void,
            bad.len(),
            &mut out as *mut *mut ffi::WasamoWindow,
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_INVALID_ARG);

    // ── 4. MEMORY-mode dispatch: malformed and non-UTF-8 → IR_MALFORMED ──
    let blob = b"not a wasamo IR header at all";
    let status = unsafe {
        ffi::wasamo_load_ui(
            ffi::WASAMO_LOAD_MEMORY,
            blob.as_ptr() as *const c_void,
            blob.len(),
            &mut out as *mut *mut ffi::WasamoWindow,
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_IR_MALFORMED);
    assert!(out.is_null());

    let non_utf8 = [0xff_u8, 0xfe, 0xfd];
    let status = unsafe {
        ffi::wasamo_load_ui(
            ffi::WASAMO_LOAD_MEMORY,
            non_utf8.as_ptr() as *const c_void,
            non_utf8.len(),
            &mut out as *mut *mut ffi::WasamoWindow,
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_IR_MALFORMED);

    let non_string_title = b";wasamo-ir v0\ncomponent C inherits W {\n\
        host prop title = 3\n\
        node VStack {}\n\
    }";
    let status = unsafe {
        ffi::wasamo_load_ui(
            ffi::WASAMO_LOAD_MEMORY,
            non_string_title.as_ptr() as *const c_void,
            non_string_title.len(),
            &mut out as *mut *mut ffi::WasamoWindow,
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_IR_MALFORMED);
    assert!(out.is_null());

    // DD-M3-P6-008: the runtime mirrors the compiler host catalog on value
    // shape too — a typed-scalar literal on `backdrop` / `theme` (which take a
    // keyword identifier) is malformed at the ABI boundary, not silently
    // accepted from hand-crafted textual IR.
    let typed_literal_backdrop = b";wasamo-ir v0\ncomponent C inherits W {\n\
        host prop backdrop = 3\n\
        node VStack {}\n\
    }";
    let status = unsafe {
        ffi::wasamo_load_ui(
            ffi::WASAMO_LOAD_MEMORY,
            typed_literal_backdrop.as_ptr() as *const c_void,
            typed_literal_backdrop.len(),
            &mut out as *mut *mut ffi::WasamoWindow,
        )
    };
    assert_eq!(status, ffi::WASAMO_ERR_IR_MALFORMED);
    assert!(out.is_null());

    // ── 5. wasamo_last_error_message round-trips a description ────────────
    let msg_ptr = ffi::wasamo_last_error_message();
    assert!(!msg_ptr.is_null(), "last-error message should be populated");
    let msg = unsafe { std::ffi::CStr::from_ptr(msg_ptr) }
        .to_str()
        .expect("last-error message must be UTF-8");
    assert!(
        msg.contains("wasamo_load_ui"),
        "last-error should mention the offending function: {msg}"
    );
}

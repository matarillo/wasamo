//! Mock-free Windows integration evidence for M4-Phase 2 T9, stage 4
//! ([DD-M4-P2-005](../../process/milestone-4/phase-2/decisions/dd-m4-p2-005-dsl-handler-surface.md)
//! §Per-item handlers,
//! [DD-M4-P2-001](../../process/milestone-4/phase-2/decisions/dd-m4-p2-001-event-routing-model.md)
//! §Drain coupling): a per-item `signal_handler` inside a `for` body,
//! driven by real `WM_LBUTTONUP` messages through the real window
//! procedure, over a widget tree built from real `.ui` source through
//! `wasamoc` -> IR -> the loader, with live widget state read back
//! afterwards. Nothing is mocked. Stages 1-3 (`b9e12fd`, `ed5117b`,
//! `38f4daf`) landed the compiler gate, the loader gate and the runtime
//! evaluation of binder reads at invocation time; this file is the
//! discriminating evidence plan §T9 names for stage 4.
//!
//! # What each fixture is for
//!
//! - [`a_per_item_handler_reads_the_index_of_the_row_that_was_clicked`] (F1)
//!   — the baseline claim: a per-item handler's `index` read resolves to
//!   the *clicked* row's position, not some fixed or coincidental value.
//!   Two different rows (0 and 2) must write two different reads — a
//!   fixture that clicked one row twice would not constrain anything
//!   (implementation-gates trap #3; the phase has a recorded case,
//!   T8 close gate W9, of a shared encoder passing 1,201 tests because
//!   both sides consumed it symmetrically).
//! - [`a_per_item_handler_reads_the_current_item_after_a_same_length_collection_reset`]
//!   (F2) — **the discriminating fixture** DD-M4-P2-005 §Technical risk
//!   re-evaluation names: "a test that only ever clicks a freshly
//!   generated row cannot distinguish invocation-time resolution from
//!   generation-time capture." See F2's own doc comment for why the
//!   same-length reset is what makes the distinction observable at all.
//! - [`a_per_item_handler_on_a_surviving_row_keeps_its_position_after_a_tail_removal`]
//!   (F3) — start-gate fact 4 (log.md): a surviving row's position never
//!   moves under tail-only reconciliation. Clicks the row immediately
//!   before the removed tail — the position most exposed to an off-by-one
//!   in the reconciler's retained-vs-removed split.
//! - [`a_removed_rows_handler_is_released_with_its_subtree`] (F4) — the
//!   registration-lifecycle evidence. DD-M4-P2-005 says this "fails
//!   silently in one direction": a handler left registered against a
//!   dropped subtree shows up in no rendered frame. A host listener
//!   connected through the real ABI (`wasamo_signal_connect`) is shown to
//!   fire while its row is alive (agreement leg) and never again once the
//!   row is removed (difference leg), with the registration's own
//!   `destroy_fn` firing exactly once, synchronously with the removal.
//! - [`a_per_item_handler_that_shortens_its_own_collection_reports_the_stale_item_rather_than_a_wrong_one`]
//!   (F5) — the reachable out-of-range boundary, start-gate fact 9: a
//!   handler body `{ xs = xs.drop-last(); root.n = value; }` clicked on
//!   the *last* row. The collection write drains synchronously inside the
//!   statement, so by the time `value` is read the position is past the
//!   end; `EvalError::ItemOutOfRange` fires and the assignment does not
//!   happen. Also carries the DD-M4-P2-001 ordering measurement — see
//!   below.
//!
//! # The DD-M4-P2-001 ordering measurement
//!
//! DD-M4-P2-001 §Drain coupling gives the reason residual 1 (cycle
//! detection) does not fire for a per-item handler that mutates its own
//! collection: "the handler has already returned when regeneration runs."
//! Start-gate fact 10 (log.md) recorded that this does not match the
//! runtime — `register_for_loop_binding` installs an ordinary reactive
//! `EffectHandle`, so regeneration runs *inside* `Signal::set`, during the
//! handler's own statement — and asked this task to re-measure it against
//! the shipped fixture rather than edit the Accepted decision.
//!
//! F5 is that fixture. It measures the ordering with a `destroy_fn`
//! (`ClickDestroyCounters`, below) connected to the clicked row's own
//! widget pointer through the real ABI, the same technique
//! `iteration_mutation_integration.rs`'s `DESTROY_COUNT` uses conceptually
//! (a destroy callback that counts subtree teardown), adapted to a
//! per-instance heap counter rather than a process-global static (see
//! "Why this file has no `test_lock`" below). **Observed:** after the
//! single synchronous `send_click` that runs both of the handler's
//! statements, `destroyed == 1` (the clicked row's own subtree was torn
//! down) and `n` still holds its `-1` sentinel (the second statement's
//! item read failed rather than reading the stale `30`). Both facts are
//! visible from *one* call, so the subtree's destruction and the second
//! statement's failed read happened inside the same synchronous span —
//! i.e. regeneration ran *before* the handler's second statement, not
//! after the handler returned. This **confirms DD-M4-P2-001's
//! conclusion** (no cycle: regeneration re-invokes no handler) while
//! **contradicting its stated reason** (the handler has not already
//! returned when regeneration runs) — exactly start-gate fact 10's
//! prediction. This file does not edit DD-M4-P2-001 (Accepted; the lead
//! dispositions divergences) and asserts only the observable state
//! (`destroyed == 1`, `n == -1`), not any wording from the decision
//! record itself.
//!
//! # Helpers copied from `event_routing_integration.rs`
//!
//! Cargo compiles every `tests/*.rs` file as its own crate, so nothing in
//! that file is importable here. `lower_ui_to_ir`, `load_window`,
//! `client_extent`, `display_limits`, `window_rect`, `frame_thickness`,
//! `send_dpi_change_to_client`, `normalise_to_reference_baseline`,
//! `send_click`, `click_and_drain` and `read_counter` below are copied
//! verbatim (per that file's own stated convention, itself copied from
//! `hit_resolution_integration.rs`) from `event_routing_integration.rs`;
//! see that file's module header for the full reasoning behind each.
//!
//! This file's own additions are [`read_text`] (the string-valued twin of
//! `read_counter`, needed by F2's discriminator) and
//! `ClickDestroyCounters` / `count_click` / `count_destroy` /
//! `connect_click_and_destroy_counters` (a host `"clicked"` listener
//! connected through the real ABI that counts both its own firings *and*
//! its `destroy_fn` firing, used by F4 and F5). The pairing is necessary
//! rather than decorative: `wasamo_signal_connect`'s `callback` and
//! `destroy_fn` share one `user_data` pointer (`wasamo-runtime/src/abi.rs`),
//! so a listener that needs to count both independently boxes them
//! together rather than allocating two pointers only one of which the ABI
//! would ever pass to a given callback.
//!
//! # Why this file has no `test_lock`
//!
//! Three sibling files (`iteration_mutation_integration.rs`,
//! `iteration_static_integration.rs`, `conditional_toggle_integration.rs`)
//! serialise their fixtures on a file-local `test_lock()` because they
//! hold **process-global static** mutable measurement state (a `static
//! DESTROY_COUNT: AtomicUsize`) shared across every `#[test]` in the file.
//! This file follows `event_routing_integration.rs` instead (six fixtures,
//! no `test_lock`): every counter here is a **per-instance heap
//! allocation** (`Box::into_raw`, freed at the end of the owning fixture),
//! never a `static`, so there is no cross-fixture mutable state for a lock
//! to protect. (`run_on_owning_runtime_thread_or_skip`'s single owning
//! thread already serialises fixture *bodies* end to end regardless — see
//! `tests/common/mod.rs` — so the lock, where it is used, is about
//! protecting a shared static's reset/read pairing across that thread's
//! job queue, not about the Compositor itself.)
//!
//! # The client stays small (M4-Phase 1 T8 finding)
//!
//! Every fixture normalises to 96 DPI at an explicitly chosen 360x240
//! physical client — the same ceiling `hit_resolution_integration.rs`'s
//! module header records (a hosted CI desktop failed a larger request
//! twice). No fixture in this file ever changes scale, so 360x240 is also
//! each fixture's only client extent. Every row `Box` uses `aspect: 20:1`
//! (a full-width 360 DIP row is 18 DIP tall) so that up to six stacked
//! rows/controls (F3's largest tree) comfortably fit the 240 DIP client
//! height with the two `Text` readback rows included.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr;

use wasamo_runtime::{ffi, WidgetNode};

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, PostMessageW, PostQuitMessage, SendMessageW,
    SM_CXMAXTRACK, SM_CXSCREEN, SM_CYMAXTRACK, SM_CYSCREEN, WM_DPICHANGED, WM_LBUTTONUP,
};

/// The reference DPI every fixture normalises to before measuring. See
/// `hit_resolution_integration.rs`'s module header (Phase 1 T9 finding
/// F-47) for why this is reached deliberately rather than assumed.
const REFERENCE_DPI: u32 = 96;

/// The physical (== DIP, since no fixture here changes scale) client every
/// fixture normalises to. See this file's module header, "The client stays
/// small".
const CLIENT_W: i32 = 360;
const CLIENT_H: i32 = 240;

// ── Copied verbatim from `event_routing_integration.rs` ────────────────────

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<per-item-handler-integration>";
    let tokens = lexer::tokenize(src, path).expect("lex failed");
    let ast = parser::parse(&tokens, path).expect("parse failed");
    let checked = check::check(&ast, path);
    assert!(
        !checked.has_errors(),
        "check errors: {:?}",
        checked.diagnostics
    );
    emit::emit(&lower::lower(&ast, &checked.namespace))
}

unsafe fn load_window(ir: &str) -> *mut ffi::WasamoWindow {
    let mut window: *mut ffi::WasamoWindow = ptr::null_mut();
    let status = ffi::wasamo_load_ui(
        ffi::WASAMO_LOAD_MEMORY,
        ir.as_ptr() as *const c_void,
        ir.len(),
        &mut window as *mut *mut ffi::WasamoWindow,
    );
    assert_eq!(status, ffi::WASAMO_OK, "{:?}", window);
    assert!(!window.is_null(), "wasamo_load_ui must return a window");
    window
}

unsafe fn client_extent(hwnd: HWND) -> (i32, i32) {
    let mut rect = RECT::default();
    GetClientRect(hwnd, &mut rect).expect("GetClientRect");
    (rect.right - rect.left, rect.bottom - rect.top)
}

/// Diagnostic-only, printed into a failing assertion so a CI runner that
/// cannot honour a requested client explains itself instead of failing
/// silently on a rectangle nobody chose (M4-Phase 1 T8/T9 precedent).
unsafe fn display_limits() -> (i32, i32, i32, i32) {
    (
        GetSystemMetrics(SM_CXSCREEN),
        GetSystemMetrics(SM_CYSCREEN),
        GetSystemMetrics(SM_CXMAXTRACK),
        GetSystemMetrics(SM_CYMAXTRACK),
    )
}

unsafe fn window_rect(hwnd: HWND) -> RECT {
    let mut rect = RECT::default();
    GetWindowRect(hwnd, &mut rect).expect("GetWindowRect");
    rect
}

/// The non-client frame, as `outer - client` on each axis. Measured live
/// rather than derived from a DPI table.
unsafe fn frame_thickness(hwnd: HWND) -> (i32, i32) {
    let outer = window_rect(hwnd);
    let (cw, ch) = client_extent(hwnd);
    (
        (outer.right - outer.left) - cw,
        (outer.bottom - outer.top) - ch,
    )
}

/// Send a real `WM_DPICHANGED` whose suggested rectangle realises
/// `(client_w, client_h)` physical client pixels at the window's current
/// position.
unsafe fn send_dpi_change_to_client(hwnd: HWND, dpi: u32, client_w: i32, client_h: i32) {
    let (frame_w, frame_h) = frame_thickness(hwnd);
    let outer = window_rect(hwnd);
    let suggested = RECT {
        left: outer.left,
        top: outer.top,
        right: outer.left + client_w + frame_w,
        bottom: outer.top + client_h + frame_h,
    };
    let wparam = WPARAM(((dpi as usize) << 16) | dpi as usize);
    let result = SendMessageW(
        hwnd,
        WM_DPICHANGED,
        wparam,
        LPARAM(&suggested as *const RECT as isize),
    );
    assert_eq!(
        result.0, 0,
        "DD-M4-P1-003 step 5: the handler returns LRESULT(0) whatever happened"
    );
}

/// Put a freshly created window into the before-state every fixture below
/// assumes: cached scale [`REFERENCE_DPI`], physical (and therefore DIP)
/// client `(client_w, client_h)`. Both halves are asserted rather than
/// assumed — see `hit_resolution_integration.rs`'s module header for why.
unsafe fn normalise_to_reference_baseline(
    window: *mut ffi::WasamoWindow,
    client_w: i32,
    client_h: i32,
    what: &str,
) {
    let hwnd = (*window).hwnd;
    send_dpi_change_to_client(hwnd, REFERENCE_DPI, client_w, client_h);
    let limits = display_limits();
    assert_eq!(
        client_extent(hwnd),
        (client_w, client_h),
        "{what}: the realised client extent must be the requested one, or every \
         assertion below is about a rectangle nobody chose; \
         requested_client=({client_w},{client_h}), \
         display=(screen {}x{}, max_track {}x{})",
        limits.0,
        limits.1,
        limits.2,
        limits.3,
    );
    assert_eq!(
        ffi::__window_scale_dpi_for_test(window),
        REFERENCE_DPI,
        "{what}: the baseline scale must be the reference one, or the factors \
         below are taken against the developer's monitor"
    );
}

/// A real `WM_LBUTTONUP` at a **physical** client position, delivered
/// synchronously via `SendMessageW`. Suitable for every click in this file
/// that only needs `hit_test_click`'s synchronous dispatch (an inline
/// handler's state write), not a queued host-listener callback.
unsafe fn send_click(hwnd: HWND, x: f32, y: f32) {
    let packed = ((y.round() as i32 as u32) << 16) | (x.round() as i32 as u32 & 0xFFFF);
    SendMessageW(
        hwnd,
        WM_LBUTTONUP,
        WPARAM(0),
        LPARAM(packed as i32 as isize),
    );
}

/// A real `WM_LBUTTONUP` at a **physical** client position, delivered
/// through the production message loop instead of by direct `SendMessageW`
/// — needed whenever a fixture connects a host listener through the real
/// ABI, because that callback only fires from `emit::drain_if_outermost`'s
/// Phase 3, reached at `wasamo_runtime::run`'s message-loop boundary, not
/// from a bare `SendMessageW`. See `event_routing_integration.rs`'s module
/// header ("Why an inline handler is observable synchronously...") for the
/// full argument. Posts the click, then a synthesised `WM_QUIT`, then pumps
/// `wasamo_runtime::run()` until it returns.
unsafe fn click_and_drain(hwnd: HWND, x: f32, y: f32) {
    let packed = ((y.round() as i32 as u32) << 16) | (x.round() as i32 as u32 & 0xFFFF);
    PostMessageW(
        hwnd,
        WM_LBUTTONUP,
        WPARAM(0),
        LPARAM(packed as i32 as isize),
    )
    .expect("PostMessageW(WM_LBUTTONUP)");
    PostQuitMessage(0);
    wasamo_runtime::run();
}

/// Read a `Text` widget's rendered content and parse it as the `i32`
/// counter it is bound to (`"\{root.<state>}"`). `__text_content_for_test`
/// reads the `content: String` field directly, which an inline handler's
/// `Signal::set` writes synchronously — no layout pass is needed for this
/// read to be current. Panics with `what` in the message if the node is
/// not a `Text` or its content does not parse.
fn read_counter(node: &WidgetNode, what: &str) -> i32 {
    let content = node
        .__text_content_for_test()
        .unwrap_or_else(|| panic!("{what}: node must be a Text widget bound to the counter"));
    content
        .parse::<i32>()
        .unwrap_or_else(|e| panic!("{what}: Text content {content:?} did not parse as i32 ({e})"))
}

// ── This file's own helpers ─────────────────────────────────────────────

/// Read a `Text` widget's raw rendered content, unparsed — the string-
/// valued twin of [`read_counter`], needed because F2's discriminator is
/// itself a string (the item a per-item handler read and appended to
/// `seen`), not an integer.
fn read_text(node: &WidgetNode, what: &str) -> String {
    node.__text_content_for_test()
        .unwrap_or_else(|| panic!("{what}: node must be a Text widget"))
        .to_owned()
}

/// Paired click / destroy counters for a single host-connected `"clicked"`
/// listener. `wasamo_signal_connect`'s `callback` and `destroy_fn` share
/// one `user_data` pointer, so a fixture that needs to count both
/// independently boxes them together (see this file's module header).
struct ClickDestroyCounters {
    clicked: std::cell::Cell<u32>,
    destroyed: std::cell::Cell<u32>,
}

unsafe extern "C" fn count_click(
    _sender: *mut ffi::WasamoWidget,
    _args: *const ffi::WasamoValue,
    _arg_count: usize,
    user_data: *mut c_void,
) {
    let counters = &*(user_data as *const ClickDestroyCounters);
    counters.clicked.set(counters.clicked.get() + 1);
}

unsafe extern "C" fn count_destroy(user_data: *mut c_void) {
    let counters = &*(user_data as *const ClickDestroyCounters);
    counters.destroyed.set(counters.destroyed.get() + 1);
}

/// Connect [`count_click`] / [`count_destroy`] to `widget`'s `"clicked"`
/// signal through the **real** ABI (`ffi::wasamo_signal_connect`), exactly
/// as `event_routing_integration.rs`'s `connect_host_click_counter` does,
/// with a `destroy_fn` supplied so F4 and F5 can observe *when* the
/// registration is severed, not only whether the callback still fires
/// afterwards.
///
/// # Safety
/// `widget` must be non-null and point to a widget live on the calling
/// (owning) thread.
unsafe fn connect_click_and_destroy_counters(
    widget: *mut ffi::WasamoWidget,
) -> (*mut ClickDestroyCounters, u64) {
    let counters = Box::into_raw(Box::new(ClickDestroyCounters {
        clicked: std::cell::Cell::new(0),
        destroyed: std::cell::Cell::new(0),
    }));
    let mut token: u64 = 0;
    let name = "clicked";
    let status = ffi::wasamo_signal_connect(
        widget,
        name.as_ptr() as *const c_char,
        name.len(),
        Some(count_click),
        counters as *mut c_void,
        Some(count_destroy),
        &mut token,
    );
    assert_eq!(
        status,
        ffi::WASAMO_OK,
        "wasamo_signal_connect must succeed for the fixture to mean anything"
    );
    (counters, token)
}

// ── F1 — the baseline claim: a per-item handler reads the clicked row's index ─

/// Three `Box` rows generated from `labels`; each writes its own `index`
/// binder read to `root.selected_index` when clicked.
const PER_ITEM_INDEX_READ_UI: &str = r#"component PerItemIndexRead inherits Window {
    state labels: string[] = ["A", "B", "C"]
    state selected_index: i32 = -1
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.selected_index}" }
        for label, i in labels {
            Box { aspect: 20:1 fill: #336699cc clicked => { root.selected_index = i; } }
        }
    }
}"#;

/// **Production path:** `.ui` -> `wasamoc check` (admits the handler inside
/// the `for` body, dsl_spec §4.15) -> `lower` / `emit` -> IR text ->
/// `ir_loader::parse_ir` + `build_widget_tree` (the loader gate; each row's
/// `WidgetNode.loop_scope` is written once by `build_node_with_loop_context`)
/// -> a real `WM_LBUTTONUP` via `send_click` -> `wnd_proc` -> `hit_test_click`
/// resolves the clicked `Box` -> `run_signal_handlers` evaluates its
/// snapshotted `loop_scope` through `ForItemHandlerEvalContext` ->
/// `evaluate`'s `IndexRead` arm -> `ctx.read_index_tracked` -> `Signal::set`
/// on `selected_index` drains synchronously -> the bound `Text` re-renders
/// -> [`read_counter`] reads it back.
///
/// Two different rows (0 and 2) are clicked, not the same row twice: a
/// fixture that clicked one row repeatedly could pass under an
/// implementation that always reports the same (wrong) index, since
/// nothing would ever demand a *different* one (implementation-gates trap
/// #3, T8 close gate W9's recorded precedent for exactly this failure
/// mode).
#[test]
fn a_per_item_handler_reads_the_index_of_the_row_that_was_clicked() {
    run_on_owning_runtime_thread_or_skip(
        "F1: per-item handler reads the clicked row's index",
        move || {
            let ir = lower_ui_to_ir(PER_ITEM_INDEX_READ_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F1 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                // root.children: [Text(selected_index), row 0, row 1, row 2].
                let (row0_rect, row2_rect) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    (
                        root.children[1].__arranged_rect_for_test(),
                        root.children[3].__arranged_rect_for_test(),
                    )
                };
                let row0_rect = row0_rect.expect("row 0 must have been laid out");
                let row2_rect = row2_rect.expect("row 2 must have been laid out");
                assert!(
                    row0_rect.width > 0.0 && row0_rect.height > 0.0,
                    "fixture stopped discriminating: row 0's rectangle {row0_rect:?} must be \
                     non-degenerate"
                );
                assert!(
                    row2_rect.width > 0.0 && row2_rect.height > 0.0,
                    "fixture stopped discriminating: row 2's rectangle {row2_rect:?} must be \
                     non-degenerate"
                );
                assert!(
                    row0_rect.y + row0_rect.height <= row2_rect.y,
                    "fixture stopped discriminating: row 0 and row 2's rectangles \
                     ({row0_rect:?}, {row2_rect:?}) must be disjoint, or a click meant for one \
                     could resolve to the other"
                );

                let (r0x, r0y) = (
                    (row0_rect.x + row0_rect.width / 2.0) * factor,
                    (row0_rect.y + row0_rect.height / 2.0) * factor,
                );
                send_click(hwnd, r0x, r0y);
                let after_row0 = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    read_counter(&root.children[0], "selected_index after clicking row 0")
                };

                // Re-read row 2's rectangle immediately before clicking it
                // (rule: never click a cached rectangle across a state
                // write that could have re-laid the tree out).
                let row2_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    root.children[3]
                        .__arranged_rect_for_test()
                        .expect("row 2 must still be laid out")
                };
                let (r2x, r2y) = (
                    (row2_rect.x + row2_rect.width / 2.0) * factor,
                    (row2_rect.y + row2_rect.height / 2.0) * factor,
                );
                send_click(hwnd, r2x, r2y);
                let after_row2 = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    read_counter(&root.children[0], "selected_index after clicking row 2")
                };

                ffi::wasamo_window_destroy(window);

                assert_eq!(after_row0, 0, "clicking row 0 must write its own index (0)");
                assert_eq!(
                    after_row2, 2,
                    "clicking row 2 must write its own index (2), not row 0's"
                );
                assert_ne!(
                    after_row0, after_row2,
                    "two different rows must produce two different reads; clicking the same row \
                     twice would not constrain per-item index resolution at all"
                );
            }
        },
    );
}

// ── F2 — the discriminating fixture: a same-length whole-value reset ───────

/// A "reset" `Box` that replaces `labels` with a **same-length** literal
/// (`["Z", "Y", "X"]`); three `Box` rows generated from `labels`, each
/// appending its own `label` binder read to `seen` when clicked; and a
/// second `for` that renders `seen` back out as `Text` children — the only
/// way this file can observe a *string* item read from a handler at all.
/// M4-Phase 2's handler surface has no `set_string` for a scalar state
/// (start-gate fact 5: "handler-position string assignment does not exist
/// for any right-hand side"), so `root.s = label` over a `string[]`
/// collection was never buildable and would only ever log at click time;
/// `seen = seen.append(label)` is buildable, because `set_string_list`
/// exists and stage 3's own gap-fix is exactly this path
/// (`evaluate_binding`'s new `ItemRead` arm, 38f4daf).
const PER_ITEM_SAME_LENGTH_RESET_UI: &str = r#"component PerItemSameLengthReset inherits Window {
    state labels: string[] = ["alpha", "beta", "gamma"]
    state seen: string[] = []
    VStack {
        spacing: 0
        padding: 0
        Box { aspect: 20:1 fill: #996633cc clicked => { labels = ["Z", "Y", "X"]; } }
        for label in labels {
            Box { aspect: 20:1 fill: #336699cc clicked => { seen = seen.append(label); } }
        }
        for entry in seen {
            Text { text: entry }
        }
    }
}"#;

/// **THE discriminating fixture** (DD-M4-P2-005 §Technical risk
/// re-evaluation; plan §T9 "Identity"). `labels = ["Z", "Y", "X"]` is a
/// same-length whole-value reset, which dsl_spec §4.15 "Identity baseline"
/// states makes **no structural edit**: the three row subtrees are
/// retained, not rebuilt — verified below (pointer and rectangle identity
/// before/after the reset), not assumed, because the discriminator means
/// nothing if the later click actually lands on a freshly generated row
/// rather than the original one.
///
/// **Why this is the discriminator.** A per-item handler that had captured
/// its `label` binder read at *generation* time would still append
/// `"alpha"` / `"gamma"` after the reset, because nothing rebuilt its
/// closure. This runtime resolves `label` at *invocation* time
/// (dsl_spec §4.19), against whatever `labels` currently holds — so it
/// appends `"Z"` / `"X"` instead, the values the reset just wrote to the
/// same positions. **A wrong (generation-time-capture) implementation
/// would make [`seen_after_row0`] read `"alpha"` and [`seen1_final`] read
/// `"gamma"`** — the two assertions this fixture ends on. A test that only
/// ever clicked a freshly generated row could not tell the two apart, per
/// DD-M4-P2-005's own naming of this gap.
///
/// **Production path:** the reset control's click drives
/// `evaluate_collection_assignment`'s `ListLit` arm (`ctx.set_string_list`
/// on `labels`) through the same synchronous `Signal::set` drain every
/// other fixture's writes use; the row clicks drive `ListAppend`'s `Str`
/// arm, which routes the binder read through `evaluate_binding`'s
/// `ItemRead` arm (`ctx.read_item_binding_tracked`) — the exact gap 38f4daf
/// closed for the string element type.
#[test]
fn a_per_item_handler_reads_the_current_item_after_a_same_length_collection_reset() {
    run_on_owning_runtime_thread_or_skip(
        "F2: per-item handler reads the item after a same-length reset",
        move || {
            let ir = lower_ui_to_ir(PER_ITEM_SAME_LENGTH_RESET_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F2 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                // root.children: [reset control, row 0 ("alpha"),
                // row 1 ("beta"), row 2 ("gamma")].
                let (row0_ptr_before, row2_ptr_before, row0_rect, row2_rect) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        4,
                        "fixture stopped discriminating: the reset control and three rows must \
                         exist before the reset"
                    );
                    (
                        root.children[1].as_ref() as *const WidgetNode,
                        root.children[3].as_ref() as *const WidgetNode,
                        root.children[1].__arranged_rect_for_test(),
                        root.children[3].__arranged_rect_for_test(),
                    )
                };
                let row0_rect = row0_rect.expect("row 0 must have been laid out");
                let row2_rect = row2_rect.expect("row 2 must have been laid out");

                let reset_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    root.children[0]
                        .__arranged_rect_for_test()
                        .expect("reset control must be laid out")
                };
                let (resx, resy) = (
                    (reset_rect.x + reset_rect.width / 2.0) * factor,
                    (reset_rect.y + reset_rect.height / 2.0) * factor,
                );
                send_click(hwnd, resx, resy);

                // Verify the reset really made no structural edit before
                // trusting the discriminator that depends on it.
                let (row0_ptr_after, row2_ptr_after, row0_rect_after, row2_rect_after) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        4,
                        "a same-length reset must not add or remove children"
                    );
                    (
                        root.children[1].as_ref() as *const WidgetNode,
                        root.children[3].as_ref() as *const WidgetNode,
                        root.children[1]
                            .__arranged_rect_for_test()
                            .expect("row 0 must still be laid out"),
                        root.children[3]
                            .__arranged_rect_for_test()
                            .expect("row 2 must still be laid out"),
                    )
                };
                assert_eq!(
                    row0_ptr_before, row0_ptr_after,
                    "row 0's subtree must be retained, not rebuilt, by a same-length reset — \
                     dsl_spec §4.15 'Identity baseline'"
                );
                assert_eq!(
                    row2_ptr_before, row2_ptr_after,
                    "row 2's subtree must be retained, not rebuilt, by a same-length reset"
                );
                assert_eq!(
                    row0_rect, row0_rect_after,
                    "row 0's rectangle must not move"
                );
                assert_eq!(
                    row2_rect, row2_rect_after,
                    "row 2's rectangle must not move"
                );

                // Discriminator, row 0: retained, formerly "alpha", now "Z".
                let (r0x, r0y) = (
                    (row0_rect_after.x + row0_rect_after.width / 2.0) * factor,
                    (row0_rect_after.y + row0_rect_after.height / 2.0) * factor,
                );
                send_click(hwnd, r0x, r0y);
                let seen_after_row0 = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        5,
                        "appending to `seen` must materialise exactly one new Text child"
                    );
                    read_text(&root.children[4], "seen[0] after clicking retained row 0")
                };

                // Discriminator, row 2: retained, formerly "gamma", now "X".
                // Re-read row 2's rectangle immediately before clicking it.
                let row2_rect_now = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    root.children[3]
                        .__arranged_rect_for_test()
                        .expect("row 2 must still be laid out")
                };
                let (r2x, r2y) = (
                    (row2_rect_now.x + row2_rect_now.width / 2.0) * factor,
                    (row2_rect_now.y + row2_rect_now.height / 2.0) * factor,
                );
                send_click(hwnd, r2x, r2y);
                let (seen0_final, seen1_final) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        6,
                        "a second append must materialise a second Text child"
                    );
                    (
                        read_text(&root.children[4], "seen[0] after both clicks"),
                        read_text(&root.children[5], "seen[1] after both clicks"),
                    )
                };

                ffi::wasamo_window_destroy(window);

                assert_eq!(
                    seen_after_row0, "Z",
                    "row 0's handler must read the item the reset just wrote (\"Z\"), not the \
                     item present when its subtree was generated (\"alpha\") — a \
                     generation-time capture would have appended \"alpha\" here"
                );
                assert_eq!(
                    seen0_final, "Z",
                    "row 0's earlier append must not have been disturbed by row 2's click"
                );
                assert_eq!(
                    seen1_final, "X",
                    "row 2's handler must read the item the reset just wrote (\"X\"), not the \
                     item present when its subtree was generated (\"gamma\")"
                );
            }
        },
    );
}

// ── F3 — a surviving row keeps its position after a tail removal ───────────

/// A "remove" `Box` that shortens `values` by one (`drop-last`); three
/// `Box` rows generated from `values`, each writing its own `v` / `idx`
/// binder reads to `root.last_value` / `root.last_index` when clicked.
const PER_ITEM_SURVIVOR_TAIL_REMOVAL_UI: &str = r#"component PerItemSurvivorTailRemoval inherits Window {
    state values: i32[] = [10, 20, 30]
    state last_value: i32 = -1
    state last_index: i32 = -1
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.last_value}" }
        Text { text: "\{root.last_index}" }
        Box { aspect: 20:1 fill: #996633cc clicked => { values = values.drop-last(); } }
        for v, idx in values {
            Box { aspect: 20:1 fill: #336699cc clicked => { root.last_value = v; root.last_index = idx; } }
        }
    }
}"#;

/// Start-gate fact 4 (log.md): "a surviving row's position never moves,
/// and that is a property of the reconciler rather than of the fixtures" —
/// `mutate_for_loop_subtree`'s tail-only reconciliation neither rebuilds
/// nor re-indexes a retained row. Row 1 (item `20`, index `1`) is the row
/// immediately *before* the removed tail (row 2) — the position most
/// exposed to an off-by-one in the retained-vs-removed split, and
/// therefore the sharper boundary than row 0 would be.
///
/// **Production path:** the remove control's click drives
/// `evaluate_collection_assignment`'s `ListDropLast` arm on `values`,
/// which reaches `mutate_for_loop_subtree`'s `Remove { tail_first_indices
/// }` arm (`plan_tail_range_change`) — the reconciler under test. Row 1's
/// subsequent click drives the same `ForItemHandlerEvalContext` /
/// `evaluate` path F1 exercises, but against a `ForItemContext.position`
/// that has survived a structural mutation rather than one from a freshly
/// built tree.
#[test]
fn a_per_item_handler_on_a_surviving_row_keeps_its_position_after_a_tail_removal() {
    run_on_owning_runtime_thread_or_skip(
        "F3: surviving row keeps its position after a tail removal",
        move || {
            let ir = lower_ui_to_ir(PER_ITEM_SURVIVOR_TAIL_REMOVAL_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F3 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                // root.children: [Text(last_value), Text(last_index),
                // remove control, row 0 (10), row 1 (20), row 2 (30)].
                let (row1_ptr_before, row1_rect) = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        6,
                        "fixture stopped discriminating: three rows must exist before removal"
                    );
                    (
                        root.children[4].as_ref() as *const WidgetNode,
                        root.children[4].__arranged_rect_for_test(),
                    )
                };
                let row1_rect = row1_rect.expect("row 1 must have been laid out");
                assert!(
                    row1_rect.width > 0.0 && row1_rect.height > 0.0,
                    "fixture stopped discriminating: row 1's rectangle {row1_rect:?} must be \
                     non-degenerate"
                );

                let remove_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    root.children[2]
                        .__arranged_rect_for_test()
                        .expect("remove control must be laid out")
                };
                let (rmx, rmy) = (
                    (remove_rect.x + remove_rect.width / 2.0) * factor,
                    (remove_rect.y + remove_rect.height / 2.0) * factor,
                );
                send_click(hwnd, rmx, rmy);

                let (row1_ptr_after, row1_rect_after) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        5,
                        "the tail removal must remove exactly one row"
                    );
                    (
                        root.children[4].as_ref() as *const WidgetNode,
                        root.children[4]
                            .__arranged_rect_for_test()
                            .expect("row 1 must still be laid out"),
                    )
                };
                assert_eq!(
                    row1_ptr_before, row1_ptr_after,
                    "the surviving row must be the same subtree, not a rebuilt one"
                );
                assert_eq!(
                    row1_rect, row1_rect_after,
                    "the surviving row's rectangle must not move"
                );

                let (cx, cy) = (
                    (row1_rect_after.x + row1_rect_after.width / 2.0) * factor,
                    (row1_rect_after.y + row1_rect_after.height / 2.0) * factor,
                );
                send_click(hwnd, cx, cy);

                let (last_value, last_index) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    (
                        read_counter(
                            &root.children[0],
                            "last_value after clicking the surviving row",
                        ),
                        read_counter(
                            &root.children[1],
                            "last_index after clicking the surviving row",
                        ),
                    )
                };

                ffi::wasamo_window_destroy(window);

                assert_eq!(
                    last_value, 20,
                    "the surviving row must still read its own item (20), not a shifted one"
                );
                assert_eq!(
                    last_index, 1,
                    "the surviving row must still read its own unchanged position (1) after the \
                     tail removal behind it"
                );
            }
        },
    );
}

// ── F4 — a removed row's handler registration is released with its subtree ─

/// A "remove" `Box` that shortens `labels` by one; three `Box` rows
/// generated from `labels`, none carrying an inline `clicked` handler —
/// only a host listener, connected before the removal.
const PER_ITEM_HANDLER_RELEASED_WITH_SUBTREE_UI: &str = r#"component PerItemHandlerReleasedWithSubtree inherits Window {
    state labels: string[] = ["A", "B", "C"]
    VStack {
        spacing: 0
        padding: 0
        Box { aspect: 20:1 fill: #996633cc clicked => { labels = labels.drop-last(); } }
        for label in labels {
            Box { aspect: 20:1 fill: #336699cc }
        }
    }
}"#;

/// The registration-lifecycle evidence (DD-M4-P2-005 §Technical risk
/// re-evaluation): "the registration lifecycle fails silently in one
/// direction. A handler left registered against a dropped subtree is not
/// visible in any rendered frame." A host listener connected through the
/// real ABI is shown to fire while its row is alive (agreement leg,
/// without which "the listener never fires" after removal could equally
/// be explained by a listener that was never wired at all), then removed
/// with its row, then shown never to fire again at the same coordinates
/// (difference leg) — with its `destroy_fn` firing exactly once,
/// synchronously with the removal, as the direct evidence that the
/// registration was actually severed rather than merely orphaned.
///
/// **Production path:** `connect_click_and_destroy_counters` reaches
/// `crate::registry::add_signal` through the real
/// `ffi::wasamo_signal_connect` entry point (abi_spec). The remove
/// control's click reaches `evaluate_collection_assignment`'s
/// `ListDropLast` arm on `labels`, `mutate_for_loop_subtree`'s tail-removal
/// branch, and `widget_destroy` on the removed row — which calls
/// `registry::remove_for_widget`, the disposal path this fixture's
/// `destroy_fn` observes. The row's dangling pointer is never dereferenced
/// again after removal: `run_signal_handlers`'s own safety argument
/// (widget.rs) is that `enqueue_signal` only compares `widget_ptr`, and the
/// difference-leg click below relies on exactly that.
#[test]
fn a_removed_rows_handler_is_released_with_its_subtree() {
    run_on_owning_runtime_thread_or_skip(
        "F4: a removed row's host listener is released with its subtree",
        move || {
            let ir = lower_ui_to_ir(PER_ITEM_HANDLER_RELEASED_WITH_SUBTREE_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F4 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                // root.children: [remove control, row 0 (A), row 1 (B),
                // row 2 (C)].
                let (row2_ptr, row2_rect) = {
                    let root = (*window).root_widget.as_mut().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        4,
                        "fixture stopped discriminating: three rows must exist before removal"
                    );
                    let row2: &mut WidgetNode = root.children[3].as_mut();
                    let ptr = row2 as *mut WidgetNode;
                    let rect = row2.__arranged_rect_for_test();
                    (ptr, rect)
                };
                let row2_rect = row2_rect.expect("row 2 must have been laid out");
                assert!(
                    row2_rect.width > 0.0 && row2_rect.height > 0.0,
                    "fixture stopped discriminating: row 2's rectangle {row2_rect:?} must be \
                     non-degenerate"
                );

                let (counters, _token) =
                    connect_click_and_destroy_counters(row2_ptr as *mut ffi::WasamoWidget);

                // Agreement leg: the listener fires while row 2 is alive,
                // at the coordinates the difference leg reuses below.
                let (cx, cy) = (
                    (row2_rect.x + row2_rect.width / 2.0) * factor,
                    (row2_rect.y + row2_rect.height / 2.0) * factor,
                );
                click_and_drain(hwnd, cx, cy);
                assert_eq!(
                    (*counters).clicked.get(),
                    1,
                    "the host listener must fire once while row 2 is still live"
                );
                assert_eq!(
                    (*counters).destroyed.get(),
                    0,
                    "row 2 must not have been destroyed yet"
                );

                // Removal: shorten the collection by one, disposing row 2's
                // subtree and, per DD-M4-P2-005's registration-lifecycle
                // rule, whatever registry entries ride along with it.
                let remove_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    root.children[0]
                        .__arranged_rect_for_test()
                        .expect("remove control must be laid out")
                };
                let (rmx, rmy) = (
                    (remove_rect.x + remove_rect.width / 2.0) * factor,
                    (remove_rect.y + remove_rect.height / 2.0) * factor,
                );
                click_and_drain(hwnd, rmx, rmy);
                let children_after_removal = (*window).root_widget.as_ref().unwrap().children.len();
                let destroyed_after_removal = (*counters).destroyed.get();

                // Difference leg: the same coordinates row 2 used to
                // occupy, now unoccupied.
                click_and_drain(hwnd, cx, cy);

                let clicked_final = (*counters).clicked.get();
                let destroyed_final = (*counters).destroyed.get();

                ffi::wasamo_window_destroy(window);
                drop(Box::from_raw(counters));

                assert_eq!(
                    children_after_removal, 3,
                    "the tail removal must remove exactly row 2"
                );
                assert_eq!(
                    destroyed_after_removal, 1,
                    "row 2's registration must be released exactly once, synchronously with its \
                     subtree's removal — before this fixture ever clicks its old coordinates"
                );
                assert_eq!(
                    destroyed_final, 1,
                    "the destroy callback must not fire a second time"
                );
                assert_eq!(
                    clicked_final, 1,
                    "a click at row 2's old coordinates must not fire its listener again: the \
                     registration was severed with the subtree, so nothing is left to enqueue"
                );
            }
        },
    );
}

// ── F5 — a handler that shortens its own collection reports the stale item ─

/// Three `Box` rows generated from `xs`; each handler shortens `xs` by one,
/// *then* reads its own `value` binder — the shape start-gate fact 9 names
/// as the reachable out-of-range boundary: `{ xs = xs.drop-last();
/// root.n = value; }` clicked on the **last** row.
const PER_ITEM_HANDLER_SHORTENS_OWN_COLLECTION_UI: &str = r#"component PerItemHandlerShortensOwnCollection inherits Window {
    state xs: i32[] = [10, 20, 30]
    state n: i32 = -1
    VStack {
        spacing: 0
        padding: 0
        Text { text: "\{root.n}" }
        for value in xs {
            Box { aspect: 20:1 fill: #336699cc clicked => { xs = xs.drop-last(); root.n = value; } }
        }
    }
}"#;

/// Start-gate fact 9 (log.md): "the item-out-of-range read is reachable,
/// and buildable... because a collection write drains its reactive effects
/// **synchronously, inside the statement**, and the handler body then
/// keeps evaluating from a clone." Clicking the *last* row (`xs[2] == 30`)
/// means the position the second statement tries to read is exactly the
/// one `drop-last` just removed.
///
/// **The observable consequence, not the log line.** `evaluate`'s `Block`
/// arm short-circuits on the first `Err` (`handler.rs`), so
/// `EvalError::ItemOutOfRange` from the second statement's `ItemRead`
/// aborts the block *before* `ctx.set_i32("root.n", …)` runs — `n` must
/// therefore still hold its `-1` sentinel, never the stale `30`, and never
/// a wrapped/defaulted `0`. `invoke_handler` only logs the error to stderr
/// (`wasamo: handler error in …`) and returns `false`; this fixture asserts
/// the state that would have been written, not that line.
///
/// This fixture also carries **the DD-M4-P2-001 ordering measurement** —
/// see this file's module header. `destroyed` (via
/// [`connect_click_and_destroy_counters`]) is the evidence: it reads `1`
/// after the single `send_click`, so the clicked row's own subtree was
/// already torn down by the time the second statement's read failed.
///
/// **Production path:** the click reaches `run_signal_handlers`, whose
/// `handlers.loop_scope` snapshot (cloned by `signal_handlers_for` before
/// any statement runs, per start-gate fact 8) drives
/// `ForItemHandlerEvalContext` through `handler::evaluate`'s `Block` arm:
/// statement 1's `Assign { rhs: ListDropLast }` reaches
/// `evaluate_collection_assignment`, whose `ctx.set_i32_list("xs", …)`
/// drains `xs`'s registered `EffectHandle` synchronously (the loop
/// reconciliation effect `register_for_loop_binding` installs) — the
/// removal of the very row being dispatched through. Statement 2's
/// `ItemRead` then reaches `ctx.read_item_i32_tracked`, which reports
/// `None` because the position is past the now-shrunk collection,
/// producing `EvalError::ItemOutOfRange`.
#[test]
fn a_per_item_handler_that_shortens_its_own_collection_reports_the_stale_item_rather_than_a_wrong_one(
) {
    run_on_owning_runtime_thread_or_skip(
        "F5: a per-item handler that shortens its own collection",
        move || {
            let ir = lower_ui_to_ir(PER_ITEM_HANDLER_SHORTENS_OWN_COLLECTION_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F5 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                // root.children: [Text(n), row 0 (10), row 1 (20), row 2 (30)].
                let (row2_ptr, row2_rect) = {
                    let root = (*window).root_widget.as_mut().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        4,
                        "fixture stopped discriminating: three rows must exist before the click"
                    );
                    let row2: &mut WidgetNode = root.children[3].as_mut();
                    let ptr = row2 as *mut WidgetNode;
                    let rect = row2.__arranged_rect_for_test();
                    (ptr, rect)
                };
                let row2_rect = row2_rect.expect("row 2 must have been laid out");

                let (counters, _token) =
                    connect_click_and_destroy_counters(row2_ptr as *mut ffi::WasamoWidget);

                let n_before = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    read_counter(&root.children[0], "n before the click")
                };
                assert_eq!(
                    n_before, -1,
                    "fixture stopped discriminating: n must start at its sentinel"
                );

                let (cx, cy) = (
                    (row2_rect.x + row2_rect.width / 2.0) * factor,
                    (row2_rect.y + row2_rect.height / 2.0) * factor,
                );
                // A single synchronous SendMessageW: both of the inline
                // handler's statements (and their reactive drains) run
                // before this call returns — see this file's module header
                // on why an inline handler needs no `click_and_drain`.
                send_click(hwnd, cx, cy);

                let (n_after, children_after, destroyed_after, health) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    (
                        read_counter(&root.children[0], "n after the click"),
                        root.children.len(),
                        (*counters).destroyed.get(),
                        ffi::__runtime_health_for_test(),
                    )
                };

                ffi::wasamo_window_destroy(window);
                drop(Box::from_raw(counters));

                assert_eq!(
                    children_after, 3,
                    "the handler's own `xs.drop-last()` must have removed exactly the clicked row"
                );
                assert_eq!(
                    destroyed_after, 1,
                    "the clicked row's own subtree must already have been destroyed by the time \
                     this assertion runs — the DD-M4-P2-001 ordering measurement: regeneration \
                     ran inside the handler's own statement sequence, not after the handler \
                     returned (see this file's module header)"
                );
                assert_eq!(
                    n_after, -1,
                    "the second statement's `root.n = value` must not have run: `value` was \
                     already past the shrunk collection's end (EvalError::ItemOutOfRange, logged \
                     to stderr), so `n` must still hold its sentinel rather than the stale 30"
                );
                assert_eq!(
                    health, "Healthy",
                    "dispatching through a row that shortens its own collection mid-handler must \
                     not leave the runtime Diverged"
                );
            }
        },
    );
}

// ── F6 — F5's agreement leg: the same handler, a still-live position ───────

/// **F5's agreement leg**, and the reason it exists is a measurement, not
/// symmetry for its own sake. F5 ends on "`n` still holds its sentinel",
/// which is what an out-of-range read produces — and also what *any* other
/// evaluation error produces, including "this node carries no loop scope
/// at all". Mutation witness W-D (dropping `loop_scope` from
/// `signal_handlers_for`'s snapshot) reddened F1, F2 and F3 and left F5
/// **green**: on its own, F5 constrains that the second statement did not
/// run, not that it failed for the reason it claims.
///
/// This fixture supplies the leg where the read must *succeed*. It drives
/// the same `.ui` and the same two-statement handler, clicking **row 0**
/// instead of the last row: `xs.drop-last()` removes position 2, so
/// position 0 is still inside the shrunk collection and `root.n = value`
/// must write `10`. A runtime with no loop scope, a wrongly-keyed binder,
/// or an item read that always reports out-of-range fails here — which is
/// exactly what F5 alone could not see.
///
/// The two together are the difference/agreement pair: same source, same
/// handler, same mutation, one position past the end and one inside it.
#[test]
fn a_per_item_handler_that_shortens_its_own_collection_still_reads_a_position_that_survived() {
    run_on_owning_runtime_thread_or_skip(
        "F6: the surviving-position agreement leg of F5",
        move || {
            let ir = lower_ui_to_ir(PER_ITEM_HANDLER_SHORTENS_OWN_COLLECTION_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "F6 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;

                // root.children: [Text(n), row 0 (10), row 1 (20), row 2 (30)].
                let row0_rect = {
                    let root = (*window).root_widget.as_ref().expect("content root");
                    assert_eq!(
                        root.children.len(),
                        4,
                        "fixture stopped discriminating: three rows must exist before the click"
                    );
                    root.children[1]
                        .__arranged_rect_for_test()
                        .expect("row 0 must have been laid out")
                };

                let (cx, cy) = (
                    (row0_rect.x + row0_rect.width / 2.0) * factor,
                    (row0_rect.y + row0_rect.height / 2.0) * factor,
                );
                send_click(hwnd, cx, cy);

                let (n_after, children_after, health) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    (
                        read_counter(&root.children[0], "n after clicking row 0"),
                        root.children.len(),
                        ffi::__runtime_health_for_test(),
                    )
                };

                ffi::wasamo_window_destroy(window);

                assert_eq!(
                    children_after, 3,
                    "the handler's own `xs.drop-last()` must have removed the tail row, not the \
                     clicked one"
                );
                assert_eq!(
                    n_after, 10,
                    "row 0's position survived the shrink, so its `value` read must succeed and \
                     write the item now at position 0 — the leg F5 cannot supply"
                );
                assert_eq!(health, "Healthy");
            }
        },
    );
}

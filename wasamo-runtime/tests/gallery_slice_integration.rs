//! Mock-free Windows integration evidence for M4-Phase 2 T10 stage 2: the
//! **shipped** `examples/gallery/gallery.ui`, brought in with
//! `include_str!` and driven through the real pipeline —
//! `wasamoc` (`lexer` -> `parser` -> `check` -> `lower` -> `emit`) ->
//! `ffi::wasamo_load_ui(WASAMO_LOAD_MEMORY, ...)` -> real `WM_*` messages
//! through the real window procedure. Nothing is mocked, and nothing here
//! constructs a gallery-shaped miniature: every fixture tests the artifact
//! that ships. Stage 1 (`f1a6555`) landed the `.ui` wiring this file
//! exercises: `state selected_index`, the per-item `clicked` on each
//! thumbnail `Box`, the lightbox's `modal-scope: true` / `dismiss` /
//! `key-down("ArrowLeft")` / `key-down("ArrowRight")`, `clicked` on the
//! `<` / `>` / `x` Buttons, and the caption `Text` bound to
//! `"Photo #\{root.selected_index}"`.
//!
//! # What each fixture is for
//!
//! - [`g1_thumbnail_click_carries_which_thumbnail_through_the_ancestor_walk`]
//!   (G1) — a thumbnail click carries **which** thumbnail, on the shipped
//!   tree. Two different rows (thumbnail 0 and thumbnail 2) must open the
//!   lightbox on two different captions — the state-level twin of T12's
//!   control A, and clicking one row twice (the agreement leg) would not
//!   constrain anything by itself (implementation-gates trap #3). This
//!   fixture also evidences T10's start-gate finding that "the thumbnail's
//!   handler sits on a `Box` whose `Text` child is what a centre click
//!   resolves to" — i.e. that the click reaches the `Box`'s handler
//!   through T3's **ancestor walk**, not direct target dispatch. **A
//!   direct measurement**, through `WidgetNode::__resolve_topmost_for_test`
//!   (added for this task; see "The resolved-target accessor" below): the
//!   production resolver is called on the click point itself, and the
//!   returned path is asserted to equal the `Text`'s own path, its kind
//!   asserted `Text`, and the `Box`'s own path asserted to be that path's
//!   parent — measured, not deduced from geometry.
//! - [`g2_opening_the_lightbox_enters_the_scope`] (G2) — opening the
//!   lightbox enters the scope on the shipped tree: before, nothing is
//!   focused; after the thumbnail click, focus is on the `<` Button (T7's
//!   own frame pair already measured this landing on a throwaway probe
//!   build of this exact tree, log.md fact 10 — this fixture lands the
//!   annotation for real and re-measures the same claim).
//! - [`g3_escape_closes_through_dismiss_and_focus_restores`] (G3) — two
//!   legs. (a) nothing focused beforehand: Escape closes the lightbox
//!   through the authored `dismiss` handler and focus is `None` again.
//!   (b) something focused beforehand (Tab once, landing on the toolbar's
//!   first stop — read back by label rather than assumed): Escape closes
//!   the lightbox and focus **restores** to that same Button. Leg (b) is
//!   what makes "restores" mean something; leg (a) alone would be
//!   satisfied by an implementation that just clears focus on close.
//! - [`g4_arrow_keys_reach_the_scope_key_down_handlers_only_while_present`]
//!   (G4) — Left/Right reach the scope container's `key-down` handlers.
//!   Part 1: with the lightbox open at `Photo #2`, `ArrowRight` steps to
//!   `#3` and `ArrowLeft` twice steps back to `#1`. Part 2, the
//!   agreement/tripwire leg: with the host key slot recorder installed,
//!   `ArrowRight` must **not** reach it while the lightbox is open (the
//!   authored handler consumes it) and **must** reach it once the
//!   lightbox is closed. Without that pair, "the handler ran" would not
//!   distinguish the authored walk from any other explanation for the
//!   caption changing.
//! - [`g5_a_background_click_is_blocked_while_the_lightbox_is_open`] (G5)
//!   — a background click is live while the lightbox is closed and
//!   blocked while it is open, at the **identical** coordinate. **The
//!   brief named the toolbar's "Albums" `ToggleButton` for this; this
//!   fixture uses "Scroll down" instead, and the module header below
//!   ("A shipped layout defect") and the fixture's own doc comment record
//!   the measured reason: no `ToggleButton` in the toolbar is reachable
//!   by any click at this client width at all.** The fixture also carries
//!   fact 6 ("which node blocks the click: the scrim or the lightbox's
//!   own `Grid`"), answered by a **direct measurement** through
//!   `WidgetNode::__resolve_topmost_for_test` (see "The resolved-target
//!   accessor" below): the background point resolves to path `[1, 1]`,
//!   kind `Grid` — the lightbox's own `Grid`, reached through its own
//!   step-3 self-match rather than any of its Cell contents.
//! - [`g6_scrolled_hit_testing`] (G6) — the clip bound's first production
//!   consumer. Clicking "Scroll down" moves `scroll_y`; (a) the geometric
//!   premise is measured, not assumed: thumbnail 0's rectangle moves up
//!   and now overlaps the toolbar band, and some other thumbnail (found
//!   by searching the measured rectangles, not computed by arithmetic)
//!   newly sits inside the viewport. (b) clicking that now-visible
//!   thumbnail opens the lightbox at its own index. (c) a point inside
//!   **both** a scrolled-away thumbnail's rectangle and a toolbar
//!   Button's rectangle is found by the same measured search and resolves
//!   to the toolbar. **This leg uses "Open lightbox" rather than a
//!   `ToggleButton`** for the same reachability reason G5 records, with
//!   the caption's index (not a `checked` flip) as the read-back — see
//!   the fixture's own doc comment on leg (c).
//! - [`g7_the_overflowing_toolbar_swallows_clicks_aimed_at_the_tab_buttons`]
//!   (G7) — turns the toolbar-overlap finding G5 / G6(c) work around into
//!   its own measured artifact: for each of `All` / `Albums` / `Favorites`,
//!   `__resolve_topmost_for_test` at the button's own rectangle centre,
//!   printed and asserted. A click at `Albums`' / `Favorites`' own centre
//!   is asserted **not** to resolve to that button (a tripwire for a
//!   known, owner-dispositioned, client-width-driven layout observation —
//!   `constraints.md` §6 — not a claim this phase's own work could break),
//!   and a `send_click` there is asserted to leave every tab button's
//!   `checked` state unchanged — the silently-never-fires shape this
//!   phase exists to catch.
//!
//! # Standing conventions
//!
//! - Every fixture normalises to [`REFERENCE_DPI`] at the
//!   [`CLIENT_W`]x[`CLIENT_H`] physical client every other integration
//!   test file in this crate uses, and **asserts** the realised baseline
//!   (`normalise_to_reference_baseline`) rather than assuming it landed.
//! - Every fixture establishes its own before-state — nothing focused,
//!   the lightbox branch absent — and reads it back
//!   ([`assert_gallery_before_state`]) before injecting any input.
//! - **No fixture derives a coordinate inside the lightbox.** T10's start
//!   gate fact 8: the lightbox `Grid`'s fixed tracks (columns
//!   `56+400+56=512`, rows `44+300+64=408`) exceed the 360x240 client, so
//!   its layout is degenerate there. Everything this file asserts about
//!   the scope's *interior* goes through focus-path read-back
//!   (`ffi::__focus_path_for_test` + [`label_of`]), `WM_KEYDOWN`, and the
//!   caption `Text`'s rendered content
//!   (`WidgetNode::__text_content_for_test`) — never a click point
//!   derived from a lightbox-interior `__arranged_rect_for_test()`.
//! - **Every click coordinate is derived from `__arranged_rect_for_test()`**
//!   (never hand-worked-out), and the geometric relationship a fixture
//!   depends on (non-degenerate, disjoint, contained, overlapping) is
//!   asserted *before* the derived point is used.
//! - **Opening the lightbox always uses [`click_and_drain`], never a bare
//!   [`send_click`].** Measured, not a style preference: `insert_structural_child`
//!   marks the parent window's layout dirty
//!   (`crate::emit::mark_layout_dirty_for`), and the freshly materialised
//!   lightbox subtree's nodes hold **no** `arranged_rect` — and are
//!   therefore not hit-testable candidates at all (T1: "a subtree
//!   attached but never laid out... is not hit-testable") — until a
//!   layout pass runs, which only happens at `emit::flush_layout` (Phase
//!   2), reached only through the message-loop boundary `click_and_drain`
//!   pumps. The same Phase-2 dependency holds for scope *entry*
//!   (`focus::sync_scopes_to_tree`, also called from `flush_layout`):
//!   `FocusState::enter_modal` is what pushes onto `modal_stack`, and
//!   `focus_core::FocusTree::esc_target` / `traversal_root` both read
//!   `modal_stack` directly — so **`Escape` cannot close a lightbox that
//!   was opened with a bare `send_click`**, and neither can the
//!   `key-down` walk reach a handler declared on the scope container. A
//!   bare [`send_click`] is used only for a click that (a) writes no
//!   structural change and (b) is not later expected to be closable by
//!   `Escape` — G5's difference-leg click, where nothing is expected to
//!   happen at all.
//!
//! # Helpers copied verbatim
//!
//! Cargo compiles every `tests/*.rs` file as its own crate, so nothing in
//! any sibling file is importable here (the convention every file in this
//! crate states). `lower_ui_to_ir`, `load_window`, `client_extent`,
//! `display_limits`, `window_rect`, `frame_thickness`,
//! `send_dpi_change_to_client`, `normalise_to_reference_baseline`,
//! `send_click`, `click_and_drain`, `last_error`, `label_of`, and
//! `node_at_path` are copied verbatim from `modal_scope_integration.rs`
//! (itself copied from `focus_identity_integration.rs` /
//! `focus_traversal_integration.rs` / `event_routing_integration.rs`).
//! `send_key`, `send_tab`, `key_and_drain`, and `install_key_slot_recorder`
//! are copied verbatim from `modal_scope_integration.rs` (itself copied
//! from `focus_traversal_integration.rs` for the first two).
//!
//! This file's own additions: [`find_path`] — a depth-first search over
//! the live tree by predicate, returning the path of child indices to the
//! first match — so no fixture hard-codes a long child-index path into
//! the shipped tree; a `.ui` edit reddens an assertion with a readable
//! "must exist" message instead of silently indexing a different node.
//! [`find_text_path`] / [`find_button_path`] specialise it for the two
//! read-back surfaces this file needs (a `Text`'s rendered content, a
//! Button/ToggleButton's label). [`read_text`], [`rect_center_physical`],
//! [`dip_rect_contains_point`], [`dip_rect_contains`], and
//! [`dip_rect_intersect`] are the small pure geometry/read-back helpers
//! the fixtures share. [`assert_gallery_before_state`] and
//! [`assert_focus_is_labelled`] are this file's shared assertion
//! sequences (established before-state; "focus is on the node labelled
//! X", discovered rather than hard-coded).
//!
//! **A shipped layout defect, unrelated to this task's own changes,
//! surfaced while building G5 / G6(c) and is measured directly by
//! [`g7_the_overflowing_toolbar_swallows_clicks_aimed_at_the_tab_buttons`]
//! (G7):** at the 360x240 client, no point in `Albums`' or `Favorites`'
//! rectangle is reachable by a click at all — the toolbar-right `HStack`
//! overflows its own `Grid` Cell column and swallows the click first.
//! G5 and G6(c) use a toolbar-right Button instead of a `ToggleButton`
//! for this reason; G7 is where the measured rectangles, resolutions,
//! and the resulting "authored handler silently never fires" behaviour
//! live — see its own doc comment for the full argument, the numbers,
//! and why a red G7 later means the overflow was fixed, not that
//! something broke.
//!
//! **`ffi::__hover_target_for_test` cannot answer "which node did this
//! click resolve to" for a non-Button target.** Measured while building
//! G1: it reads `WindowState::hover`, which `widget.rs`'s `update_hover`
//! narrows to an *enabled Button-family* resolved target only (confirmed
//! independently by `hover_transition_integration.rs`'s own regression
//! fixture, "nothing is retained for a non-Button resolved target"), so it
//! reads `None` for every non-Button node this file's fixtures click (a
//! thumbnail's `Text`, the lightbox's scrim `Box` / `Grid`).
//!
//! # The resolved-target accessor
//!
//! This finding was reported to the lead rather than routed around, and
//! the lead re-decided trap #1's "no runtime code" prediction (the start
//! gate's own named condition for doing so): `WidgetNode::__resolve_topmost_for_test`
//! is a **new** production accessor, the one exception to this file's
//! "no runtime, compiler or IR code" boundary, added in the same commit as
//! this file's own edits. It forwards to `hit::resolve_topmost` — the
//! same function `hit_test_click` and `update_hover` call — so it is a
//! second **caller**, not a second predicate implementation; a test using
//! it cannot drift from production hit-testing behaviour. Its coordinates
//! are DIP, the same space `__arranged_rect_for_test()`'s rectangle is in,
//! never physical pixels. G1 and G5(fact 6) use it to turn what would
//! otherwise be a geometric deduction into a direct measurement; G7 uses
//! it to turn the toolbar-overlap finding into its own measured artifact
//! — see each fixture's own doc comment for what it measured.
//!
//! # The client stays small (M4-Phase 1 T8 finding)
//!
//! Every fixture normalises to 96 DPI at an explicitly chosen 360x240
//! physical client — the same ceiling every other integration test file
//! in this crate records (a hosted CI desktop failed a larger request
//! twice). The shipped gallery's own toolbar row (`rows: 56 1* 28`)
//! fits this client with an 88 DIP `item-cross-size` thumbnail grid
//! wrapping into it (T10 start-gate fact 7), which is what makes driving
//! the fixtures against the shipped `.ui` itself possible without a
//! gallery-shaped miniature.

#![cfg(windows)]

mod common;
use common::run_on_owning_runtime_thread_or_skip;

use std::cell::RefCell;
use std::ffi::{c_void, CStr};
use std::ptr;
use std::rc::Rc;

use wasamo_runtime::{ffi, DipRect, WidgetNode};

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_LEFT, VK_RIGHT, VK_TAB};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowRect, PostMessageW, PostQuitMessage, SendMessageW,
    SM_CXMAXTRACK, SM_CXSCREEN, SM_CYMAXTRACK, SM_CYSCREEN, WM_DPICHANGED, WM_KEYDOWN,
    WM_LBUTTONUP,
};

/// The shipped gallery, brought in verbatim — this file's whole point is
/// to test the artifact that ships, not a gallery-shaped miniature.
const GALLERY_UI: &str = include_str!("../../examples/gallery/gallery.ui");

/// The reference DPI every fixture normalises to before measuring. See
/// `hit_resolution_integration.rs`'s module header (Phase 1 T9 finding
/// F-47) for why this is reached deliberately rather than assumed.
const REFERENCE_DPI: u32 = 96;

/// The physical (== DIP, since no fixture here changes scale) client every
/// fixture normalises to. See this file's module header, "The client stays
/// small".
const CLIENT_W: i32 = 360;
const CLIENT_H: i32 = 240;

// ── Copied verbatim from `modal_scope_integration.rs` ───────────────────────

fn lower_ui_to_ir(src: &str) -> String {
    use wasamoc::{check, emit, lexer, lower, parser};

    let path = "<gallery-slice-integration>";
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
/// synchronously via `SendMessageW`. Suitable for a click whose handler
/// writes no structural change and that is not later closed by `Escape`
/// (see this file's module header, "Opening the lightbox always uses
/// `click_and_drain`").
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
/// through the production message loop instead of by direct `SendMessageW`.
/// Posts the click, then a synthesised `WM_QUIT`, then pumps
/// `wasamo_runtime::run()` until it returns — reaching `emit::flush_layout`
/// (Phase 2), where a freshly materialised subtree gets its first layout
/// pass and modal-scope entry runs. Required for every click that opens
/// the lightbox in this file (see this file's module header).
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

/// A real `WM_KEYDOWN` for an arbitrary virtual-key code, with no modifier
/// state manipulated.
unsafe fn send_key(hwnd: HWND, vk: u16) -> windows::Win32::Foundation::LRESULT {
    SendMessageW(hwnd, WM_KEYDOWN, WPARAM(vk as usize), LPARAM(0))
}

/// A real `WM_KEYDOWN(VK_TAB)` with no Shift held.
unsafe fn send_tab(hwnd: HWND) -> windows::Win32::Foundation::LRESULT {
    send_key(hwnd, VK_TAB.0)
}

/// A `WM_KEYDOWN` analogue of [`click_and_drain`]: posts the key, then a
/// synthesised `WM_QUIT`, then pumps `wasamo_runtime::run()` — needed
/// whenever the key's handler (e.g. `dismiss`) mutates state structurally
/// and the fixture needs Phase 2's exit-restoration to have run.
unsafe fn key_and_drain(hwnd: HWND, vk: u16) {
    PostMessageW(hwnd, WM_KEYDOWN, WPARAM(vk as usize), LPARAM(0))
        .expect("PostMessageW(WM_KEYDOWN)");
    PostQuitMessage(0);
    wasamo_runtime::run();
}

/// Read the last-error message through the C ABI.
unsafe fn last_error() -> Option<String> {
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

/// Read a Button-family node's label through the C ABI
/// (`wasamo_get_property`, `PROP_BUTTON_LABEL = 1`). Reading the label back
/// makes "it landed on the widget I meant" a claim about a widget rather
/// than about an index or a path.
fn label_of(node: &WidgetNode) -> String {
    const PROP_BUTTON_LABEL: u32 = 1;
    let widget = node as *const WidgetNode as *mut ffi::WasamoWidget;
    let mut value = ffi::WasamoValue {
        tag: ffi::WASAMO_VALUE_NONE,
        payload: ffi::WasamoValuePayload { v_i32: 0 },
    };
    let status = unsafe { ffi::wasamo_get_property(widget, PROP_BUTTON_LABEL, &mut value) };
    assert_eq!(
        status,
        ffi::WASAMO_OK,
        "wasamo_get_property(label) failed: {:?}",
        unsafe { last_error() }
    );
    assert_eq!(value.tag, ffi::WASAMO_VALUE_STRING);
    let view = unsafe { value.payload.v_string };
    let bytes = unsafe { std::slice::from_raw_parts(view.ptr as *const u8, view.len) };
    std::str::from_utf8(bytes)
        .expect("label must be UTF-8")
        .to_owned()
}

/// Descend `root` by `path` (child indices, root-relative — the same shape
/// `ffi::__focus_path_for_test` / `ffi::__hover_target_for_test` return)
/// and return the node at the end.
fn node_at_path<'a>(root: &'a WidgetNode, path: &[usize]) -> &'a WidgetNode {
    let mut node: &WidgetNode = root;
    for &i in path {
        node = node.children[i].as_ref();
    }
    node
}

/// Install a recorder in the window's **host key slot** and hand back the
/// list it appends to. `window.rs`'s `WM_KEYDOWN` arm calls `key_down_fn`
/// only after traversal, arrow routing, dismissal, and the authored
/// `key-down` walk have all declined the key, so what this list contains
/// after a message is the direct answer to "did the runtime's key
/// machinery consume it" — installing a recorder in a pre-existing `pub`
/// field is not shipping an installer (`focus_traversal_integration.rs`'s
/// module header states this in full).
unsafe fn install_key_slot_recorder(window: *mut ffi::WasamoWindow) -> Rc<RefCell<Vec<u16>>> {
    let seen: Rc<RefCell<Vec<u16>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&seen);
    (*window).key_down_fn = Some(Box::new(move |vk| sink.borrow_mut().push(vk)));
    seen
}

// ── This file's own helpers ─────────────────────────────────────────────

/// Depth-first search over the live tree (including `root` itself) for the
/// first node matching `predicate`, returning the path of child indices to
/// it. No fixture in this file hard-codes a long child-index path into the
/// shipped tree because of this — a `.ui` edit reddens an assertion with a
/// readable "must exist" message instead of silently indexing a different
/// node (this file's module header).
fn find_path(root: &WidgetNode, predicate: &dyn Fn(&WidgetNode) -> bool) -> Option<Vec<usize>> {
    fn walk(
        node: &WidgetNode,
        path: &mut Vec<usize>,
        predicate: &dyn Fn(&WidgetNode) -> bool,
    ) -> bool {
        if predicate(node) {
            return true;
        }
        for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            if walk(child.as_ref(), path, predicate) {
                return true;
            }
            path.pop();
        }
        false
    }
    let mut path = Vec::new();
    if walk(root, &mut path, predicate) {
        Some(path)
    } else {
        None
    }
}

/// [`find_path`] specialised to a `Text` node's rendered content.
fn find_text_path(root: &WidgetNode, content: &str) -> Option<Vec<usize>> {
    find_path(root, &|n| n.__text_content_for_test() == Some(content))
}

/// [`find_path`] specialised to a Button/ToggleButton's label, read back
/// through the real C ABI ([`label_of`]) rather than trusted from source
/// order.
fn find_button_path(root: &WidgetNode, label: &str) -> Option<Vec<usize>> {
    find_path(root, &|n| {
        matches!(n.__kind_name_for_test(), "Button" | "ToggleButton") && label_of(n) == label
    })
}

/// Read a `Text` widget's rendered content, panicking with `what` in the
/// message if the node is not a `Text`.
fn read_text(node: &WidgetNode, what: &str) -> String {
    node.__text_content_for_test()
        .unwrap_or_else(|| panic!("{what}: node must be a Text widget"))
        .to_owned()
}

/// The physical-pixel centre of `rect` (a DIP rectangle from
/// `__arranged_rect_for_test()`), scaled by the runtime's committed
/// DPI/reference factor — never a hand-worked-out constant.
fn rect_center_physical(rect: DipRect, factor: f32) -> (f32, f32) {
    (
        (rect.x + rect.width / 2.0) * factor,
        (rect.y + rect.height / 2.0) * factor,
    )
}

/// Whether `(x, y)` (DIP) lies inside `rect`, left/top-inclusive,
/// right/bottom-exclusive — the same edge convention `hit.rs`'s own
/// `contains` uses (mirrored here because `hit` is a private module).
fn dip_rect_contains_point(rect: DipRect, x: f32, y: f32) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// Whether `outer` fully contains `inner`, edge-inclusive on all four
/// sides (a rectangle contains itself).
fn dip_rect_contains(outer: DipRect, inner: DipRect) -> bool {
    outer.x <= inner.x
        && inner.x + inner.width <= outer.x + outer.width
        && outer.y <= inner.y
        && inner.y + inner.height <= outer.y + outer.height
}

/// The overlap of `a` and `b`, or `None` when they do not intersect at
/// all (used by G6(c) to find, and then assert, a point doubly contained
/// in two different widgets' rectangles).
fn dip_rect_intersect(a: DipRect, b: DipRect) -> Option<DipRect> {
    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = (a.x + a.width).min(b.x + b.width);
    let y1 = (a.y + a.height).min(b.y + b.height);
    if x1 > x0 && y1 > y0 {
        Some(DipRect {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        })
    } else {
        None
    }
}

/// Every fixture's before-state: nothing focused, the lightbox `if`
/// branch absent (only the main `Grid` present). Established and read
/// back before any fixture injects input (this file's module header).
unsafe fn assert_gallery_before_state(window: *mut ffi::WasamoWindow, what: &str) {
    assert_eq!(
        ffi::__focus_path_for_test(window),
        None,
        "{what}: nothing must be focused before the fixture injects any input"
    );
    let root = (*window).root_widget.as_ref().expect("content root");
    assert_eq!(
        root.children.len(),
        1,
        "{what}: the lightbox branch must be absent (only the main Grid present) \
         before the fixture injects any input"
    );
    assert!(
        find_path(root, &|n| n.__focus_role_for_test() == "modal-scope").is_none(),
        "{what}: no modal-scope subtree may be present in the before-state"
    );
}

/// Assert that focus is on the Button/ToggleButton labelled
/// `expected_label`, discovering its path with [`find_button_path`]
/// rather than a hard-coded one (this file's module header). Checks all
/// three of the retained record, the painted indicator, and the node's
/// own label — the same pairing `modal_scope_integration.rs`'s
/// `assert_focused_stop` checks, rebuilt here because the path is
/// discovered rather than passed in.
unsafe fn assert_focus_is_labelled(
    window: *mut ffi::WasamoWindow,
    expected_label: &str,
    what: &str,
) {
    let actual_path = ffi::__focus_path_for_test(window);
    let root = (*window).root_widget.as_ref().expect("content root");
    let expected_path = find_button_path(root, expected_label);
    assert_eq!(
        actual_path, expected_path,
        "{what}: focus must be on the Button/ToggleButton labelled {expected_label:?} \
         (found there at {expected_path:?}), but the retained focus record names \
         {actual_path:?}"
    );
    let path = actual_path.unwrap_or_else(|| panic!("{what}: nothing is focused at all"));
    let node = node_at_path(root, &path);
    assert_eq!(
        node.__button_focused_for_test(),
        Some(true),
        "{what}: the node the retained record names ({path:?}) must paint the focus \
         indicator"
    );
    assert_eq!(label_of(node), expected_label);
}

// ── G1 — a thumbnail click carries which thumbnail, through the ancestor walk ─

/// See this file's module header for the full rationale. Window A covers
/// thumbnail 0 plus the ancestor-walk / resolved-target measurement plus
/// the same-thumbnail-twice agreement leg (close via Escape, reclick);
/// window B is the difference leg (thumbnail 2).
#[test]
fn g1_thumbnail_click_carries_which_thumbnail_through_the_ancestor_walk() {
    run_on_owning_runtime_thread_or_skip(
        "G1: thumbnail click carries which thumbnail, through the ancestor walk",
        move || {
            let ir = lower_ui_to_ir(GALLERY_UI);

            let caption0;
            let caption0_again;

            // Window A: thumbnail 0, the resolved-target measurement, and
            // the agreement leg (same thumbnail clicked twice).
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G1 window A baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
                assert_gallery_before_state(window, "G1 window A before-state");

                let text0_path = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    find_text_path(root, "IMG 001 #0").expect(
                        "thumbnail 0's Text (\"IMG 001 #0\") must exist in the shipped tree",
                    )
                };
                let mut box0_path = text0_path.clone();
                box0_path.pop();
                let (box0_rect, text0_rect) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    (
                        node_at_path(root, &box0_path).__arranged_rect_for_test(),
                        node_at_path(root, &text0_path).__arranged_rect_for_test(),
                    )
                };
                let box0_rect = box0_rect.expect("thumbnail 0's Box must be laid out");
                let text0_rect = text0_rect.expect("thumbnail 0's Text must be laid out");
                assert!(
                    box0_rect.width > 0.0 && box0_rect.height > 0.0,
                    "fixture stopped discriminating: thumbnail 0's rectangle {box0_rect:?} \
                     must be non-degenerate"
                );
                assert!(
                    text0_rect.width > 0.0 && text0_rect.height > 0.0,
                    "fixture stopped discriminating: thumbnail 0's Text rectangle {text0_rect:?} \
                     must be non-degenerate"
                );

                // Geometric premise, kept as a premise (not the claim
                // itself, see below): Text sits inside its Box.
                assert!(
                    dip_rect_contains(box0_rect, text0_rect),
                    "fixture stopped discriminating: thumbnail 0's Text rectangle {text0_rect:?} \
                     must sit inside its Box's rectangle {box0_rect:?}"
                );

                // The ancestor-walk evidence — a **direct measurement**
                // through `WidgetNode::__resolve_topmost_for_test`, which
                // forwards to `hit::resolve_topmost` itself (the same
                // function `hit_test_click` / `update_hover` call — a
                // second caller, not a second predicate). Coordinates are
                // DIP, the same space `arranged_rect` is in, so the centre
                // of `text0_rect` is used directly, with no scale factor.
                let (cx_dip, cy_dip) = (
                    text0_rect.x + text0_rect.width / 2.0,
                    text0_rect.y + text0_rect.height / 2.0,
                );
                let resolved_path = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    root.__resolve_topmost_for_test(cx_dip, cy_dip)
                }
                .expect("a point over thumbnail 0's Text must resolve to something");
                let resolved_kind = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    node_at_path(root, &resolved_path).__kind_name_for_test()
                };
                eprintln!(
                    "G1 measurement: __resolve_topmost_for_test({cx_dip}, {cy_dip}) = \
                     {resolved_path:?} (kind={resolved_kind:?}); thumbnail 0's Text is at \
                     {text0_path:?}, its Box at {box0_path:?}"
                );
                assert_eq!(
                    resolved_path, text0_path,
                    "measured: the point resolves to {resolved_path:?} (kind={resolved_kind:?}), \
                     not thumbnail 0's own Text path {text0_path:?} — reporting the actual \
                     measured target rather than adjusting the expectation"
                );
                assert_eq!(
                    resolved_kind, "Text",
                    "measured: the resolved node's kind is {resolved_kind:?}, not Text"
                );
                assert_eq!(
                    resolved_path[..resolved_path.len() - 1],
                    box0_path[..],
                    "the resolved Text's parent path must be the handler-bearing Box's own \
                     path {box0_path:?} — this is what makes the Box's handler firing from \
                     this point an ancestor-walk claim rather than direct dispatch"
                );

                let (cx, cy) = rect_center_physical(text0_rect, factor);

                // The click: opens the lightbox and carries index 0. Uses
                // `click_and_drain` because the agreement leg below closes
                // it again with Escape (this file's module header).
                click_and_drain(hwnd, cx, cy);
                caption0 = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        2,
                        "the click must have opened the lightbox"
                    );
                    let p = find_path(root, &|n| {
                        n.__text_content_for_test()
                            .is_some_and(|c| c.starts_with("Photo #"))
                    })
                    .expect("the caption Text must exist once the lightbox is open");
                    read_text(node_at_path(root, &p), "caption after opening thumbnail 0")
                };
                assert_eq!(
                    caption0, "Photo #0",
                    "clicking thumbnail 0 must open the lightbox on Photo #0"
                );

                // Agreement leg: close (Escape, through the authored
                // `dismiss`) and reclick the *same* thumbnail. The caption
                // must be unchanged.
                key_and_drain(hwnd, VK_ESCAPE.0);
                {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        1,
                        "the lightbox must be closed before the agreement-leg reclick"
                    );
                }
                let box0_rect_again = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    node_at_path(root, &box0_path).__arranged_rect_for_test()
                }
                .expect("thumbnail 0 must still be laid out after the round trip");
                let (cx2, cy2) = rect_center_physical(box0_rect_again, factor);
                click_and_drain(hwnd, cx2, cy2);
                caption0_again = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        2,
                        "the reclick must reopen the lightbox"
                    );
                    let p = find_path(root, &|n| {
                        n.__text_content_for_test()
                            .is_some_and(|c| c.starts_with("Photo #"))
                    })
                    .expect("the caption Text must exist once the lightbox is open again");
                    read_text(
                        node_at_path(root, &p),
                        "caption after reclicking thumbnail 0",
                    )
                };

                ffi::wasamo_window_destroy(window);
            }

            assert_eq!(
                caption0_again, caption0,
                "clicking the same thumbnail twice (with a close/reopen round trip in \
                 between, since the lightbox occludes it while open) must leave the caption \
                 unchanged"
            );

            // Window B: the difference leg — thumbnail 2.
            let caption2;
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G1 window B baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
                assert_gallery_before_state(window, "G1 window B before-state");

                let box2_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let text2_path = find_text_path(root, "IMG 003 #2")
                        .expect("thumbnail 2's Text (\"IMG 003 #2\") must exist");
                    let mut box2_path = text2_path;
                    box2_path.pop();
                    node_at_path(root, &box2_path).__arranged_rect_for_test()
                }
                .expect("thumbnail 2's Box must be laid out");
                assert!(box2_rect.width > 0.0 && box2_rect.height > 0.0);
                let (cx, cy) = rect_center_physical(box2_rect, factor);

                click_and_drain(hwnd, cx, cy);
                caption2 = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        2,
                        "the click must have opened the lightbox"
                    );
                    let p = find_path(root, &|n| {
                        n.__text_content_for_test()
                            .is_some_and(|c| c.starts_with("Photo #"))
                    })
                    .expect("the caption Text must exist once the lightbox is open");
                    read_text(node_at_path(root, &p), "caption after opening thumbnail 2")
                };

                ffi::wasamo_window_destroy(window);
            }
            assert_eq!(
                caption2, "Photo #2",
                "clicking thumbnail 2 must open the lightbox on Photo #2"
            );
            assert_ne!(
                caption0, caption2,
                "two different rows must produce two different captions; clicking the same \
                 row twice would not constrain per-item index resolution at all"
            );
        },
    );
}

// ── G2 — opening the lightbox enters the scope ──────────────────────────────

/// See this file's module header for the full rationale.
#[test]
fn g2_opening_the_lightbox_enters_the_scope() {
    run_on_owning_runtime_thread_or_skip("G2: opening the lightbox enters the scope", move || {
        let ir = lower_ui_to_ir(GALLERY_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G2 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
            assert_gallery_before_state(window, "G2 before-state");

            let box0_rect = {
                let root = (*window).root_widget.as_ref().unwrap();
                let text0_path =
                    find_text_path(root, "IMG 001 #0").expect("thumbnail 0's Text must exist");
                let mut box0_path = text0_path;
                box0_path.pop();
                node_at_path(root, &box0_path).__arranged_rect_for_test()
            }
            .expect("thumbnail 0's Box must be laid out");
            assert!(box0_rect.width > 0.0 && box0_rect.height > 0.0);
            let (cx, cy) = rect_center_physical(box0_rect, factor);

            click_and_drain(hwnd, cx, cy);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children.len(),
                    2,
                    "the click must have opened the lightbox"
                );
            }

            assert_focus_is_labelled(window, "<", "G2 after opening the lightbox");

            ffi::wasamo_window_destroy(window);
        }
    });
}

// ── G3 — Esc closes through dismiss, and focus restores (two legs) ─────────

/// See this file's module header for the full rationale. Leg (a): nothing
/// focused beforehand. Leg (b): Tab once first, so a real Button was
/// focused before the lightbox opened — the leg that makes "restores"
/// mean something rather than merely "clears".
#[test]
fn g3_escape_closes_through_dismiss_and_focus_restores() {
    run_on_owning_runtime_thread_or_skip(
        "G3: Escape closes through dismiss and focus restores",
        move || {
            let ir = lower_ui_to_ir(GALLERY_UI);

            // Leg (a): nothing focused beforehand.
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G3a baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
                assert_gallery_before_state(window, "G3a before-state");

                let box0_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let text0_path =
                        find_text_path(root, "IMG 001 #0").expect("thumbnail 0's Text must exist");
                    let mut box0_path = text0_path;
                    box0_path.pop();
                    node_at_path(root, &box0_path).__arranged_rect_for_test()
                }
                .expect("thumbnail 0's Box must be laid out");
                let (cx, cy) = rect_center_physical(box0_rect, factor);
                click_and_drain(hwnd, cx, cy);
                {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(root.children.len(), 2, "G3a: the lightbox must have opened");
                }

                key_and_drain(hwnd, VK_ESCAPE.0);

                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children.len(),
                    1,
                    "G3a: Escape must close the lightbox through the authored dismiss handler"
                );
                assert!(
                    find_path(root, &|n| n
                        .__text_content_for_test()
                        .is_some_and(|c| c.starts_with("Photo #")))
                    .is_none(),
                    "G3a: the caption Text must no longer be findable once the lightbox has \
                     closed"
                );
                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    None,
                    "G3a: with nothing focused before the lightbox opened, focus must be None \
                     again after Escape"
                );

                ffi::wasamo_window_destroy(window);
            }

            // Leg (b): Tab once first, landing on the toolbar's first stop
            // (read back by label, not assumed).
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G3b baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
                assert_gallery_before_state(window, "G3b before-state");

                send_tab(hwnd);
                let first_stop_path = ffi::__focus_path_for_test(window)
                    .expect("a single Tab must focus the domain's first stop");
                let first_stop_label = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    label_of(node_at_path(root, &first_stop_path))
                };
                eprintln!(
                    "G3b measurement: the first Tab stop is {first_stop_label:?} at \
                     {first_stop_path:?}"
                );

                let box0_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let text0_path =
                        find_text_path(root, "IMG 001 #0").expect("thumbnail 0's Text must exist");
                    let mut box0_path = text0_path;
                    box0_path.pop();
                    node_at_path(root, &box0_path).__arranged_rect_for_test()
                }
                .expect("thumbnail 0's Box must be laid out");
                let (cx, cy) = rect_center_physical(box0_rect, factor);
                click_and_drain(hwnd, cx, cy);

                // Setup check (G2 already pins this claim in isolation):
                // entry moved focus into the scope.
                assert_focus_is_labelled(window, "<", "G3b after opening");

                key_and_drain(hwnd, VK_ESCAPE.0);

                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children.len(),
                    1,
                    "G3b: Escape must close the lightbox"
                );
                assert_eq!(
                    ffi::__focus_path_for_test(window),
                    Some(first_stop_path.clone()),
                    "G3b: focus must restore to the widget focused before the lightbox opened \
                     ({first_stop_label:?} at {first_stop_path:?})"
                );
                assert_eq!(
                    label_of(node_at_path(root, &first_stop_path)),
                    first_stop_label,
                    "G3b: the restored node's own label must be unchanged"
                );

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── G4 — arrow keys reach the scope's key-down handlers, only while present ─

/// See this file's module header for the full rationale. Part 1: the
/// caption steps under Left/Right while the lightbox is open. Part 2, the
/// agreement/tripwire leg: the identical key does *not* reach the host key
/// slot while the lightbox is open, and *does* once it is closed.
#[test]
fn g4_arrow_keys_reach_the_scope_key_down_handlers_only_while_present() {
    run_on_owning_runtime_thread_or_skip(
        "G4: arrow keys reach the scope's key-down handlers only while present",
        move || {
            let ir = lower_ui_to_ir(GALLERY_UI);

            // Part 1: ArrowRight / ArrowLeft step the caption.
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G4a baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
                assert_gallery_before_state(window, "G4a before-state");

                let box2_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let text2_path =
                        find_text_path(root, "IMG 003 #2").expect("thumbnail 2's Text must exist");
                    let mut box2_path = text2_path;
                    box2_path.pop();
                    node_at_path(root, &box2_path).__arranged_rect_for_test()
                }
                .expect("thumbnail 2's Box must be laid out");
                let (cx, cy) = rect_center_physical(box2_rect, factor);
                click_and_drain(hwnd, cx, cy);

                let read_caption = |window: *mut ffi::WasamoWindow, what: &str| -> String {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let p = find_path(root, &|n| {
                        n.__text_content_for_test()
                            .is_some_and(|c| c.starts_with("Photo #"))
                    })
                    .unwrap_or_else(|| panic!("{what}: the caption Text must exist"));
                    read_text(node_at_path(root, &p), what)
                };

                assert_eq!(
                    read_caption(window, "caption after opening thumbnail 2"),
                    "Photo #2",
                    "fixture stopped discriminating: opening thumbnail 2 must show Photo #2"
                );

                send_key(hwnd, VK_RIGHT.0);
                assert_eq!(
                    read_caption(window, "caption after ArrowRight"),
                    "Photo #3",
                    "ArrowRight must reach the authored key-down(\"ArrowRight\") handler"
                );

                send_key(hwnd, VK_LEFT.0);
                assert_eq!(
                    read_caption(window, "caption after one ArrowLeft"),
                    "Photo #2"
                );
                send_key(hwnd, VK_LEFT.0);
                assert_eq!(
                    read_caption(window, "caption after two ArrowLeft"),
                    "Photo #1",
                    "two ArrowLeft presses from Photo #3 must land on Photo #1"
                );

                ffi::wasamo_window_destroy(window);
            }

            // Part 2: the agreement/tripwire leg via the host key slot.
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G4b baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
                assert_gallery_before_state(window, "G4b before-state");

                let seen = install_key_slot_recorder(window);

                let box0_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let text0_path =
                        find_text_path(root, "IMG 001 #0").expect("thumbnail 0's Text must exist");
                    let mut box0_path = text0_path;
                    box0_path.pop();
                    node_at_path(root, &box0_path).__arranged_rect_for_test()
                }
                .expect("thumbnail 0's Box must be laid out");
                let (cx, cy) = rect_center_physical(box0_rect, factor);
                click_and_drain(hwnd, cx, cy);

                send_key(hwnd, VK_RIGHT.0);
                assert!(
                    seen.borrow().is_empty(),
                    "G4b: with the lightbox open, ArrowRight must be consumed by the authored \
                     key-down handler and never reach the host key slot; saw {:?}",
                    seen.borrow()
                );

                key_and_drain(hwnd, VK_ESCAPE.0);
                {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        1,
                        "G4b: the lightbox must be closed before the fallthrough leg"
                    );
                }

                send_key(hwnd, VK_RIGHT.0);
                assert_eq!(
                    seen.borrow().as_slice(),
                    &[VK_RIGHT.0],
                    "G4b: with the lightbox closed, ArrowRight must reach the host key slot"
                );

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── G5 — a background click is blocked while the lightbox is open ──────────

/// See this file's module header for the full rationale.
///
/// **Three measured findings that changed this fixture's shape from the
/// brief's design, all reported rather than routed around:**
///
/// 1. **No toolbar `ToggleButton` is reliably clickable at this client
///    width at all** (found while building this fixture — a shipped
///    defect, unrelated to the lightbox). `Grid`'s reverse-order
///    resolution visits the later-declared toolbar-right Cell (`Scroll
///    down` / `Scroll up` / `Open lightbox`) before the toolbar-left one
///    (`All` / `Albums` / `Favorites`), and the toolbar-right *container's
///    own arranged rectangle* — not just its three buttons' — spans
///    `x=[17.8, 360.0]`, `y=[0, 56]` (measured): its own padding plus
///    content width, right-aligned to the client edge. Since a
///    non-clipping container whose own rectangle contains the point is
///    itself a valid resolution target (`hit.rs`'s `resolve_topmost` step
///    3) whenever none of *its* children match first, `Albums`
///    (`x=[63.8, 142.4]`) and `Favorites` (`x=[150.4, 237.0]`) sit
///    **entirely inside** the toolbar-right container's rectangle and are
///    therefore unreachable by any click, full stop — not merely
///    overlapped by a specific sibling Button, occluded by the container
///    itself even in the visual gaps between its buttons. Only `All`
///    (`x=[8, 55.8]`) has a small reachable sliver (`x<17.8`), and it
///    cannot discriminate a flip (`tab_all_selected` starts `true`). This
///    fixture therefore uses **`Scroll down`** (a toolbar-right member,
///    reliably reachable by construction) as the background control, and
///    thumbnail 0's rectangle shift — already established in
///    [`g6_scrolled_hit_testing`] — as the read-back, instead of a
///    `ToggleButton`'s `checked` state.
/// 2. **Fact 6 ("which node blocks the click: the scrim or the lightbox's
///    own Grid") is answered by a direct measurement**, through
///    `WidgetNode::__resolve_topmost_for_test` (this file's module header,
///    "The resolved-target accessor" — added by the lead after this
///    fixture first reported that `ffi::__hover_target_for_test` cannot
///    witness a non-Button resolved target at all, the same limitation
///    G1 hit). **Measured: `__resolve_topmost_for_test` on the background
///    point resolves to path `[1, 1]`, kind `Grid`** — the lightbox
///    subtree's root is `[1]`, so the resolved node is the lightbox's own
///    `Grid` (`[1, 1]`'s last index), reached through the `Grid`'s own
///    step-3 self-match (`hit.rs`'s `resolve_topmost`) rather than one of
///    its children — i.e. none of the `Grid`'s own Cell contents (the
///    photo `Box`, the caption `VStack`, the `<` / `>` / `x` Buttons,
///    positioned by the degenerate fixed-track layout T10's start gate
///    fact 8 records) happen to sit at this particular background point,
///    so resolution falls through to the `Grid` itself. This **confirms**
///    what an earlier geometric deduction over this same tree predicted,
///    but is no longer that deduction — it is what the production
///    resolver itself returned.
/// 3. The lightbox is opened with **`Open lightbox`** rather than a
///    thumbnail click here, so `Scroll down`'s own click point never has
///    to change between the agreement and difference legs.
#[test]
fn g5_a_background_click_is_blocked_while_the_lightbox_is_open() {
    run_on_owning_runtime_thread_or_skip(
        "G5: a background click is blocked while the lightbox is open",
        move || {
            let ir = lower_ui_to_ir(GALLERY_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G5 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
                assert_gallery_before_state(window, "G5 before-state");

                let (wrap_path, scroll_down_rect) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let wrap_path = find_path(root, &|n| n.__kind_name_for_test() == "WrapPanel")
                        .expect("the WrapPanel must exist");
                    let p = find_button_path(root, "Scroll down")
                        .expect("the Scroll down Button must exist");
                    let r = node_at_path(root, &p)
                        .__arranged_rect_for_test()
                        .expect("Scroll down must be laid out");
                    (wrap_path, r)
                };
                assert!(
                    scroll_down_rect.width > 0.0 && scroll_down_rect.height > 0.0,
                    "fixture stopped discriminating: Scroll down's rectangle \
                     {scroll_down_rect:?} must be non-degenerate"
                );
                let (sx, sy) = rect_center_physical(scroll_down_rect, factor);

                let thumb0_rect_before = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    node_at_path(root, &wrap_path).children[0]
                        .__arranged_rect_for_test()
                        .expect("thumbnail 0 must be laid out")
                };

                // Agreement leg: lightbox closed, Scroll down's own centre
                // (a reliably-reachable toolbar-right member — finding 1)
                // is live. click_and_drain: the offset-y write is
                // size-affecting, so the moved rectangle is only current
                // after a layout pass (Phase 2) — same reasoning as
                // `g6_scrolled_hit_testing`.
                click_and_drain(hwnd, sx, sy);
                let thumb0_rect_after_closed = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    node_at_path(root, &wrap_path).children[0]
                        .__arranged_rect_for_test()
                        .expect("thumbnail 0 must still be laid out")
                };
                assert_ne!(
                    thumb0_rect_after_closed, thumb0_rect_before,
                    "G5: with the lightbox closed, clicking Scroll down must move thumbnail \
                     0's rectangle (before={thumb0_rect_before:?})"
                );

                // Open the lightbox via `Open lightbox` (finding 3) —
                // reliably reachable for the same reason `Scroll down` is.
                let open_lightbox_rect = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let p = find_button_path(root, "Open lightbox")
                        .expect("the Open lightbox Button must exist");
                    node_at_path(root, &p)
                        .__arranged_rect_for_test()
                        .expect("Open lightbox must be laid out")
                };
                let (olx, oly) = rect_center_physical(open_lightbox_rect, factor);
                click_and_drain(hwnd, olx, oly);
                {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        2,
                        "the lightbox must be open for the difference leg"
                    );
                }

                // Re-derive Scroll down's rectangle after the lightbox
                // opened — never trust a rectangle cached across a state
                // write.
                let scroll_down_rect_now = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let p = find_button_path(root, "Scroll down").unwrap();
                    node_at_path(root, &p).__arranged_rect_for_test()
                }
                .expect("Scroll down must still be laid out");
                assert_eq!(
                    scroll_down_rect_now, scroll_down_rect,
                    "Scroll down's own rectangle must be unaffected by the lightbox opening \
                     (the lightbox is a root-ZStack sibling overlay, not nested inside the \
                     toolbar Grid)"
                );
                let (sx2_dip, sy2_dip) = (
                    scroll_down_rect_now.x + scroll_down_rect_now.width / 2.0,
                    scroll_down_rect_now.y + scroll_down_rect_now.height / 2.0,
                );
                let (sx2, sy2) = rect_center_physical(scroll_down_rect_now, factor);

                // Fact 6 — a **direct measurement** through
                // `WidgetNode::__resolve_topmost_for_test` (this file's
                // module header, "The resolved-target accessor"): which
                // node does the background point actually resolve to while
                // the lightbox is open? Coordinates are DIP (the same
                // space `arranged_rect` is in), not the physical `sx2`/
                // `sy2` the click below uses.
                let (resolved_path, resolved_kind, lightbox_path) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let path = root.__resolve_topmost_for_test(sx2_dip, sy2_dip).expect(
                        "a point over the toolbar must resolve to something even with the \
                             lightbox open",
                    );
                    let kind = node_at_path(root, &path).__kind_name_for_test();
                    let lightbox_path =
                        find_path(root, &|n| n.__focus_role_for_test() == "modal-scope")
                            .expect("the lightbox scope must be present");
                    (path, kind, lightbox_path)
                };
                let relative_path = resolved_path.strip_prefix(lightbox_path.as_slice());
                eprintln!(
                    "G5 fact 6 measurement: __resolve_topmost_for_test({sx2_dip}, {sy2_dip}) = \
                     {resolved_path:?} (kind={resolved_kind:?}); lightbox subtree root=\
                     {lightbox_path:?}, path relative to lightbox={relative_path:?}"
                );
                assert!(
                    resolved_path.starts_with(lightbox_path.as_slice()),
                    "fact 6: the background point must resolve to a node *inside* the \
                     lightbox subtree (the covering widget that blocks the click), not to the \
                     toolbar underneath; measured kind={resolved_kind:?}, \
                     path={resolved_path:?}, lightbox subtree root={lightbox_path:?}"
                );

                // Difference leg: the identical click, lightbox open,
                // leaves thumbnail 0's rectangle unchanged (Scroll down is
                // not reached).
                send_click(hwnd, sx2, sy2);
                let thumb0_rect_after_open = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    node_at_path(root, &wrap_path).children[0].__arranged_rect_for_test()
                };
                assert_eq!(
                    thumb0_rect_after_open,
                    Some(thumb0_rect_after_closed),
                    "G5: with the lightbox open, the identical click on Scroll down's centre \
                     must not move thumbnail 0's rectangle further"
                );
                {
                    let root = (*window).root_widget.as_ref().unwrap();
                    assert_eq!(
                        root.children.len(),
                        2,
                        "the lightbox must still be open (Scroll down's click must not have \
                         dismissed it either)"
                    );
                }

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

// ── G6 — scrolled hit-testing: the clip bound's first production consumer ──

/// See this file's module header for the full rationale. (a) the
/// geometric premise is measured, not assumed. (b) the thumbnail newly
/// inside the viewport opens the lightbox at its own index. (c) a point
/// inside both a scrolled-away thumbnail's rectangle and a toolbar
/// Button's rectangle resolves to the toolbar — using `Open lightbox`
/// rather than a `ToggleButton` (see this fixture's own comment on leg
/// (c) for the measured reason: no `ToggleButton` is reachable here at
/// all).
#[test]
fn g6_scrolled_hit_testing() {
    run_on_owning_runtime_thread_or_skip("G6: scrolled hit-testing", move || {
        let ir = lower_ui_to_ir(GALLERY_UI);
        unsafe {
            let window = load_window(&ir);
            let hwnd = (*window).hwnd;
            normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G6 baseline");
            let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
            assert_gallery_before_state(window, "G6 before-state");

            let (wrap_path, scrollview_path) = {
                let root = (*window).root_widget.as_ref().unwrap();
                (
                    find_path(root, &|n| n.__kind_name_for_test() == "WrapPanel")
                        .expect("the WrapPanel must exist"),
                    find_path(root, &|n| n.__kind_name_for_test() == "ScrollView")
                        .expect("the ScrollView must exist"),
                )
            };

            let thumb_rects_before: Vec<DipRect> = {
                let root = (*window).root_widget.as_ref().unwrap();
                let wrap = node_at_path(root, &wrap_path);
                wrap.children
                    .iter()
                    .map(|c| {
                        c.__arranged_rect_for_test()
                            .expect("every thumbnail must be laid out before scrolling")
                    })
                    .collect()
            };
            assert_eq!(
                thumb_rects_before.len(),
                18,
                "fixture stopped discriminating: the gallery must generate all 18 thumbnails"
            );

            let viewport_rect = {
                let root = (*window).root_widget.as_ref().unwrap();
                node_at_path(root, &scrollview_path).__arranged_rect_for_test()
            }
            .expect("the ScrollView must be laid out");

            // Geometric premise: before any scroll, thumbnail 0 sits
            // inside the viewport.
            assert!(
                dip_rect_contains(viewport_rect, thumb_rects_before[0]),
                "fixture stopped discriminating: before any scroll, thumbnail 0 {:?} must sit \
                 inside the viewport {:?}",
                thumb_rects_before[0],
                viewport_rect
            );

            let scroll_down_rect = {
                let root = (*window).root_widget.as_ref().unwrap();
                let p = find_button_path(root, "Scroll down")
                    .expect("the Scroll down Button must exist");
                node_at_path(root, &p).__arranged_rect_for_test()
            }
            .expect("Scroll down must be laid out");
            let (sx, sy) = rect_center_physical(scroll_down_rect, factor);

            // click_and_drain: the offset-y write is size-affecting
            // (`widget.rs`'s `set_property`), so the moved rectangles are
            // only current after a layout pass (Phase 2) has run.
            click_and_drain(hwnd, sx, sy);

            let thumb_rects_after: Vec<DipRect> = {
                let root = (*window).root_widget.as_ref().unwrap();
                let wrap = node_at_path(root, &wrap_path);
                wrap.children
                    .iter()
                    .map(|c| {
                        c.__arranged_rect_for_test()
                            .expect("every thumbnail must be laid out after scrolling")
                    })
                    .collect()
            };
            let viewport_rect_after = {
                let root = (*window).root_widget.as_ref().unwrap();
                node_at_path(root, &scrollview_path).__arranged_rect_for_test()
            }
            .expect("the ScrollView must still be laid out");

            // (a) geometric premise, post-scroll: thumbnail 0 moved up and
            // now overlaps the toolbar band (the region above the
            // viewport's own top edge — derived from the measured
            // viewport rectangle, not a hardcoded row height).
            assert!(
                thumb_rects_after[0].y < thumb_rects_before[0].y,
                "thumbnail 0 must have moved up: before={:?} after={:?}",
                thumb_rects_before[0],
                thumb_rects_after[0]
            );
            let toolbar_band = DipRect {
                x: 0.0,
                y: 0.0,
                width: CLIENT_W as f32,
                height: viewport_rect_after.y,
            };
            assert!(
                dip_rect_intersect(thumb_rects_after[0], toolbar_band).is_some(),
                "thumbnail 0's post-scroll rectangle {:?} must overlap the toolbar band {:?}",
                thumb_rects_after[0],
                toolbar_band
            );
            assert!(
                !dip_rect_contains(viewport_rect_after, thumb_rects_after[0]),
                "thumbnail 0 must no longer sit inside the viewport {:?}: {:?}",
                viewport_rect_after,
                thumb_rects_after[0]
            );

            // Which index is now first inside the viewport, that was not
            // before? Found by searching the measured rectangles, not
            // computed by arithmetic.
            let newly_visible = (0..18usize).find(|&i| {
                !dip_rect_contains(viewport_rect, thumb_rects_before[i])
                    && dip_rect_contains(viewport_rect_after, thumb_rects_after[i])
            });
            let newly_visible = newly_visible.unwrap_or_else(|| {
                panic!(
                    "G6(a): after scrolling, no thumbnail newly entered the viewport — \
                     before={thumb_rects_before:?} after={thumb_rects_after:?} \
                     viewport_before={viewport_rect:?} viewport_after={viewport_rect_after:?}"
                )
            });
            eprintln!(
                "G6 measurement: thumbnail {newly_visible} is now the first one inside the \
                 viewport, at {:?}",
                thumb_rects_after[newly_visible]
            );

            // (b) a click on the now-visible thumbnail opens the lightbox
            // with its own index.
            let (nx, ny) = rect_center_physical(thumb_rects_after[newly_visible], factor);
            click_and_drain(hwnd, nx, ny);
            let caption = {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children.len(),
                    2,
                    "the click must have opened the lightbox"
                );
                let p = find_path(root, &|n| {
                    n.__text_content_for_test()
                        .is_some_and(|c| c.starts_with("Photo #"))
                })
                .expect("the caption Text must exist once the lightbox is open");
                read_text(
                    node_at_path(root, &p),
                    "caption after opening the scrolled-in thumbnail",
                )
            };
            assert_eq!(
                caption,
                format!("Photo #{newly_visible}"),
                "G6(b): clicking the now-visible thumbnail must open the lightbox at its own \
                 index"
            );

            key_and_drain(hwnd, VK_ESCAPE.0);
            {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children.len(),
                    1,
                    "G6(b) cleanup: the lightbox must close before leg (c)"
                );
            }

            // Re-derive every rectangle fresh before leg (c) — never trust
            // a rectangle cached across the open/close round trip above.
            let thumb_rects_c: Vec<DipRect> = {
                let root = (*window).root_widget.as_ref().unwrap();
                let wrap = node_at_path(root, &wrap_path);
                wrap.children
                    .iter()
                    .map(|c| {
                        c.__arranged_rect_for_test()
                            .expect("thumbnail must be laid out")
                    })
                    .collect()
            };
            let viewport_rect_c = {
                let root = (*window).root_widget.as_ref().unwrap();
                node_at_path(root, &scrollview_path).__arranged_rect_for_test()
            }
            .expect("the ScrollView must be laid out");

            // **No toolbar `ToggleButton` is usable here** — the same
            // finding `g5_a_background_click_is_blocked_while_the_lightbox_is_open`'s
            // doc comment records in full: `Albums` / `Favorites` sit
            // entirely inside the toolbar-right container's own
            // arranged rectangle (`x=[17.8, 360.0]`), so no point in
            // either one is ever reachable by a click, and `All` cannot
            // discriminate (pre-checked). This leg uses **`Open
            // lightbox`** instead — a toolbar-right member, reliably
            // reachable by construction (`g5`'s finding 1) — with the
            // caption's *index* as the read-back: `Open lightbox`'s own
            // handler does not touch `selected_index`, while the
            // scrolled-away thumbnail's handler would set it to that
            // thumbnail's own index, so the two resolutions are
            // distinguishable by which index the caption shows after the
            // click, using `newly_visible` (leg (b)'s index, left
            // unchanged by every click since) as the "nothing else
            // touched it" baseline.
            let open_lightbox_rect = {
                let root = (*window).root_widget.as_ref().unwrap();
                let p = find_button_path(root, "Open lightbox")
                    .expect("the Open lightbox Button must exist");
                node_at_path(root, &p)
                    .__arranged_rect_for_test()
                    .expect("Open lightbox must be laid out")
            };

            // (c) find a point inside both a scrolled-away thumbnail's
            // rectangle and Open lightbox's rectangle.
            let scrolled_away: Vec<usize> = (0..18usize)
                .filter(|&i| !dip_rect_contains(viewport_rect_c, thumb_rects_c[i]))
                .collect();

            let mut doubly_contained: Option<(usize, (f32, f32))> = None;
            for &i in &scrolled_away {
                if let Some(overlap) = dip_rect_intersect(thumb_rects_c[i], open_lightbox_rect) {
                    let point = (
                        overlap.x + overlap.width / 2.0,
                        overlap.y + overlap.height / 2.0,
                    );
                    doubly_contained = Some((i, point));
                    break;
                }
            }

            let Some((thumb_idx, point)) = doubly_contained else {
                panic!(
                    "G6(c): no point at this {CLIENT_W}x{CLIENT_H} client is inside both a \
                     scrolled-away thumbnail's rectangle and Open lightbox's rectangle \
                     {open_lightbox_rect:?}; scrolled-away thumbnails={scrolled_away:?}, \
                     rects={:?} — reporting rather than substituting a weaker assertion",
                    scrolled_away
                        .iter()
                        .map(|&i| thumb_rects_c[i])
                        .collect::<Vec<_>>()
                );
            };
            eprintln!(
                "G6(c) measurement: thumbnail {thumb_idx}'s rectangle {:?} and Open lightbox's \
                 rectangle {open_lightbox_rect:?} both contain point {point:?}",
                thumb_rects_c[thumb_idx]
            );
            assert!(
                dip_rect_contains_point(thumb_rects_c[thumb_idx], point.0, point.1),
                "the chosen point {point:?} must lie inside thumbnail {thumb_idx}'s rectangle \
                 {:?}",
                thumb_rects_c[thumb_idx]
            );
            assert!(
                dip_rect_contains_point(open_lightbox_rect, point.0, point.1),
                "the chosen point {point:?} must lie inside Open lightbox's rectangle \
                 {open_lightbox_rect:?}"
            );
            assert_ne!(
                thumb_idx, newly_visible,
                "fixture stopped discriminating: the scrolled-away thumbnail {thumb_idx} used \
                 for this leg must differ from leg (b)'s own index {newly_visible}, or a wrong \
                 resolution could coincidentally read back the same caption"
            );

            // The click opens the lightbox — via whichever handler
            // actually runs. Uses `click_and_drain`: if it is Open
            // lightbox's handler, that materialises the lightbox subtree
            // (this file's module header, "opening the lightbox always
            // uses click_and_drain").
            let (px, py) = (point.0 * factor, point.1 * factor);
            click_and_drain(hwnd, px, py);

            let caption_c = {
                let root = (*window).root_widget.as_ref().unwrap();
                assert_eq!(
                    root.children.len(),
                    2,
                    "the click at the doubly-contained point must have opened the lightbox \
                     (via *some* handler — Open lightbox's or the thumbnail's — this assertion \
                     alone does not yet say which)"
                );
                let p = find_path(root, &|n| {
                    n.__text_content_for_test()
                        .is_some_and(|c| c.starts_with("Photo #"))
                })
                .expect("the caption Text must exist once the lightbox is open");
                read_text(
                    node_at_path(root, &p),
                    "caption after the doubly-contained click",
                )
            };
            assert_eq!(
                caption_c,
                format!("Photo #{newly_visible}"),
                "G6(c): the click at the doubly-contained point must reach Open lightbox \
                 (selected_index must stay at leg (b)'s {newly_visible}, not change to the \
                 scrolled-away thumbnail {thumb_idx}'s own index)"
            );

            ffi::wasamo_window_destroy(window);
        }
    });
}

// ── G7 — the overflowing toolbar swallows clicks aimed at the tab buttons ──

/// **A runnable measurement of a known, owner-dispositioned layout
/// observation**
/// (`process/milestone-4/phase-2/requirements/constraints.md` §6): the
/// gallery's toolbar `Row` overlaps at client widths narrower than its
/// content — an owner on-device observation, scale-independent, whose
/// *overflow semantics* (what the `Row`/`HStack` does when its content
/// does not fit) were sent to M4-Phase 4 as a layout design question
/// outside this phase's scope. **§6 records that the overlap is visible;
/// it does not measure that the overlap also makes the overlapped
/// widgets unclickable.** G5's and G6(c)'s own doc comments already state
/// that in prose, as the reason neither fixture picks a toolbar
/// `ToggleButton` the way the task brief expected; this fixture is the
/// artifact behind that prose — the same three resolutions, measured
/// through `WidgetNode::__resolve_topmost_for_test` and asserted, rather
/// than described.
///
/// **The cause is layout overflow, not a routing defect.**
/// `hit::resolve_topmost`'s step 3 — a non-clipping container whose own
/// rectangle contains the point is itself a valid resolution target when
/// none of its children match first — is `docs/dsl_spec.md` §4.19's own
/// stated behaviour ("every widget with a visual is a candidate"),
/// working exactly as specified. What produces the overlap is the
/// toolbar-right `HStack`'s own arranged rectangle exceeding its `Grid`
/// Cell's column and extending back over the toolbar-left Cell — a
/// layout fact, measured below, not a hit-testing one. **Measured while
/// writing this fixture, and not predicted by the earlier prose: the
/// toolbar-*left* `HStack` overflows too**, rightward past its own
/// column, so both containers' own rectangles extend toward each other
/// and each contains the other's content region — which is also why
/// `All`'s own centre (not only `Albums`' / `Favorites`') resolves to a
/// toolbar-right Button below, even though `All` cannot be used as this
/// fixture's behavioural discriminator (it starts `checked: true`).
///
/// **The overlap is client-width-driven, so it is not necessarily
/// present at the window size a user actually runs the gallery at.**
/// Constraints §6 records it as scale-independent (100% reproduces it)
/// but that is orthogonal to width: the overlap is *caused* by width, and
/// a wider client may not reproduce it at all.
///
/// **Re-trigger.** If the `HStack` overflow semantics change (M4-Phase 4)
/// so the toolbar no longer overlaps at this client width, the
/// assertions below — that a click at `Albums`' / `Favorites`' own
/// centre does *not* resolve to that button — will go red. **That red
/// means the overflow was fixed and this fixture should be revisited
/// (relaxed or removed), not that something broke.** Each assertion's
/// own message repeats this.
#[test]
fn g7_the_overflowing_toolbar_swallows_clicks_aimed_at_the_tab_buttons() {
    run_on_owning_runtime_thread_or_skip(
        "G7: the overflowing toolbar swallows clicks aimed at the tab buttons",
        move || {
            let ir = lower_ui_to_ir(GALLERY_UI);
            unsafe {
                let window = load_window(&ir);
                let hwnd = (*window).hwnd;
                normalise_to_reference_baseline(window, CLIENT_W, CLIENT_H, "G7 baseline");
                let factor = ffi::__window_scale_dpi_for_test(window) as f32 / REFERENCE_DPI as f32;
                assert_gallery_before_state(window, "G7 before-state");

                // The overlap as a measured layout fact: the toolbar-right
                // HStack's own rectangle — its **direct parent**, not a
                // rect-containment search. Measured while writing this
                // fixture: containment alone is ambiguous here — the
                // toolbar-*left* HStack's own rectangle (`All`'s direct
                // parent) *also* contains Scroll down's rectangle, because
                // it overflows rightward past its own Cell column the same
                // way the toolbar-right HStack overflows leftward past
                // its. Both are printed below so the ambiguity itself is
                // in the artifact as measured numbers.
                let scroll_down_path = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    find_button_path(root, "Scroll down").expect("Scroll down must exist")
                };
                let mut toolbar_right_path = scroll_down_path.clone();
                toolbar_right_path.pop();
                let all_path = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    find_button_path(root, "All").expect("All must exist")
                };
                let mut toolbar_left_path = all_path;
                toolbar_left_path.pop();

                let (scroll_down_rect, toolbar_right_rect, toolbar_left_rect) = {
                    let root = (*window).root_widget.as_ref().unwrap();
                    let scroll_down_rect = node_at_path(root, &scroll_down_path)
                        .__arranged_rect_for_test()
                        .expect("Scroll down must be laid out");
                    let right = node_at_path(root, &toolbar_right_path);
                    assert_eq!(
                        right.__kind_name_for_test(),
                        "HStack",
                        "Scroll down's direct parent must be the toolbar-right HStack"
                    );
                    let left = node_at_path(root, &toolbar_left_path);
                    assert_eq!(
                        left.__kind_name_for_test(),
                        "HStack",
                        "All's direct parent must be the toolbar-left HStack"
                    );
                    (
                        scroll_down_rect,
                        right
                            .__arranged_rect_for_test()
                            .expect("the toolbar-right HStack must be laid out"),
                        left.__arranged_rect_for_test()
                            .expect("the toolbar-left HStack must be laid out"),
                    )
                };
                eprintln!(
                    "G7 measurement: toolbar-right HStack path={toolbar_right_path:?} \
                     rect={toolbar_right_rect:?} (Scroll down's rect={scroll_down_rect:?}); \
                     toolbar-left HStack path={toolbar_left_path:?} \
                     rect={toolbar_left_rect:?} — both HStacks' own rectangles overflow past \
                     their Grid Cell's column toward each other, and both happen to contain \
                     Scroll down's rectangle, which is why this fixture identifies \
                     toolbar-right by direct parentage rather than by rectangle containment"
                );

                // Measure each tab button's own-centre resolution — every
                // line prints, whatever the outcome.
                let root = (*window).root_widget.as_ref().unwrap();
                let mut own_paths: Vec<(&str, Vec<usize>, DipRect)> = Vec::new();
                for label in ["All", "Albums", "Favorites"] {
                    let own_path = find_button_path(root, label)
                        .unwrap_or_else(|| panic!("the {label} ToggleButton must exist"));
                    let own_rect = node_at_path(root, &own_path)
                        .__arranged_rect_for_test()
                        .unwrap_or_else(|| panic!("{label} must be laid out"));
                    let (cx_dip, cy_dip) = (
                        own_rect.x + own_rect.width / 2.0,
                        own_rect.y + own_rect.height / 2.0,
                    );
                    let resolved_path = root.__resolve_topmost_for_test(cx_dip, cy_dip);
                    let (resolved_kind, resolved_label) = match &resolved_path {
                        Some(rp) => {
                            let node = node_at_path(root, rp);
                            let kind = node.__kind_name_for_test();
                            let lbl = if matches!(kind, "Button" | "ToggleButton") {
                                Some(label_of(node))
                            } else {
                                None
                            };
                            (Some(kind), lbl)
                        }
                        None => (None, None),
                    };
                    eprintln!(
                        "G7 measurement: {label:?} own_path={own_path:?} own_rect={own_rect:?} \
                         centre=({cx_dip}, {cy_dip}) resolved_path={resolved_path:?} \
                         resolved_kind={resolved_kind:?} resolved_label={resolved_label:?}"
                    );

                    if label == "Albums" || label == "Favorites" {
                        assert_ne!(
                            resolved_path,
                            Some(own_path.clone()),
                            "G7 tripwire (constraints.md §6, M4-Phase 4 owns the Row/HStack \
                             overflow semantics): a click at {label}'s own centre must NOT \
                             resolve to {label} at this client width — measured \
                             resolved_path={resolved_path:?} (kind={resolved_kind:?}, \
                             label={resolved_label:?}) equalled {label}'s own path \
                             {own_path:?}. If this goes red, the toolbar overflow was fixed (or \
                             this client width no longer reproduces it) — revisit this \
                             fixture, it does not mean something broke."
                        );
                    }

                    own_paths.push((label, own_path, own_rect));
                }

                // The behaviour leg, not only the geometry: a click at
                // Albums' centre must leave every tab button's `checked`
                // unchanged — the "authored, accepted, silently does
                // nothing" shape this phase exists to catch.
                let albums_rect = own_paths
                    .iter()
                    .find(|(label, _, _)| *label == "Albums")
                    .map(|(_, _, rect)| *rect)
                    .expect("Albums must have been measured above");
                let checked_before: Vec<(&str, Option<bool>)> = own_paths
                    .iter()
                    .map(|(label, path, _)| {
                        (
                            *label,
                            node_at_path(root, path).__togglebutton_checked_for_test(),
                        )
                    })
                    .collect();

                let (ax, ay) = rect_center_physical(albums_rect, factor);
                send_click(hwnd, ax, ay);

                let root = (*window).root_widget.as_ref().unwrap();
                let checked_after: Vec<(&str, Option<bool>)> = own_paths
                    .iter()
                    .map(|(label, path, _)| {
                        (
                            *label,
                            node_at_path(root, path).__togglebutton_checked_for_test(),
                        )
                    })
                    .collect();
                eprintln!(
                    "G7 measurement: checked_before={checked_before:?} \
                     checked_after={checked_after:?} (click at Albums' own centre)"
                );
                assert_eq!(
                    checked_after, checked_before,
                    "G7: a click at Albums' own centre must leave every tab button's checked \
                     state unchanged — the authored `clicked` handler is accepted by both \
                     gates and silently never fires at this client width, which is exactly \
                     the failure class this phase exists to catch"
                );

                ffi::wasamo_window_destroy(window);
            }
        },
    );
}

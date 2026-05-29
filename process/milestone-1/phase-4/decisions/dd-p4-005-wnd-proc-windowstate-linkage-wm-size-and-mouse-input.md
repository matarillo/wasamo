### DD-P4-005 — `wnd_proc` ↔ `WindowState` linkage; WM_SIZE and mouse input

**Status:** Accepted

**Context:**
`wnd_proc` is a `unsafe extern "system" fn` registered at window class
creation. It has no inherent access to Rust state. Two event types require
reaching into Rust state from `wnd_proc`:

1. **WM_SIZE** — trigger re-layout with the new window dimensions.
   (Deferred from Phase 3; `private/CLAUDE.md` lists this as Phase 4 scope.)
2. **WM_LBUTTONDOWN / WM_LBUTTONUP / WM_MOUSEMOVE / WM_MOUSELEAVE** —
   button hit-test and hover state.

**Options:**

Option A — `GWLP_USERDATA` stores `*mut WindowState`; `WindowState` holds event callbacks
- After `WindowState` is constructed, call
  `SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize)`.
  In `wnd_proc`, retrieve it with `GetWindowLongPtrW(hwnd, GWLP_USERDATA)`.
  Add optional callbacks to `WindowState`:
  ```rust
  pub resize_fn:     Option<Box<dyn FnMut(f32, f32)>>,
  pub mouse_down_fn: Option<Box<dyn FnMut(i32, i32)>>,
  pub mouse_move_fn: Option<Box<dyn FnMut(i32, i32)>>,
  pub mouse_leave_fn: Option<Box<dyn FnMut()>>,
  ```
  The host sets these before calling `wasamo_run()`.
  `wnd_proc` dereferences the raw pointer and calls the relevant closure.
- What you gain: `window.rs` stays decoupled from `widget.rs` (callbacks
  are type-erased); standard Win32 idiom; works for both resize and mouse.
- **`unsafe` scope:** The only unsafe operations are
  `SetWindowLongPtrW` (one line in `window::create()`) and
  `GetWindowLongPtrW` + pointer dereference (2–3 lines in `wnd_proc`).
  `wnd_proc` is already `unsafe extern "system"`, and `window.rs` already
  contains extensive Win32 unsafe calls. All callback fields on
  `WindowState` (`Box<dyn FnMut>`) are safe Rust types. The public API
  (`wasamo::window_create`, `wasamo::run`, etc.) gains no new `unsafe`
  annotations; the unsafe surface does not grow beyond `window.rs`.
- What you give up: Raw pointer dereference in `wnd_proc` is `unsafe`;
  callers must ensure `WindowState` outlives the HWND (already required
  by the existing ownership model).

Option B — Thread-local static holds a reference to the widget tree
- `wnd_proc` calls into a thread-local `WIDGET_ROOT` to trigger layout
  or hit-test directly.
- What you gain: No pointer manipulation in `wnd_proc`.
- What you give up: Couples `window.rs` to `widget.rs` through a global;
  harder to extend to multi-window; non-idiomatic for Win32.

Option C — Poll mouse state in the message loop instead of `wnd_proc`
- What you give up: Misses out-of-focus events; not idiomatic; hover
  detection requires `TrackMouseEvent` anyway.

**Decision:** Option A — `GWLP_USERDATA` + callbacks on `WindowState`.
`SetWindowLongPtrW` is called at the end of `window::create()`, after the
`WindowState` `Box` is constructed (pointer is stable from that point).
The unsafe surface is contained entirely within `window.rs`.

**Hover tracking:** `WM_MOUSELEAVE` requires a prior `TrackMouseEvent`
call. `mouse_move_fn` calls `TrackMouseEvent` on first invocation (one-shot
per enter/leave cycle).

**WM_SIZE detail:** The `(width, height)` passed to `resize_fn` are the
new client area dimensions from `LOWORD(lparam)` / `HIWORD(lparam)`,
converted to `f32`. The host closure calls `root.run_layout(w, h)`.

---

//! `wasamo` — Zig bindings for the Wasamo UI framework.
//!
//! # Structure
//!
//! - **Stable-core surface** (this file root): `Runtime`, `Window`,
//!   `Widget`, `Value`, `Connection`, `Error`, `Status`.
//! - **`experimental` namespace** at the bottom of this file: widget
//!   constructors (`text`, `button`, `vstack`, `hstack`) and property-ID
//!   constants. These mirror the `WASAMO_EXPERIMENTAL` layer in `wasamo.h`
//!   and must be expected to break in any M2+ release.
//!
//! # ABI contract
//!
//! - All handles are UI-thread-only. The thread that calls `Runtime.init`
//!   owns the runtime; every other function must run on that thread.
//! - Widget handles are lightweight pointers that remain valid for
//!   property R/W as long as the owning window is alive.
//! - `Runtime.deinit` calls `wasamo_shutdown`.
//! - `Window.deinit` calls `wasamo_window_destroy`.
//!
//! # Scope (DD-P7-004 — Hello-Counter-minimal)
//!
//! Observers (`wasamo_observe_property` / `wasamo_unobserve_property`)
//! and generic signal connect/disconnect are intentionally not wrapped.

const std = @import("std");

// ── Raw C ABI declarations ─────────────────────────────────────────────
//
// Hand-written extern block mirroring bindings/c/wasamo.h.
// `__declspec(dllimport)` / `WASAMO_API` are calling-convention details
// that Zig resolves at link time; we only declare the Zig signatures here.
//
// All functions use the default (C / __cdecl) calling convention, which
// matches WASAMO_API on x64 Windows.

pub const c = struct {
    pub const WasamoStatus = i32;

    pub const WASAMO_OK: WasamoStatus = 0;
    pub const WASAMO_ERR_INVALID_ARG: WasamoStatus = -1;
    pub const WASAMO_ERR_RUNTIME: WasamoStatus = -2;
    pub const WASAMO_ERR_NOT_INITIALIZED: WasamoStatus = -3;
    pub const WASAMO_ERR_WRONG_THREAD: WasamoStatus = -4;

    pub const WasamoWindow = opaque {};
    pub const WasamoWidget = opaque {};

    pub const WasamoValueTag = i32;
    pub const WASAMO_VALUE_NONE: WasamoValueTag = 0;
    pub const WASAMO_VALUE_I32: WasamoValueTag = 1;
    pub const WASAMO_VALUE_F64: WasamoValueTag = 2;
    pub const WASAMO_VALUE_BOOL: WasamoValueTag = 3;
    pub const WASAMO_VALUE_STRING: WasamoValueTag = 4;
    pub const WASAMO_VALUE_WIDGET: WasamoValueTag = 5;

    pub const WasamoStringView = extern struct {
        ptr: [*c]const u8,
        len: usize,
    };

    pub const WasamoValueAs = extern union {
        v_i32: i32,
        v_f64: f64,
        v_bool: i32,
        v_string: WasamoStringView,
        v_widget: ?*WasamoWidget,
    };

    pub const WasamoValue = extern struct {
        tag: WasamoValueTag,
        as: WasamoValueAs,
    };

    pub const WasamoDestroyFn = ?*const fn (user_data: ?*anyopaque) callconv(.c) void;

    pub const WasamoSignalHandlerFn = *const fn (
        sender: ?*WasamoWidget,
        args: [*c]const WasamoValue,
        arg_count: usize,
        user_data: ?*anyopaque,
    ) callconv(.c) void;

    // M1 experimental property-ID constants
    pub const WASAMO_BUTTON_LABEL: u32 = 1;
    pub const WASAMO_BUTTON_STYLE: u32 = 2;
    pub const WASAMO_TEXT_CONTENT: u32 = 3;
    pub const WASAMO_TEXT_STYLE: u32 = 4;

    pub extern fn wasamo_init() WasamoStatus;
    pub extern fn wasamo_shutdown() void;
    pub extern fn wasamo_last_error_message() ?[*:0]const u8;

    pub extern fn wasamo_window_create(
        title_utf8: [*c]const u8,
        title_len: usize,
        width: i32,
        height: i32,
        out: *?*WasamoWindow,
    ) WasamoStatus;
    pub extern fn wasamo_window_show(window: *WasamoWindow) WasamoStatus;
    pub extern fn wasamo_window_destroy(window: *WasamoWindow) WasamoStatus;
    pub extern fn wasamo_run() void;
    pub extern fn wasamo_quit() void;

    pub extern fn wasamo_get_property(
        widget: *WasamoWidget,
        property_id: u32,
        out_value: *WasamoValue,
    ) WasamoStatus;
    pub extern fn wasamo_set_property(
        widget: *WasamoWidget,
        property_id: u32,
        value: *const WasamoValue,
    ) WasamoStatus;

    // ── M1 experimental constructors ───────────────────────────────────
    pub extern fn wasamo_text_create(
        content_utf8: [*c]const u8,
        content_len: usize,
        out: *?*WasamoWidget,
    ) WasamoStatus;
    pub extern fn wasamo_button_create(
        label_utf8: [*c]const u8,
        label_len: usize,
        out: *?*WasamoWidget,
    ) WasamoStatus;
    pub extern fn wasamo_vstack_create(
        children: [*c]?*WasamoWidget,
        count: usize,
        out: *?*WasamoWidget,
    ) WasamoStatus;
    pub extern fn wasamo_hstack_create(
        children: [*c]?*WasamoWidget,
        count: usize,
        out: *?*WasamoWidget,
    ) WasamoStatus;
    pub extern fn wasamo_window_set_root(
        window: *WasamoWindow,
        root: *WasamoWidget,
    ) WasamoStatus;
    pub extern fn wasamo_button_set_clicked(
        button: *WasamoWidget,
        callback: WasamoSignalHandlerFn,
        user_data: ?*anyopaque,
        destroy_fn: WasamoDestroyFn,
        out_token: *u64,
    ) WasamoStatus;
};

// ── Error ──────────────────────────────────────────────────────────────

pub const Error = error{
    InvalidArg,
    Runtime,
    NotInitialized,
    WrongThread,
    UnknownStatus,
};

fn statusToError(status: c.WasamoStatus) Error {
    return switch (status) {
        c.WASAMO_ERR_INVALID_ARG => Error.InvalidArg,
        c.WASAMO_ERR_RUNTIME => Error.Runtime,
        c.WASAMO_ERR_NOT_INITIALIZED => Error.NotInitialized,
        c.WASAMO_ERR_WRONG_THREAD => Error.WrongThread,
        else => Error.UnknownStatus,
    };
}

fn check(status: c.WasamoStatus) Error!void {
    if (status == c.WASAMO_OK) return;
    return statusToError(status);
}

/// Returns the thread-local last-error message, or null if none.
pub fn lastErrorMessage() ?[:0]const u8 {
    const ptr = c.wasamo_last_error_message() orelse return null;
    return std.mem.span(ptr);
}

// ── Value ──────────────────────────────────────────────────────────────

/// A value for property get/set or signal callbacks.
pub const Value = union(enum) {
    none,
    i32: i32,
    f64: f64,
    bool: bool,
    /// Borrowed UTF-8 string slice. The runtime copies internally if needed.
    string: []const u8,
    widget: Widget,
};

fn valueToRaw(v: Value) c.WasamoValue {
    return switch (v) {
        .none => .{ .tag = c.WASAMO_VALUE_NONE, .as = .{ .v_i32 = 0 } },
        .i32 => |x| .{ .tag = c.WASAMO_VALUE_I32, .as = .{ .v_i32 = x } },
        .f64 => |x| .{ .tag = c.WASAMO_VALUE_F64, .as = .{ .v_f64 = x } },
        .bool => |b| .{ .tag = c.WASAMO_VALUE_BOOL, .as = .{ .v_bool = if (b) @as(i32, 1) else 0 } },
        .string => |s| .{
            .tag = c.WASAMO_VALUE_STRING,
            .as = .{ .v_string = .{ .ptr = s.ptr, .len = s.len } },
        },
        .widget => |w| .{ .tag = c.WASAMO_VALUE_WIDGET, .as = .{ .v_widget = w.raw } },
    };
}

fn rawToValue(raw: c.WasamoValue) Value {
    return switch (raw.tag) {
        c.WASAMO_VALUE_I32 => .{ .i32 = raw.as.v_i32 },
        c.WASAMO_VALUE_F64 => .{ .f64 = raw.as.v_f64 },
        c.WASAMO_VALUE_BOOL => .{ .bool = raw.as.v_bool != 0 },
        c.WASAMO_VALUE_STRING => blk: {
            const sv = raw.as.v_string;
            if (sv.ptr == null) break :blk .none;
            break :blk .{ .string = sv.ptr[0..sv.len] };
        },
        c.WASAMO_VALUE_WIDGET => .{ .widget = .{ .raw = raw.as.v_widget } },
        else => .none,
    };
}

// ── Widget ─────────────────────────────────────────────────────────────

/// Lightweight pointer to a runtime-owned widget.
///
/// Handles are valid for property R/W for the lifetime of the window
/// that owns the root widget tree.
pub const Widget = struct {
    raw: ?*c.WasamoWidget,

    pub fn getProperty(self: Widget, property_id: u32) Error!Value {
        var raw_val: c.WasamoValue = .{ .tag = c.WASAMO_VALUE_NONE, .as = .{ .v_i32 = 0 } };
        try check(c.wasamo_get_property(self.raw.?, property_id, &raw_val));
        return rawToValue(raw_val);
    }

    pub fn setProperty(self: Widget, property_id: u32, value: Value) Error!void {
        const raw_val = valueToRaw(value);
        try check(c.wasamo_set_property(self.raw.?, property_id, &raw_val));
    }

    /// Register a click handler on a Button widget. **EXPERIMENTAL.**
    ///
    /// `context` is passed as `user_data` to the callback on each
    /// invocation. The callback runs on the UI thread, never re-entrantly
    /// during a `wasamo_*` call (DD-P6-003 queued-emission guarantee).
    ///
    /// No `destroy_fn` is registered; `context` lifetime is the caller's
    /// responsibility. Pass `null` for `context` if unused.
    pub fn onClicked(
        self: Widget,
        callback: c.WasamoSignalHandlerFn,
        context: ?*anyopaque,
    ) Error!Connection {
        var token: u64 = 0;
        try check(c.wasamo_button_set_clicked(
            self.raw.?,
            callback,
            context,
            null,
            &token,
        ));
        return .{ .token = token };
    }
};

// ── Connection ─────────────────────────────────────────────────────────

/// Opaque token identifying a signal or observer connection.
///
/// In M1 Hello Counter the clicked handler lives for the app's lifetime;
/// explicit disconnect will be added in a later phase.
pub const Connection = struct {
    token: u64,
};

// ── Runtime ────────────────────────────────────────────────────────────

/// RAII guard for the Wasamo runtime.
///
/// Call `Runtime.init` once on the UI thread. Call `deinit` when done.
pub const Runtime = struct {
    pub fn init() Error!Runtime {
        try check(c.wasamo_init());
        return .{};
    }

    pub fn deinit(_: *Runtime) void {
        c.wasamo_shutdown();
    }

    /// Enter the event loop. Blocks until `quit` is called.
    pub fn run(_: *const Runtime) void {
        c.wasamo_run();
    }

    /// Signal the event loop to exit.
    pub fn quit(_: *const Runtime) void {
        c.wasamo_quit();
    }
};

// ── Window ─────────────────────────────────────────────────────────────

/// RAII handle to a Wasamo window.
pub const Window = struct {
    raw: *c.WasamoWindow,

    pub fn create(title: []const u8, width: i32, height: i32) Error!Window {
        var raw: ?*c.WasamoWindow = null;
        try check(c.wasamo_window_create(title.ptr, title.len, width, height, &raw));
        return .{ .raw = raw.? };
    }

    pub fn deinit(self: *Window) void {
        _ = c.wasamo_window_destroy(self.raw);
    }

    pub fn show(self: *const Window) Error!void {
        try check(c.wasamo_window_show(self.raw));
    }

    /// Attach a widget tree as the window's root. **EXPERIMENTAL.**
    ///
    /// The runtime takes ownership of the widget tree. Widget handles
    /// remain valid for property R/W.
    pub fn setRoot(self: *const Window, widget: Widget) Error!void {
        try check(c.wasamo_window_set_root(self.raw, widget.raw.?));
    }
};

// ── experimental namespace ─────────────────────────────────────────────
//
// All symbols below mirror `WASAMO_EXPERIMENTAL`-marked entries in
// `wasamo.h`. Expect breakage in any M2+ release.

pub const experimental = struct {
    pub const BUTTON_LABEL = c.WASAMO_BUTTON_LABEL;
    pub const BUTTON_STYLE = c.WASAMO_BUTTON_STYLE;
    pub const TEXT_CONTENT = c.WASAMO_TEXT_CONTENT;
    pub const TEXT_STYLE = c.WASAMO_TEXT_STYLE;

    pub fn text(content: []const u8) Error!Widget {
        var raw: ?*c.WasamoWidget = null;
        try check(c.wasamo_text_create(content.ptr, content.len, &raw));
        return .{ .raw = raw };
    }

    pub fn button(label: []const u8) Error!Widget {
        var raw: ?*c.WasamoWidget = null;
        try check(c.wasamo_button_create(label.ptr, label.len, &raw));
        return .{ .raw = raw };
    }

    /// Create a vertical stack. Children are consumed: the runtime takes
    /// ownership of the underlying allocations. Widget handles remain
    /// valid for property R/W after this call.
    pub fn vstack(children: []Widget) Error!Widget {
        var raw_children = std.BoundedArray(?*c.WasamoWidget, 64).init(0) catch unreachable;
        for (children) |ch| raw_children.append(ch.raw) catch unreachable;
        var raw: ?*c.WasamoWidget = null;
        try check(c.wasamo_vstack_create(
            raw_children.slice().ptr,
            raw_children.len,
            &raw,
        ));
        return .{ .raw = raw };
    }

    /// Create a horizontal stack. Same ownership semantics as `vstack`.
    pub fn hstack(children: []Widget) Error!Widget {
        var raw_children = std.BoundedArray(?*c.WasamoWidget, 64).init(0) catch unreachable;
        for (children) |ch| raw_children.append(ch.raw) catch unreachable;
        var raw: ?*c.WasamoWidget = null;
        try check(c.wasamo_hstack_create(
            raw_children.slice().ptr,
            raw_children.len,
            &raw,
        ));
        return .{ .raw = raw };
    }
};

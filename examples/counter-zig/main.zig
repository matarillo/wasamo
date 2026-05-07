//! counter-zig/main.zig — Hello Counter, M2 declarative shape (DD-M2-P6-008).
//!
//! All UI structure (widget tree, state, binding, click handler) lives in
//! examples/counter/counter.ui. build.zig invokes wasamoc to compile that
//! file to IR text; the resulting counter.uic is exposed to this source
//! as the anonymous import "counter_uic" (see build.zig). @embedFile
//! inlines the IR bytes at compile time. main hands the (pointer, length)
//! blob to wasamo_load_ui via WASAMO_LOAD_MEMORY. The host contains zero
//! wasamo_set_property calls — A2 is structurally enforced.

const std = @import("std");
const wasamo = @import("wasamo");

const counter_uic: []const u8 = @embedFile("counter_uic");

pub fn main() !void {
    if (wasamo.c.wasamo_init() != wasamo.c.WASAMO_OK) {
        try printLastError("wasamo_init");
        return error.InitFailed;
    }
    defer wasamo.c.wasamo_shutdown();

    var window: ?*wasamo.c.WasamoWindow = null;
    const status = wasamo.c.wasamo_load_ui(
        wasamo.c.WASAMO_LOAD_MEMORY,
        counter_uic.ptr,
        counter_uic.len,
        &window,
    );
    if (status != wasamo.c.WASAMO_OK) {
        try printLastError("wasamo_load_ui");
        return error.LoadUiFailed;
    }

    const w = window orelse return error.LoadUiReturnedNull;
    if (wasamo.c.wasamo_window_show(w) != wasamo.c.WASAMO_OK) {
        try printLastError("wasamo_window_show");
        return error.WindowShowFailed;
    }

    wasamo.c.wasamo_run();
}

fn printLastError(prefix: []const u8) !void {
    if (wasamo.lastErrorMessage()) |msg| {
        std.debug.print("{s} failed: {s}\n", .{ prefix, msg });
    } else {
        std.debug.print("{s} failed (no error message)\n", .{prefix});
    }
}

//! gallery-zig/main.zig - Photo Gallery host.
//!
//! The UI lives in examples/gallery/gallery.ui. build.zig invokes wasamoc
//! to compile that file to IR text; the resulting gallery.uic is exposed to
//! this source as the anonymous import "gallery_uic". main hands the
//! embedded blob to wasamo_load_ui via WASAMO_LOAD_MEMORY.

const std = @import("std");
const wasamo = @import("wasamo");

const gallery_uic: []const u8 = @embedFile("gallery_uic");

pub fn main() !void {
    if (wasamo.c.wasamo_init() != wasamo.c.WASAMO_OK) {
        try printLastError("wasamo_init");
        return error.InitFailed;
    }
    defer wasamo.c.wasamo_shutdown();

    var window: ?*wasamo.c.WasamoWindow = null;
    const status = wasamo.c.wasamo_load_ui(
        wasamo.c.WASAMO_LOAD_MEMORY,
        gallery_uic.ptr,
        gallery_uic.len,
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

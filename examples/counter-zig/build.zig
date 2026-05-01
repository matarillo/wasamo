const std = @import("std");

// Build script for the Hello Counter Zig example.
//
// CI usage (from repo root):
//   cargo build --release --workspace
//   zig build --prefix-exe-dir . \
//       --wasamo-lib ../../target/release/wasamo.dll.lib \
//       --wasamo-zig ../../bindings/zig/wasamo.zig
//
// Local usage:
//   cargo build --workspace
//   zig build --wasamo-lib ../../target/debug/wasamo.dll.lib \
//              --wasamo-zig ../../bindings/zig/wasamo.zig

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const wasamo_lib_path = b.option(
        []const u8,
        "wasamo-lib",
        "Path to wasamo.dll.lib (default: ../../target/release/wasamo.dll.lib)",
    ) orelse "../../target/release/wasamo.dll.lib";

    const wasamo_zig_path = b.option(
        []const u8,
        "wasamo-zig",
        "Path to bindings/zig/wasamo.zig (default: ../../bindings/zig/wasamo.zig)",
    ) orelse "../../bindings/zig/wasamo.zig";

    // ── wasamo module ──────────────────────────────────────────────────
    const wasamo_mod = b.addModule("wasamo", .{
        .root_source_file = b.path(wasamo_zig_path),
        .target = target,
        .optimize = optimize,
    });
    wasamo_mod.addObjectFile(.{ .cwd_relative = wasamo_lib_path });

    // ── counter executable ─────────────────────────────────────────────
    const exe = b.addExecutable(.{
        .name = "counter-zig",
        .root_module = b.createModule(.{
            .root_source_file = b.path("main.zig"),
            .target = target,
            .optimize = optimize,
            .imports = &.{
                .{ .name = "wasamo", .module = wasamo_mod },
            },
        }),
    });

    b.installArtifact(exe);

    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());
    const run_step = b.step("run", "Run the counter example");
    run_step.dependOn(&run_cmd.step);
}

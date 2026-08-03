const std = @import("std");

// Build script for the Hello Counter Zig example (M2 declarative shape,
// DD-M2-P6-008).
//
// CI usage (from repo root):
//   cargo build --release -p wasamo-runtime
//   cargo build --release --workspace
//   zig build -Dwasamo-lib=../../target/release/wasamo.dll.lib \
//             -Dwasamo-zig=../../bindings/zig/wasamo.zig \
//             -Dwasamoc=../../target/release/wasamoc.exe
//
// Local usage:
//   cargo build -p wasamo-runtime
//   cargo build --workspace
//   zig build -Dwasamo-lib=../../target/debug/wasamo.dll.lib \
//             -Dwasamo-zig=../../bindings/zig/wasamo.zig \
//             -Dwasamoc=../../target/debug/wasamoc.exe
//
// `wasamoc` must have been built beforehand — see CLAUDE.md
// "Build ordering requirements".

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

    const wasamoc_path = b.option(
        []const u8,
        "wasamoc",
        "Path to wasamoc.exe (default: ../../target/release/wasamoc.exe)",
    ) orelse "../../target/release/wasamoc.exe";

    const counter_ui_path = b.option(
        []const u8,
        "counter-ui",
        "Path to examples/counter/counter.ui (default: ../counter/counter.ui)",
    ) orelse "../counter/counter.ui";

    // ── DSL build pipeline (DD-M2-P6-008) ──────────────────────────────
    //
    // wasamoc compiles counter.ui to counter.uic; the resulting LazyPath
    // is exposed to main.zig as an anonymous import, which @embedFile
    // reads at compile time.
    const wasamoc_run = b.addSystemCommand(&.{wasamoc_path});
    wasamoc_run.addArg("build");
    wasamoc_run.addFileArg(b.path(counter_ui_path));
    const counter_uic = wasamoc_run.addOutputFileArg("counter.uic");

    // ── wasamo module ──────────────────────────────────────────────────
    const wasamo_mod = b.addModule("wasamo", .{
        .root_source_file = b.path(wasamo_zig_path),
        .target = target,
        .optimize = optimize,
    });
    wasamo_mod.addObjectFile(.{ .cwd_relative = wasamo_lib_path });

    // ── counter executable ─────────────────────────────────────────────
    const exe_module = b.createModule(.{
        .root_source_file = b.path("main.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{
            .{ .name = "wasamo", .module = wasamo_mod },
        },
    });
    exe_module.addAnonymousImport("counter_uic", .{
        .root_source_file = counter_uic,
    });

    const exe = b.addExecutable(.{
        .name = "counter-zig",
        .root_module = exe_module,
    });

    b.installArtifact(exe);

    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());
    const run_step = b.step("run", "Run the counter example");
    run_step.dependOn(&run_cmd.step);
}

const std = @import("std");

// Build script for the Photo Gallery Zig example.
//
// CI usage (from repo root):
//   cargo build --release --workspace
//   zig build -Dwasamo-lib=../../target/release/wasamo.dll.lib \
//             -Dwasamo-zig=../../bindings/zig/wasamo.zig \
//             -Dwasamoc=../../target/release/wasamoc.exe \
//             -Doptimize=ReleaseSafe

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

    const gallery_ui_path = b.option(
        []const u8,
        "gallery-ui",
        "Path to examples/gallery/gallery.ui (default: ../gallery/gallery.ui)",
    ) orelse "../gallery/gallery.ui";

    const wasamoc_run = b.addSystemCommand(&.{wasamoc_path});
    wasamoc_run.addArg("build");
    wasamoc_run.addFileArg(b.path(gallery_ui_path));
    const gallery_uic = wasamoc_run.addOutputFileArg("gallery.uic");

    const wasamo_mod = b.addModule("wasamo", .{
        .root_source_file = b.path(wasamo_zig_path),
        .target = target,
        .optimize = optimize,
    });
    wasamo_mod.addObjectFile(.{ .cwd_relative = wasamo_lib_path });

    const exe_module = b.createModule(.{
        .root_source_file = b.path("main.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{
            .{ .name = "wasamo", .module = wasamo_mod },
        },
    });
    exe_module.addAnonymousImport("gallery_uic", .{
        .root_source_file = gallery_uic,
    });

    const exe = b.addExecutable(.{
        .name = "gallery-zig",
        .root_module = exe_module,
    });

    b.installArtifact(exe);

    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());
    const run_step = b.step("run", "Run the Gallery example");
    run_step.dependOn(&run_cmd.step);
}

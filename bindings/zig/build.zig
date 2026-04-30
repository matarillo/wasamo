const std = @import("std");

// Build script for the Wasamo Zig binding.
//
// Exposes one module (`wasamo`) and one test step (`zig build test`).
//
// The binding links against `wasamo.dll.lib` (the import library emitted by
// the `wasamo-runtime` Rust crate). The import library is expected at the
// path given by the `--wasamo-lib` option (default: the workspace release
// target dir relative to this file's location).
//
// CI usage:
//   cargo build --release --workspace          # emits target/release/wasamo.dll.lib
//   zig build test --wasamo-lib ../../target/release/wasamo.dll.lib
//
// Local usage (debug DLL):
//   cargo build --workspace
//   zig build test --wasamo-lib ../../target/debug/wasamo.dll.lib

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Allow the caller to override where wasamo.dll.lib lives.
    // Default: ../../target/release/wasamo.dll.lib (repo-relative from bindings/zig/).
    const wasamo_lib_path = b.option(
        []const u8,
        "wasamo-lib",
        "Path to wasamo.dll.lib (default: ../../target/release/wasamo.dll.lib)",
    ) orelse "../../target/release/wasamo.dll.lib";

    // ── wasamo module ──────────────────────────────────────────────────
    const mod = b.addModule("wasamo", .{
        .root_source_file = b.path("wasamo.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Link the import library. `addObjectFile` passes the .lib directly to
    // the linker without any name mangling (analogous to rustc's
    // `dylib:+verbatim=wasamo.dll.lib`).
    mod.addObjectFile(.{ .cwd_relative = wasamo_lib_path });

    // ── smoke test ─────────────────────────────────────────────────────
    //
    // Compiles and links smoke_test.zig against wasamo.dll.lib, resolving
    // every extern symbol declared in wasamo.zig. The resulting executable
    // is never run: wasamo.dll must be on PATH at runtime, but the CI test
    // only needs to verify that all symbols link (matching wasamo.h).
    //
    // `zig build test` compiles and links only; it does not execute the binary.
    const smoke = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("smoke_test.zig"),
            .target = target,
            .optimize = optimize,
            .imports = &.{
                .{ .name = "wasamo", .module = mod },
            },
        }),
    });

    // Depend on the compile step only, not on the run step, so that the
    // test passes even when wasamo.dll is not on PATH.
    const test_step = b.step("test", "Link-resolve all wasamo symbols");
    test_step.dependOn(&smoke.step);
}

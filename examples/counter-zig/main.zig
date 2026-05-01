//! counter-zig/main.zig — Hello Counter example in Zig (M1 host-imperative shape)
//!
//! This program constructs the same widget tree as examples/counter/counter.ui
//! imperatively through the wasamo Zig binding over the experimental C ABI:
//!
//!   VStack {
//!     Text  { "Count: 0"  font: title }
//!     Button { "Increment" style: accent }
//!   }
//!
//! See examples/counter/counter.ui for the future M2 declarative form.
//! The .ui -> runtime lowering (wasamoc codegen) is M2 scope; M1 verifies
//! that the C ABI and Visual Layer work correctly.

const std = @import("std");
const wasamo = @import("wasamo");

// ── Counter state ──────────────────────────────────────────────────────────────

const CounterState = struct {
    label: wasamo.Widget,
    count: i32,
};

var g_state: CounterState = undefined;

// ── Button click callback ──────────────────────────────────────────────────────

fn onIncrement(
    sender: ?*wasamo.c.WasamoWidget,
    args: [*c]const wasamo.c.WasamoValue,
    arg_count: usize,
    user_data: ?*anyopaque,
) callconv(.c) void {
    _ = sender;
    _ = args;
    _ = arg_count;
    _ = user_data;

    g_state.count += 1;

    var buf: [32]u8 = undefined;
    const text = std.fmt.bufPrint(&buf, "Count: {}", .{g_state.count}) catch return;

    g_state.label.setProperty(
        wasamo.experimental.TEXT_CONTENT,
        .{ .string = text },
    ) catch {};
}

// ── Entry point ────────────────────────────────────────────────────────────────

pub fn main() !void {
    // 1. Initialize the runtime.
    var rt = try wasamo.Runtime.init();
    defer rt.deinit();

    // 2. Create a window (800 x 600).
    var window = try wasamo.Window.create("Counter", 800, 600);
    defer window.deinit();

    // 3. Build the widget tree (bottom-up, matching counter.ui).

    // Text: "Count: 0" with title typography (TypographyStyle::Title = 3).
    const label = try wasamo.experimental.text("Count: 0");
    try label.setProperty(wasamo.experimental.TEXT_STYLE, .{ .i32 = 3 });

    // Button: "Increment" with accent style (ButtonStyle::Accent = 1).
    const btn = try wasamo.experimental.button("Increment");
    try btn.setProperty(wasamo.experimental.BUTTON_STYLE, .{ .i32 = 1 });

    // Store label in global state for the callback to update.
    g_state = .{ .label = label, .count = 0 };

    // Connect the click handler before handing children to the stack.
    _ = try btn.onClicked(onIncrement, null);

    // VStack: label + button.
    var children = [_]wasamo.Widget{ label, btn };
    const root = try wasamo.experimental.vstack(&children);

    // 4. Install the root widget and show the window.
    try window.setRoot(root);
    try window.show();

    // 5. Run the message loop (blocks until the window is closed).
    rt.run();
}

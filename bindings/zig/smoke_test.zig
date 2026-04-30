// smoke_test.zig — Phase 7 Zig binding link-resolution smoke test.
//
// Takes the address of every extern function declared in wasamo.zig to
// force the linker to resolve them against wasamo.dll.lib. We never call
// any of them: doing so would require wasamo_init on a UI thread with a
// message loop, which the test harness does not provide.
//
// Analogous to bindings/rust-sys/src/lib.rs `link_smoke::symbols_resolve`.

const wasamo = @import("wasamo");
const c = wasamo.c;

test "all extern symbols resolve" {
    const ptrs = [_]*const anyopaque{
        @ptrCast(&c.wasamo_init),
        @ptrCast(&c.wasamo_shutdown),
        @ptrCast(&c.wasamo_last_error_message),
        @ptrCast(&c.wasamo_window_create),
        @ptrCast(&c.wasamo_window_show),
        @ptrCast(&c.wasamo_window_destroy),
        @ptrCast(&c.wasamo_run),
        @ptrCast(&c.wasamo_quit),
        @ptrCast(&c.wasamo_get_property),
        @ptrCast(&c.wasamo_set_property),
        // M1 experimental layer
        @ptrCast(&c.wasamo_text_create),
        @ptrCast(&c.wasamo_button_create),
        @ptrCast(&c.wasamo_vstack_create),
        @ptrCast(&c.wasamo_hstack_create),
        @ptrCast(&c.wasamo_window_set_root),
        @ptrCast(&c.wasamo_button_set_clicked),
    };
    // Prevent the compiler from optimizing out the array.
    _ = &ptrs;
}

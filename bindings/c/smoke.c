/*
 * smoke.c — Phase 6 CI smoke test.
 *
 * Verifies two things:
 *   1. wasamo.h compiles cleanly under MSVC and Clang.
 *   2. Every stable-core ABI function declared in the header resolves
 *      against wasamo.dll.lib at link time. Drift between wasamo.h and
 *      the Rust extern "C" surface produces a linker error here.
 *
 * The program is never executed; building it is the test.
 */

#include "wasamo.h"

int main(void) {
    /* Force the linker to resolve every public symbol. Taking the address
     * is enough; we never call any of these in the smoke test. */
    void* fns[] = {
        (void*)&wasamo_init,
        (void*)&wasamo_shutdown,
        (void*)&wasamo_last_error_message,
        (void*)&wasamo_window_create,
        (void*)&wasamo_window_show,
        (void*)&wasamo_window_destroy,
        (void*)&wasamo_run,
        (void*)&wasamo_quit,
        (void*)&wasamo_get_property,
        (void*)&wasamo_set_property,
        (void*)&wasamo_observe_property,
        (void*)&wasamo_unobserve_property,
        (void*)&wasamo_signal_connect,
        (void*)&wasamo_signal_disconnect,
    };
    return (int)(sizeof(fns) / sizeof(fns[0]));
}

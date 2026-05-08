/*
 * counter-c/main.c — Hello Counter, M2 declarative shape (DD-M2-P6-008).
 *
 * All UI structure (widget tree, state, binding, click handler) lives in
 * examples/counter/counter.ui. CMake invokes wasamoc to compile that file
 * to IR text, then generates counter_uic.h embedding the IR bytes as a
 * static array. This binary just hands the (pointer, length) blob to
 * wasamo_load_ui (WASAMO_LOAD_MEMORY) and runs the message loop. The host
 * contains zero wasamo_set_property calls — A2 is structurally enforced.
 */

#include <stdio.h>
#include "../../bindings/c/wasamo.h"
#include "counter_uic.h"

int main(void)
{
    if (wasamo_init() != WASAMO_OK) {
        fprintf(stderr, "wasamo_init failed: %s\n", wasamo_last_error_message());
        return 1;
    }

    WasamoWindow* window = NULL;
    WasamoStatus s = wasamo_load_ui(
        WASAMO_LOAD_MEMORY,
        COUNTER_UIC,
        COUNTER_UIC_LEN,
        &window);
    if (s != WASAMO_OK) {
        fprintf(stderr, "wasamo_load_ui failed: %s\n", wasamo_last_error_message());
        wasamo_shutdown();
        return 1;
    }

    if (wasamo_window_show(window) != WASAMO_OK) {
        fprintf(stderr, "wasamo_window_show failed: %s\n", wasamo_last_error_message());
        wasamo_shutdown();
        return 1;
    }

    wasamo_run();
    wasamo_shutdown();
    return 0;
}

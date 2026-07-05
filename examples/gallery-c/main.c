/*
 * gallery-c/main.c - Photo Gallery host.
 *
 * The UI lives in examples/gallery/gallery.ui. CMake invokes wasamoc to
 * compile that file to IR text, then generates gallery_uic.h embedding the
 * IR bytes as a static array. This binary only loads that blob via
 * WASAMO_LOAD_MEMORY and runs the message loop.
 */

#include <stdio.h>
#include "../../bindings/c/wasamo.h"
#include "gallery_uic.h"

int main(void)
{
    if (wasamo_init() != WASAMO_OK) {
        fprintf(stderr, "wasamo_init failed: %s\n", wasamo_last_error_message());
        return 1;
    }

    WasamoWindow* window = NULL;
    WasamoStatus s = wasamo_load_ui(
        WASAMO_LOAD_MEMORY,
        GALLERY_UIC,
        GALLERY_UIC_LEN,
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

/*
 * counter-c/main.c — Hello Counter example in C (M1 host-imperative shape)
 *
 * This program constructs the same widget tree as examples/counter/counter.ui
 * imperatively through the wasamo C ABI experimental layer:
 *
 *   VStack {
 *     Text  { "Count: 0"  font: title }
 *     Button { "Increment" style: accent }
 *   }
 *
 * See examples/counter/counter.ui for the future M2 declarative form.
 * The .ui → runtime lowering (wasamoc codegen) is M2 scope; M1 verifies
 * that the C ABI and Visual Layer work correctly.
 */

#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include "../../bindings/c/wasamo.h"

/* ── Counter state ─────────────────────────────────────────────────────────── */

typedef struct {
    WasamoWidget* label;   /* Text widget showing "Count: N" */
    int           count;
} CounterState;

static CounterState g_state;

/* ── Button click handler ──────────────────────────────────────────────────── */

static void __cdecl on_increment(
    WasamoWidget*       sender,
    const WasamoValue*  args,
    size_t              arg_count,
    void*               user_data)
{
    (void)sender; (void)args; (void)arg_count; (void)user_data;

    g_state.count++;

    char buf[32];
    int len = snprintf(buf, sizeof(buf), "Count: %d", g_state.count);

    WasamoValue v;
    v.tag = WASAMO_VALUE_STRING;
    v.as.v_string.ptr = buf;
    v.as.v_string.len = (size_t)len;
    wasamo_set_property(g_state.label, WASAMO_TEXT_CONTENT, &v);
}

/* ── Entry point ───────────────────────────────────────────────────────────── */

int main(void)
{
    /* 1. Initialize the runtime. */
    if (wasamo_init() != WASAMO_OK) {
        fprintf(stderr, "wasamo_init failed: %s\n", wasamo_last_error_message());
        return 1;
    }

    /* 2. Create a window (800 × 600). */
    WasamoWindow* window = NULL;
    {
        const char* title = "Counter";
        if (wasamo_window_create(title, strlen(title), 800, 600, &window) != WASAMO_OK) {
            fprintf(stderr, "wasamo_window_create failed: %s\n", wasamo_last_error_message());
            wasamo_shutdown();
            return 1;
        }
    }

    /* 3. Build the widget tree (bottom-up, matching counter.ui). */

    /* Text: "Count: 0" with title typography */
    WasamoWidget* label = NULL;
    {
        const char* initial = "Count: 0";
        if (wasamo_text_create(initial, strlen(initial), &label) != WASAMO_OK) {
            fprintf(stderr, "wasamo_text_create failed: %s\n", wasamo_last_error_message());
            wasamo_window_destroy(window);
            wasamo_shutdown();
            return 1;
        }
        /* TypographyStyle::Title = 3 (abi_spec §5, widget.rs) */
        WasamoValue style_v;
        style_v.tag = WASAMO_VALUE_I32;
        style_v.as.v_i32 = 3;
        wasamo_set_property(label, WASAMO_TEXT_STYLE, &style_v);
    }
    g_state.label = label;
    g_state.count = 0;

    /* Button: "Increment" with accent style */
    WasamoWidget* button = NULL;
    {
        const char* btn_label = "Increment";
        if (wasamo_button_create(btn_label, strlen(btn_label), &button) != WASAMO_OK) {
            fprintf(stderr, "wasamo_button_create failed: %s\n", wasamo_last_error_message());
            wasamo_window_destroy(window);
            wasamo_shutdown();
            return 1;
        }
        /* ButtonStyle::Accent = 1 (abi_spec §5, widget.rs) */
        WasamoValue style_v;
        style_v.tag = WASAMO_VALUE_I32;
        style_v.as.v_i32 = 1;
        wasamo_set_property(button, WASAMO_BUTTON_STYLE, &style_v);
    }

    /* Connect the click handler before handing children to the stack. */
    uint64_t click_token = 0;
    wasamo_button_set_clicked(button, on_increment, NULL, NULL, &click_token);

    /* VStack: label + button */
    WasamoWidget* children[2] = { label, button };
    WasamoWidget* vstack = NULL;
    if (wasamo_vstack_create(children, 2, &vstack) != WASAMO_OK) {
        fprintf(stderr, "wasamo_vstack_create failed: %s\n", wasamo_last_error_message());
        wasamo_window_destroy(window);
        wasamo_shutdown();
        return 1;
    }

    /* 4. Install the root widget and show the window. */
    if (wasamo_window_set_root(window, vstack) != WASAMO_OK) {
        fprintf(stderr, "wasamo_window_set_root failed: %s\n", wasamo_last_error_message());
        wasamo_window_destroy(window);
        wasamo_shutdown();
        return 1;
    }
    wasamo_window_show(window);

    /* 5. Run the message loop (blocks until the window is closed). */
    wasamo_run();

    /* 6. Cleanup. */
    wasamo_window_destroy(window);
    wasamo_shutdown();
    return 0;
}

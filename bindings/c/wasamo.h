/*
 * wasamo.h — Wasamo C ABI
 *
 * Canonical specification: ../../docs/abi_spec.md
 * Authoritative decisions: ../../docs/decisions/phase-6-c-abi.md
 *
 * This header defines a two-layer C ABI:
 *   - Stable core: candidate for the M4 ABI freeze.
 *   - M1 experimental: marked WASAMO_EXPERIMENTAL, not subject to M4 stability.
 *
 * UTF-8 is the only string encoding accepted or returned. All public
 * functions and host-supplied callback typedefs use __cdecl (WASAMO_API).
 * Strict UI-thread affinity: the thread that calls wasamo_init owns the
 * runtime; all other functions and callbacks run on that thread.
 */

#ifndef WASAMO_H
#define WASAMO_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── 2.1 Symbol export ─────────────────────────────────────────────────── */

#if defined(WASAMO_BUILDING_DLL)
#  define WASAMO_EXPORT __declspec(dllexport)
#else
#  define WASAMO_EXPORT __declspec(dllimport)
#endif

/* ── 2.2 Calling convention ────────────────────────────────────────────── */

#define WASAMO_API __cdecl

/* ── M1 experimental marker ────────────────────────────────────────────── */

/*
 * Documentation marker: code using any WASAMO_EXPERIMENTAL-annotated
 * symbol must expect breakage in any M2+ release.
 */
#define WASAMO_EXPERIMENTAL

/* ── 3.1 WasamoStatus ──────────────────────────────────────────────────── */

typedef int32_t WasamoStatus;

#define WASAMO_OK                   0
#define WASAMO_ERR_INVALID_ARG     -1
#define WASAMO_ERR_RUNTIME         -2
#define WASAMO_ERR_NOT_INITIALIZED -3
#define WASAMO_ERR_WRONG_THREAD    -4

/* ── 3.2 Opaque handles ────────────────────────────────────────────────── */

typedef struct WasamoWindow WasamoWindow;
typedef struct WasamoWidget WasamoWidget;

/* ── 3.3 WasamoValue (tagged union) ────────────────────────────────────── */

typedef int32_t WasamoValueTag;

#define WASAMO_VALUE_NONE   0
#define WASAMO_VALUE_I32    1
#define WASAMO_VALUE_F64    2
#define WASAMO_VALUE_BOOL   3
#define WASAMO_VALUE_STRING 4
#define WASAMO_VALUE_WIDGET 5

typedef struct {
    const char* ptr;   /* UTF-8, not necessarily NUL-terminated */
    size_t      len;
} WasamoStringView;

typedef struct {
    WasamoValueTag tag;
    union {
        int32_t          v_i32;
        double           v_f64;
        int32_t          v_bool;   /* 0 = false, non-zero = true */
        WasamoStringView v_string;
        WasamoWidget*    v_widget;
    } as;
} WasamoValue;

/* ── 3.4 Callback typedefs ─────────────────────────────────────────────── */

typedef void (WASAMO_API *WasamoDestroyFn)(void* user_data);

typedef void (WASAMO_API *WasamoSignalHandlerFn)(
    WasamoWidget*       sender,
    const WasamoValue*  args,
    size_t              arg_count,
    void*               user_data);

typedef void (WASAMO_API *WasamoPropertyObserverFn)(
    WasamoWidget*       widget,
    uint32_t            property_id,
    const WasamoValue*  new_value,
    void*               user_data);

/* ── 4.1 Runtime lifecycle ─────────────────────────────────────────────── */

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_init(void);
WASAMO_EXPORT void         WASAMO_API wasamo_shutdown(void);

/*
 * Returns a thread-local NUL-terminated UTF-8 string describing the most
 * recent non-OK status produced on the calling thread. The pointer is
 * valid until the next ABI call on that thread. May return NULL or "" if
 * no error has been produced; hosts must tolerate both.
 */
WASAMO_EXPORT const char* WASAMO_API wasamo_last_error_message(void);

/* ── 4.2 Window and event loop ─────────────────────────────────────────── */

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_window_create(
    const char*    title_utf8,
    size_t         title_len,
    int32_t        width,
    int32_t        height,
    WasamoWindow** out);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_window_show(WasamoWindow* window);
WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_window_destroy(WasamoWindow* window);

WASAMO_EXPORT void WASAMO_API wasamo_run(void);
WASAMO_EXPORT void WASAMO_API wasamo_quit(void);

/* ── 4.3 Property get/set ──────────────────────────────────────────────── */

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_get_property(
    WasamoWidget*  widget,
    uint32_t       property_id,
    WasamoValue*   out_value);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_set_property(
    WasamoWidget*       widget,
    uint32_t            property_id,
    const WasamoValue*  value);

/* ── 4.4 Property-change observers ─────────────────────────────────────── */

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_observe_property(
    WasamoWidget*             widget,
    uint32_t                  property_id,
    WasamoPropertyObserverFn  callback,
    void*                     user_data,
    WasamoDestroyFn           destroy_fn,
    uint64_t*                 out_token);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_unobserve_property(uint64_t token);

/* ── 4.5 Component-declared signal register ────────────────────────────── */

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_signal_connect(
    WasamoWidget*          widget,
    const char*            signal_name_utf8,
    size_t                 name_len,
    WasamoSignalHandlerFn  callback,
    void*                  user_data,
    WasamoDestroyFn        destroy_fn,
    uint64_t*              out_token);

WASAMO_EXPORT WasamoStatus WASAMO_API wasamo_signal_disconnect(uint64_t token);

/* ── 5. M1 experimental layer ──────────────────────────────────────────── */
/*
 * Every symbol below is WASAMO_EXPERIMENTAL. Hosts that use these must
 * expect breakage in any M2+ release.
 *
 * The exact set is finalised during Phase 6 implementation; this section
 * grows as Hello Counter (Phase 8) requirements concretise.
 */

/* Property-ID constants for M1 widgets. WASAMO_EXPERIMENTAL. */
#define WASAMO_BUTTON_LABEL  1u
#define WASAMO_BUTTON_STYLE  2u
#define WASAMO_TEXT_CONTENT  3u
#define WASAMO_TEXT_STYLE    4u

/* Imperative widget builders. WASAMO_EXPERIMENTAL.
 * Finalised during Phase 6 implementation; declared here when added:
 *   wasamo_vstack_create / wasamo_hstack_create /
 *   wasamo_text_create / wasamo_button_create /
 *   wasamo_container_append_child /
 *   wasamo_window_set_root /
 *   wasamo_button_set_clicked
 */

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* WASAMO_H */

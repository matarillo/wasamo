cmake_minimum_required(VERSION 3.21)
project(wasamo_counter_c C)

# ── Usage ──────────────────────────────────────────────────────────────
#
# Building from the Wasamo repo root (CI usage):
#
#   cmake -S examples/counter-c -B build/counter-c \
#         -DWASAMO_ROOT=<path/to/wasamo/repo/root>
#   cmake --build build/counter-c --config Release
#
# WASAMO_ROOT must contain:
#   bindings/c/wasamo.h
#   target/release/wasamo.dll.lib
#   target/release/wasamoc.exe        (DD-M2-P6-008: wasamoc compiles
#                                      counter.ui to IR at build time)
#   examples/counter/counter.ui
#
# Run `cargo build --release --workspace` from the repo root before invoking
# CMake — see CLAUDE.md "Build ordering requirements".

# ── Locate wasamo ──────────────────────────────────────────────────────

if(NOT DEFINED WASAMO_ROOT)
    # Default: two levels up from this file (examples/counter-c/ -> repo root)
    get_filename_component(WASAMO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
endif()

set(WASAMO_INCLUDE_DIR "${WASAMO_ROOT}/bindings/c")
set(WASAMO_IMPORT_LIB  "${WASAMO_ROOT}/target/release/wasamo.dll.lib")
set(WASAMOC_EXE        "${WASAMO_ROOT}/target/release/wasamoc.exe")
set(COUNTER_UI         "${WASAMO_ROOT}/examples/counter/counter.ui")

if(NOT EXISTS "${WASAMO_IMPORT_LIB}")
    message(FATAL_ERROR
        "wasamo.dll.lib not found at ${WASAMO_IMPORT_LIB}.\n"
        "Run 'cargo build --release --workspace' from the Wasamo repo root first,\n"
        "or pass -DWASAMO_ROOT=<path> pointing to a pre-built tree.")
endif()

if(NOT EXISTS "${WASAMOC_EXE}")
    message(FATAL_ERROR
        "wasamoc.exe not found at ${WASAMOC_EXE}.\n"
        "Run 'cargo build --release -p wasamoc' from the Wasamo repo root first.")
endif()

if(NOT EXISTS "${COUNTER_UI}")
    message(FATAL_ERROR "counter.ui not found at ${COUNTER_UI}.")
endif()

# ── Import target ──────────────────────────────────────────────────────

add_library(wasamo SHARED IMPORTED)
set_target_properties(wasamo PROPERTIES
    IMPORTED_IMPLIB   "${WASAMO_IMPORT_LIB}"
    INTERFACE_INCLUDE_DIRECTORIES "${WASAMO_INCLUDE_DIR}"
)

# ── DSL build pipeline (DD-M2-P6-008) ──────────────────────────────────
#
# 1. wasamoc compiles counter.ui to counter.uic (IR text).
# 2. embed_uic.cmake generates counter_uic.h embedding the .uic bytes
#    as a static unsigned char array.
# 3. main.c includes counter_uic.h and passes the blob to wasamo_load_ui
#    (WASAMO_LOAD_MEMORY).

set(COUNTER_UIC   "${CMAKE_CURRENT_BINARY_DIR}/counter.uic")
set(COUNTER_UIC_H "${CMAKE_CURRENT_BINARY_DIR}/counter_uic.h")

add_custom_command(
    OUTPUT  "${COUNTER_UIC}"
    COMMAND "${WASAMOC_EXE}" build "${COUNTER_UI}" "${COUNTER_UIC}"
    DEPENDS "${COUNTER_UI}" "${WASAMOC_EXE}"
    COMMENT "wasamoc: counter.ui -> counter.uic"
    VERBATIM
)

add_custom_command(
    OUTPUT  "${COUNTER_UIC_H}"
    COMMAND "${CMAKE_COMMAND}"
            "-DUIC_FILE=${COUNTER_UIC}"
            "-DHEADER_FILE=${COUNTER_UIC_H}"
            -P "${CMAKE_CURRENT_SOURCE_DIR}/embed_uic.cmake"
    DEPENDS "${COUNTER_UIC}" "${CMAKE_CURRENT_SOURCE_DIR}/embed_uic.cmake"
    COMMENT "Generating counter_uic.h"
    VERBATIM
)

# ── Counter executable ─────────────────────────────────────────────────

add_executable(counter main.c "${COUNTER_UIC_H}")
target_include_directories(counter PRIVATE "${CMAKE_CURRENT_BINARY_DIR}")
target_link_libraries(counter PRIVATE wasamo)

if(MSVC)
    target_compile_options(counter PRIVATE /W3 /utf-8)
endif()

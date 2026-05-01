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
#
# The build verifies that main.c compiles and all ABI symbols link.
# The resulting executable is not run in CI (GUI only; wasamo.dll need
# not be on PATH at build time).

# ── Locate wasamo ──────────────────────────────────────────────────────

if(NOT DEFINED WASAMO_ROOT)
    # Default: two levels up from this file (examples/counter-c/ -> repo root)
    get_filename_component(WASAMO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
endif()

set(WASAMO_INCLUDE_DIR "${WASAMO_ROOT}/bindings/c")
set(WASAMO_IMPORT_LIB  "${WASAMO_ROOT}/target/release/wasamo.dll.lib")

if(NOT EXISTS "${WASAMO_IMPORT_LIB}")
    message(FATAL_ERROR
        "wasamo.dll.lib not found at ${WASAMO_IMPORT_LIB}.\n"
        "Run 'cargo build --release --workspace' from the Wasamo repo root first,\n"
        "or pass -DWASAMO_ROOT=<path> pointing to a pre-built tree.")
endif()

# ── Import target ──────────────────────────────────────────────────────

add_library(wasamo SHARED IMPORTED)
set_target_properties(wasamo PROPERTIES
    IMPORTED_IMPLIB   "${WASAMO_IMPORT_LIB}"
    INTERFACE_INCLUDE_DIRECTORIES "${WASAMO_INCLUDE_DIR}"
)

# ── Counter executable ─────────────────────────────────────────────────

add_executable(counter main.c)
target_link_libraries(counter PRIVATE wasamo)

if(MSVC)
    target_compile_options(counter PRIVATE /W3 /utf-8)
endif()

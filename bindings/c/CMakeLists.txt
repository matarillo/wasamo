cmake_minimum_required(VERSION 3.21)
project(wasamo_c_sample C)

# ── Usage ──────────────────────────────────────────────────────────────
#
# This CMakeLists.txt is the canonical template for C consumers of the
# Wasamo C ABI. Copy it into your project and adjust WASAMO_ROOT.
#
# Building from the Wasamo repo root (CI usage):
#
#   cmake -S bindings/c -B build/cmake-c \
#         -DWASAMO_ROOT=<path/to/wasamo/repo/root>
#   cmake --build build/cmake-c
#
# WASAMO_ROOT must contain:
#   bindings/c/wasamo.h        — public header
#   target/release/wasamo.dll  — the DLL (Windows)
#   target/release/wasamo.dll.lib — import library (Windows MSVC/clang)
#
# The build verifies that the header compiles and every ABI symbol links.
# The resulting executable is never run in CI (wasamo.dll need not be on
# PATH at build time).

# ── Locate wasamo ──────────────────────────────────────────────────────

if(NOT DEFINED WASAMO_ROOT)
    # Default: two levels up from this file (bindings/c/ -> repo root)
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

# ── Smoke executable ───────────────────────────────────────────────────
#
# Compiles and links smoke.c against wasamo.dll.lib. This is the same
# TU that the Phase 6 CI cl/clang-cl steps use; running it under CMake
# proves the CMake template itself works.
#
# /utf-8 ensures MSVC accepts the UTF-8 source file (matching the Phase 6
# cl invocation). clang-cl respects this flag too.

add_executable(wasamo_smoke smoke.c)
target_link_libraries(wasamo_smoke PRIVATE wasamo)

if(MSVC)
    target_compile_options(wasamo_smoke PRIVATE /W3 /utf-8)
endif()

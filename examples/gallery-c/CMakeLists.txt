cmake_minimum_required(VERSION 3.21)
project(wasamo_gallery_c C)

# Usage from the Wasamo repo root:
#
#   cargo build --release -p wasamo-runtime
#   cargo build --release --workspace
#   cmake -S examples/gallery-c -B build/gallery-c
#   cmake --build build/gallery-c --config Release
#
# The release workspace build provides target/release/wasamo.dll.lib and
# target/release/wasamoc.exe before this example shells out to wasamoc.

if(NOT DEFINED WASAMO_ROOT)
    get_filename_component(WASAMO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
endif()

set(WASAMO_INCLUDE_DIR "${WASAMO_ROOT}/bindings/c")
set(WASAMO_IMPORT_LIB  "${WASAMO_ROOT}/target/release/wasamo.dll.lib")
set(WASAMOC_EXE        "${WASAMO_ROOT}/target/release/wasamoc.exe")
set(GALLERY_UI         "${WASAMO_ROOT}/examples/gallery/gallery.ui")

if(NOT EXISTS "${WASAMO_IMPORT_LIB}")
    message(FATAL_ERROR
        "wasamo.dll.lib not found at ${WASAMO_IMPORT_LIB}.\n"
        "Run 'cargo build --release -p wasamo-runtime' and then\n"
        "'cargo build --release --workspace' from the Wasamo repo root first,\n"
        "or pass -DWASAMO_ROOT=<path> pointing to a pre-built tree.")
endif()

if(NOT EXISTS "${WASAMOC_EXE}")
    message(FATAL_ERROR
        "wasamoc.exe not found at ${WASAMOC_EXE}.\n"
        "Run 'cargo build --release -p wasamoc' from the Wasamo repo root first.")
endif()

if(NOT EXISTS "${GALLERY_UI}")
    message(FATAL_ERROR "gallery.ui not found at ${GALLERY_UI}.")
endif()

add_library(wasamo SHARED IMPORTED)
set_target_properties(wasamo PROPERTIES
    IMPORTED_IMPLIB "${WASAMO_IMPORT_LIB}"
    INTERFACE_INCLUDE_DIRECTORIES "${WASAMO_INCLUDE_DIR}"
)

set(GALLERY_UIC   "${CMAKE_CURRENT_BINARY_DIR}/gallery.uic")
set(GALLERY_UIC_H "${CMAKE_CURRENT_BINARY_DIR}/gallery_uic.h")

add_custom_command(
    OUTPUT "${GALLERY_UIC}"
    COMMAND "${WASAMOC_EXE}" build "${GALLERY_UI}" "${GALLERY_UIC}"
    DEPENDS "${GALLERY_UI}" "${WASAMOC_EXE}"
    COMMENT "wasamoc: gallery.ui -> gallery.uic"
    VERBATIM
)

add_custom_command(
    OUTPUT "${GALLERY_UIC_H}"
    COMMAND "${CMAKE_COMMAND}"
            "-DUIC_FILE=${GALLERY_UIC}"
            "-DHEADER_FILE=${GALLERY_UIC_H}"
            "-DARRAY_NAME=GALLERY_UIC"
            "-DHEADER_GUARD=WASAMO_GALLERY_UIC_H"
            -P "${CMAKE_CURRENT_SOURCE_DIR}/embed_uic.cmake"
    DEPENDS "${GALLERY_UIC}" "${CMAKE_CURRENT_SOURCE_DIR}/embed_uic.cmake"
    COMMENT "Generating gallery_uic.h"
    VERBATIM
)

add_executable(gallery-c main.c "${GALLERY_UIC_H}")
target_include_directories(gallery-c PRIVATE "${CMAKE_CURRENT_BINARY_DIR}")
target_link_libraries(gallery-c PRIVATE wasamo)

if(MSVC)
    target_compile_options(gallery-c PRIVATE /W3 /utf-8)
endif()

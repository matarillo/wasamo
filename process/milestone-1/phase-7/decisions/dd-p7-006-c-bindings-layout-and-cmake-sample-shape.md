### DD-P7-006 — C bindings layout and CMake sample shape

**Status:** Accepted (CMake build verifiable locally — see note below)

**Context:**
The C side already has `bindings/c/wasamo.h` and a smoke-test TU
(`bindings/c/smoke.c`) that CI builds with MSVC and clang-cl
(Phase 6). Phase 7 adds a "CMake sample" — i.e., a buildable
C consumer that a binding-author would copy as a starting point.
The shape of this sample affects how `bindings/c/` is organized.

**Options:**

Option A — `bindings/c/` holds header + import-lib copy + CMake template; sample lives in Phase 8 (recommended)
- `bindings/c/wasamo.h` (already present)
- `bindings/c/wasamo.dll.lib` — produced by the runtime build,
  copied into `bindings/c/` by the build script (or referenced
  by relative path from the workspace target dir)
- `bindings/c/CMakeLists.txt` — a template `add_library` /
  `target_include_directories` / `target_link_libraries` block,
  documented as "copy this into your project."
- The actual `examples/counter-c/` sample (Phase 8 work) consumes
  this template via `add_subdirectory` or `find_package`.

- What you gain: The reusable surface (header + import lib +
  CMake snippet) lives at a single, advertised location.
  Phase 7 produces the contract; Phase 8 produces the demo that
  exercises it.
- What you give up: A build engineer wanting to verify "does the
  CMake template actually work?" must wait for Phase 8. Mitigated
  by extending the existing CI smoke test to drive the CMake
  template (already builds and links a TU; this just changes the
  driver from "raw cl.exe" to "cmake --build").

Option B — Sample C app lives in `bindings/c/sample/` (Phase 7); Phase 8 just adds Counter
A standalone "minimal CMake consumer" sample inside `bindings/c/`,
distinct from `examples/counter-c/`.

- What you gain: Phase 7 has its own buildable artifact, not just
  a template.
- What you give up: Two C samples (`bindings/c/sample/` and
  `examples/counter-c/`) doing similar things. The `bindings/c/`
  one risks bit-rotting once `examples/counter-c/` becomes the
  real demo. ROADMAP Phase 8 already lists `examples/counter-c/`
  with a README; that is the real Phase 8 sample.

**Recommendation:** **Option A.** Phase 7 ships the *contract*
(header, import lib, CMake template, CI proof that all three link).
Phase 8 ships the *demo* (`examples/counter-c/`) consuming that
contract. Extend the existing smoke-test CI step to also drive a
CMake build of the same TU, so the template is CI-verified before
Phase 8 starts.

**Agreement note (2026-04-30):** The local SSH dev box has CMake
available at
`C:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe`
(bundled with the VS 2026 Community install). With the appropriate
PATH / `VCINSTALLDIR` environment set up (e.g. via
`vcvars64.bat`), the CMake template should be buildable locally
before pushing to CI. Confirming local buildability is part of the
implementation step for this item.

---

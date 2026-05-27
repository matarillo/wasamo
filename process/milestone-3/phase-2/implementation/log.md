## Decisions log

(empty — record here mid-phase decisions that deviate from the ADR
or refine its task slicing; see Phase 1's progress file for the
shape.)

---

## CI / verification log

- **2026-05-20 / T12 local:** `cargo build -p gallery-rust` — green.
- **2026-05-20 / T12 local:** `cargo build --release -p
  gallery-rust` — green.
- **2026-05-20 / T12 local GUI smoke:** `Start-Process
  .\target\release\gallery-rust.exe` — command succeeded. Manual
  visual correctness is owner-manual GUI smoke per framing decision G.
- **2026-05-20 / T12 owner-manual GUI smoke:** owner-provided
  screenshot `private/m3-p2-t12 screenshot 2026-05-20 232123.png`
  reviewed locally. The blue `Box` fill is visible, the
  `M3 Phase 2 Box` `Text` placeholder is centred inside it, and the
  Box occupies the expected 16:9 width-fit region within the window.
  The screenshot remains untracked under `private/`.
- **2026-05-20 / T12 step-end local:** `cargo fmt --all -- --check`
  — green.
- **2026-05-20 / T12 step-end local:** `cargo clean` — initial run
  blocked on the launched `gallery-rust.exe`; after it exited, rerun
  succeeded.
- **2026-05-20 / T12 step-end local:** `cargo build --release
  --workspace` — green (existing `wasamo` linkable target /
  `wasamo-sys` import library order warnings only).
- **2026-05-20 / T12 step-end local:** `cargo build --workspace` —
  green (same existing warnings only).
- **2026-05-20 / T12 step-end local:** `cargo test --workspace` —
  green.
- **2026-05-20 / T12 step-end local:** final `cargo fmt --all --
  --check` — green.
- **2026-05-20 / T13 phase-end local:** `cargo fmt --all --
  --check` — green.
- **2026-05-20 / T13 phase-end clean rebuild:** `cargo clean` —
  succeeded; removed 3027 files / 942.7 MiB.
- **2026-05-20 / T13 phase-end local:** `cargo build --release
  --workspace` — green (existing `wasamo` linkable target /
  `wasamo-sys` import library order warnings only).
- **2026-05-20 / T13 phase-end local:** `cargo build --workspace` —
  green (same existing warnings only).
- **2026-05-20 / T13 phase-end local:** `cargo test --workspace` —
  green, including T11 `aspect_box_with_text_child_lays_out_and_paints_fill`.
- **2026-05-20 / T13 spec re-sync:** `docs/dsl_spec.md` 0.7 →
  0.8; §4.9 Phase status marker flipped to `M3-Phase 2 closed;
  implementation-synced`. No draft / implementation divergence was
  found during the close re-sync.
- **2026-05-20 / T13 forward distillation:** M3-Phase 3 pre-doc
  input authored at `docs/notes/m3-phase-3/predoc-inputs.md`,
  carrying forward Box intrinsic sizing, placeholder thumbnails,
  spec-drafting, value-boundary, and verification constraints.
- **2026-05-20 / T13 phase-end retrospective:** recorded at
  `docs/notes/m3-phase-2/phase-end-retrospective.md`; T13's
  step-end pointer recorded at
  `docs/notes/m3-phase-2/t13-step-end-retrospective.md`.
- **2026-05-20 / T13 progress lifecycle:** frontmatter moved
  `active` → `closing` at T13 start, then `closing` → `retired`
  after checklist flip confirmation. The file remains present for
  owner review of the T13 flip; durable information has been
  distilled into the ADR, spec, CHANGELOG, notes, and M3 plan
  progress row.

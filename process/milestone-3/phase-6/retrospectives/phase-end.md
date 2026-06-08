---
title: M3-Phase 6 phase-end retrospective
status: recorded
created: 2026-06-09
scope: phase-end
phase: M3-Phase 6 — ZStack + conditional rendering
---

# M3-Phase 6 phase-end retrospective

## Scope

M3-Phase 6 ships `ZStack` plus conditional rendering as one gallery-lightbox
unit, closes the Phase 4 residual R1 static Window title, and syncs the
Phase 6 implementation back into the living specs. This retro is the
separate phase-end record for checklist items 12-18, after T9 was merged into
`feat/m3-phase-6` and the phase branch CI gate ran.

## Main Learnings

The strongest process learning is that the final-step split needs one more
sharp edge: local rebuild evidence and phase-branch CI evidence are different
ownership domains. T9 was right to leave the `workflow_dispatch` run id for
phase-end, but once T9 changed production Rust for A12 diagnostics, local
`fmt` / clean rebuild returned to T9 ownership. That split is now recorded in
the log and handoff.

The second learning is that frozen task lists age quickly when phase-local
owner decisions land. Phase 6 had DD-M3-P6-008 inserting T7b after T7, and
Observation 5 moving from "step 1 deferred" to "both remediation steps done".
Mutable phase docs must be revised against the current SSOT, not treated as
read-only historical predictions.

The third learning is that semantic migrations now have enough concrete
samples for a process rule. The `IrMember` migration showed how silent
filtering helpers can under-count structural members; the T7b `host_props` /
`host_bindings` migration showed a better call-site audit plus
compile-error-forcing construction path. Phase 7 / process work should codify
that forcing artifact in a VDR.

## Checklist

12. **Acceptance criteria (Ax) achieved:** **achieved**
    - **A4** is discharged. ZStack is accepted by `wasamoc`, emitted through
      textual IR, parsed / validated by the runtime loader, laid out with
      union sizing and document-order z-order, and proven by Windows-runtime
      z-order / clip fixtures plus gallery lightbox evidence.
    - **A7** is discharged. Conditional rendering ships as structural
      `IrMember::ControlFlow(ControlFlowNode::If)` with compile-time
      diagnostics, textual IR roundtrip, load-time presence, reactive
      present/absent mutation, declared Visual order, subtree disposal,
      fresh-on-return semantics, layout dirtying, and same-drain effect
      observation.
    - **R1** is closed. Static component `title:` reaches the native window
      through the existing internal load path with no new ABI export.
    - **A11 / A12** phase slices are synced: `docs/dsl_spec.md` and
      `docs/architecture.md` describe the landed ZStack, conditional,
      host-surface, and structural-mutation shapes. `docs/abi_spec.md` stayed
      untouched because the phase added no C ABI surface.
    - ADR "discharged" statements and implementation are consistent. No
      phase-end ADR touch case fired.

13. **`CHANGELOG.md` / `process/_roadmap.md` consistency:** **consistent**
    - `CHANGELOG.md` Unreleased now has an `M3-Phase 6 — ZStack +
      conditional rendering (2026-06-09)` entry covering A4 / A7, R1,
      host-surface separation, gallery evidence, spec sync, and carry-forward
      residuals.
    - `process/_roadmap.md` remains unchanged: it is the acceptance-criteria
      SSOT, and its A4 / A7 / A11 / A12 wording still matches the shipped
      Phase 6 scope. The phase status lives in `process/milestone-3/plan.md`,
      whose Phase 6 row is now `complete`.

14. **`VISION.md` / thesis-level claim impact:** **no direct update**
    - Phase 6 strengthens the M3 thesis by proving layout overlay and
      binding-driven tree shape in the gallery, but it does not change the
      thesis wording or roadmap categories.
    - The semantic-migration audit rule is process-governance material, not a
      product-vision claim. It is carried forward for a VDR rather than folded
      into `VISION.md`.

15. **Next-phase framing inputs:** **organized**
    - `implementation/handoff.md` is the phase-close handoff. Confirmed
      Phase 7 targets: control-flow family extension from `IrMember` /
      `ControlFlowNode`; `BindingTarget::ConditionalSubtree` as the structural
      binding seam; declared-tree / entity-tree identity and `key:` retention;
      placement storage-model decision; structural failure observability; and
      reactive-drain residuals 1-3.
    - Confirmed M4 / later targets: dynamic Window title / host bindings,
      modal lightbox input / focus capture, caption metrics / DPI sensitivity,
      DPI runtime quality, and real image / thumbnail-click behavior.
    - Closed / local-only items are not carried as open residuals: R1 static
      title is closed; Observation 5's Compositor owning-thread remediation
      steps are both DONE / committed; T9 local-only phase-sync material does
      not survive phase close.
    - Doc-folded material stays in the living docs rather than handoff prose:
      ZStack realised semantics, structural conditional semantics, ScrollView
      direct-conditional rejection, and component host-surface separation.

16. **CI green:** **green**
    - Phase branch `feat/m3-phase-6` was pushed at head
      `2b4f80f69bcdd62054fc37d55e04c282bba78cfb`.
    - GitHub Actions workflow `CI`, event `workflow_dispatch`, run
      [27149254110](https://github.com/matarillo/wasamo/actions/runs/27149254110)
      concluded **success** (`2026-06-08T15:43:27Z` ->
      `2026-06-08T15:46:37Z`).
    - Green CI steps: release workspace build, debug workspace build for
      tests, workspace tests, C ABI smoke (`cl` / `clang-cl`), CMake smoke,
      Zig binding smoke, `counter-c`, `counter-rust`, `counter-zig`, and
      `wasamoc check counter.ui`.
    - Local clean rebuild ground truth after the A12 code follow-up is
      recorded in `implementation/log.md`: `cargo fmt --all -- --check`,
      `cargo clean`, release workspace build, debug workspace build, and
      `cargo test --workspace` all green. The current CI workflow does not
      include a `cargo fmt` step, so fmt remains local evidence.

17. **Human-visible GUI smoke:** **needed; satisfied by phase-specific
    gallery smoke**
    - Phase 6 changed runtime / compiler / gallery-visible behavior, so
      human-visible smoke was needed. The relevant visible acceptance target
      was the gallery lightbox, not the counter examples.
    - T8 owner-manual gallery smoke passed after the final validator /
      gallery behavior was in place: closed -> open toggle, close without
      resize, z-order (photo / caption / nav over scrim), scrim fill after
      resize using a same-size positive control, native title `"Gallery"`,
      4:3 photo geometry, and caption/nav fit after the `32` -> `64`
      caption-row correction.
    - No additional phase-end human-visible smoke was rerun after T8 because
      post-T8 changes were diagnostics / documentation / process close work,
      not new GUI behavior. Counter hosts remain covered by the phase-branch
      CI smoke steps and by the T6 title observation.

18. **CI YAML sanity check:** **no change required**
    - Phase 6 added Rust code, IR/compiler/runtime behavior, and gallery
      source, but no new language, external build system, or CI matrix axis.
      Existing Windows CI already covers workspace build/test, bindings, and
      counter host smokes. No `.github/workflows/ci.yml` update is required.

## Merge Readiness

Checklist items 12-18 are recorded, the phase branch CI gate is green, and
`implementation/preamble.md` is in `status: closing`. Per the retrospective
procedure, this record is evidence for the phase -> main merge gate; it is not
itself merge authorization. The phase -> main no-ff merge still requires
explicit owner approval, and push remains a separate gate.

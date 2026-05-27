### DD-M3-P2-006 — Placeholder pattern (Box + Text child) canonicalization

**Status:** Accepted

**Context:**
[m3-target-app-predoc.md — 保留 2 closure](../notes/m3/m3-target-app-predoc.md#保留-2-closure-image-widget-surface-の-m3-開封可否--不開封-m4-へ-defer)
establishes that the M3 Image-widget deferral is carried by a
**Box + Text-child** placeholder. Phase 2 settles how this pattern is
canonicalized: where it lives in the spec, how it appears in the
example, and what later phases cite when they consume it.

**Options:**

Option A — Normative spec convention in `docs/dsl_spec.md`
(recommended)
- Add a dedicated subsection of the Box chapter titled "Image
  placeholder pattern (M3)". The pattern is spelled normatively:

  ```
  Box { aspect: <ratio>; fill: <color>; Text { text: <label> } }
  ```

  with the example forms typically used in the gallery (1:1 square,
  16:9 photo, neutral `#cccccc` fill, label text giving photo index
  or filename). The subsection records that the pattern is the
  agreed M3 substitute for the deferred Image widget surface, and
  that Phase 3 (WrapPanel of thumbnails) and Phase 6 (ZStack
  lightbox) consume the same pattern verbatim.

  - What you gain: Single citable spec location. Phase 3 and Phase 6
    spec writing cites it rather than redefining. M5 LSP / tooling
    sees a documented pattern. M4 Image-widget ADR has a clear
    "supersedes" target.
  - What you give up: Spec real estate — one subsection at Phase 2.
    Trivial.
  - **Technical risk:** Low. Spec writing only.

Option B — Informal pattern noted but not normative
- A passing mention in the Box chapter ("placeholders typically
  use Box + Text") without normative spec status.

  - What you give up: Phase 3 / Phase 6 either restate the pattern
    (drift risk) or cite an informal mention (weak citation).

Option C — Helper widget alias (e.g. `Placeholder { ... }`)
- A new widget kind that expands to the Box + Text pattern.

  - What you give up: A new widget for a deferred-Image bridge —
    the alias would have its own scope to defend. The M4 Image
    widget would supersede *both* Box + Text and the alias, doubling
    the supersession surface. Cleaner to keep the bridge structural,
    not nominal.

**Recommendation:** Option A — normative spec convention in
`docs/dsl_spec.md`. Phase 2 ships the subsection with the example
forms and the explicit cross-reference to Phase 3 and Phase 6's
expected usage. The Phase 2 `examples/gallery/gallery.ui` Box
sub-screen (framing decision F) demonstrates the pattern; the
Phase 2 spec marker
(`**Phase status:** M3-Phase 2 ADR-accepted design draft; pending
implementation re-sync`) sits at the top of the Box chapter and
applies to this subsection.

**Forward-compat exposure:** Option A's exposure under foreseeable
future events:

- M4 (or later) Image widget landing supersedes the placeholder
  pattern. The supersession is clean: the normative subsection
  gains a "Superseded by `<Image>` widget (M4 ADR)" header, and
  the spec retains the pattern as a back-compat shape for
  pre-Image authors. Phase 3 and Phase 6 spec citations remain
  valid because the pattern they cite is still spec-recorded;
  they migrate to `<Image>` syntactically when Image lands.

The pattern itself is structural (Box + Text), so it survives any
visual / styling refinement Phase 2's siblings make to Text or to
fill rendering.

---

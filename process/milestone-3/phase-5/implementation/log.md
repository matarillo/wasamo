## Decisions log

- **T1 — R-A pre-implementation spike: Grid track-list parser grafting
  shape + `n*` lexer-token decision (2026-05-29).** Settles
  [preamble.md risk R-A](./preamble.md#technical-risks-planning-time-recon)
  and the first
  [plan.md T1 bullet](./plan.md#t1--wasamoc-check-grid-surface-and-diagnostics)
  before the implementation bullets open. Spot-checked against
  [`wasamoc/src/parser.rs`](../../../../wasamoc/src/parser.rs),
  [`wasamoc/src/lexer.rs`](../../../../wasamoc/src/lexer.rs), and
  [`wasamoc/src/ast.rs`](../../../../wasamoc/src/ast.rs) at HEAD
  `ce82287`.

  **Problem confirmed.** `parse_property_bind`
  ([parser.rs:233](../../../../wasamoc/src/parser.rs#L233)) reads
  exactly one `Expr` through `parse_expr`
  ([parser.rs:381](../../../../wasamoc/src/parser.rs#L381)), which
  admits only single scalar tokens (`StringLit` / `IntLit` /
  `FloatLit` / `Measurement` / `Ident` / `true` / `false` /
  `RatioLit` / `ColorLit`). A whitespace-separated track list
  (`columns: 180 1* 2*`) cannot ride the generic property-bind path:
  after `180` the `parse_widget_decl` member loop re-enters
  `parse_member`, sees an `IntLit`, and errors `expected member`.
  Separately, the lexer's `'*'` arm
  ([lexer.rs:267](../../../../wasamoc/src/lexer.rs#L267)) errors
  `unexpected '*'` for any `*` not immediately followed by `=`, so
  `*` / `1*` do not tokenize at all today. **A lexer change is
  therefore unavoidable** — the only open question was its shape.

  **Decision 1 — routing: widget-type context in `parse_widget_decl`.**
  Every Grid (component root or nested) is parsed as a
  `Member::WidgetDecl` through `parse_widget_decl`
  ([parser.rs:251](../../../../wasamoc/src/parser.rs#L251)); a Grid's
  `columns:` / `rows:` only ever appear inside that widget body. So
  `parse_widget_decl` is the single, complete routing site. After
  reading `type_name` and `{`, compute `is_grid = type_name ==
  "Grid"` and, in the member loop, route to the Grid-specific
  track-list parser **only** when `is_grid` and the upcoming tokens
  are `Ident("columns" | "rows")` followed by `Colon`; every other
  member (including non-track Grid attributes and all non-Grid
  widgets) stays on `parse_member` unchanged. This is the
  widget-type context routing the risk anticipated; it does **not**
  open a general list/collection grammar (DD-M3-P5-002 Option A) and
  leaves `parse_property_bind` / `parse_expr` untouched. A
  `columns:` / `rows:` attribute on a non-Grid widget keeps its
  current generic-scalar behaviour (and is rejected downstream as an
  unknown attribute), so the track-list grammar never leaks outside
  Grid.

  **Decision 2 — lexer token: payload-less `Token::Star`, no fused
  `n*` literal.** The minimal lexer change wins: the `'*'` arm emits
  a new payload-less `Token::Star` instead of erroring when the next
  char is not `=` (the `*=` → `Token::StarEq` path is untouched).
  The lexer learns *nothing* about track lists — it does not fuse an
  adjacent leading integer into a weighted-star literal. All
  track-grammar knowledge (fixed vs unit-star vs weighted-star,
  weight range `[1, 1024]`, fixed `>= 1`) lives in the Grid-scoped
  parser path, matching DD-M3-P5-002's "narrow Grid-specific parser
  path" boundary. Considered and rejected: fusing `n*` into a single
  `StarLit(weight)` token in `scan_number` (the
  `RatioLit` / `Measurement` precedent). Rejected because
  `RatioLit` / `Measurement` are general-purpose value literals
  usable across many attributes, whereas a weighted-star is
  meaningful *only* inside a Grid track list; putting `n*` fusion in
  the lexer would add a globally-recognised token for a hyper-local
  grammar. The payload-less `Star` is the smallest concession (one
  character that was previously valid only as `*=`); a stray `*` in
  any non-track position now surfaces a parser-level `expected
  expression`/`expected member` diagnostic instead of the old
  lexer-level `unexpected '*'`, which is acceptable (no existing
  valid `.ui` uses a bare `*`).

  **Decision 3 — adjacency via spans distinguishes `1*` from `1 *`.**
  The track grammar must separate `1*` (one weighted-star track of
  weight 1) from `1 *` (a `Fixed(1)` track followed by a unit-star
  track) — both are legitimate but different track lists. With the
  payload-less `Star`, the Grid track-list parser reassembles the
  weighted star by checking byte-span adjacency: an `IntLit(n)`
  immediately followed by `Star` with `int_tok.span.end ==
  star_tok.span.start` lowers to `Star(n)`; otherwise the `IntLit`
  is a `Fixed(n)` track and a following `Star` (non-adjacent or
  standalone) is a unit `Star(1)`. This is the same adjacency
  mechanism `RatioLit` already relies on (whitespace before `:`
  defeats the ratio — see lexer test
  `integer_with_whitespace_before_colon_is_not_ratio`), so the
  `1 *` vs `1*` rule is consistent with existing surface behaviour.
  Weight/fixed value-range validation is deferred to `wasamoc check`
  (`auto` reserved-future and `1.5*` rejections likewise), keeping
  the lexer free of value policy exactly as it is for `RatioLit`
  (`0:0` lexes; validity checked later).

  **Decision 4 — AST carrier: new `Member::GridTracks` variant.** The
  track-list parser produces a new `Member` variant (working shape
  `Member::GridTracks { axis: TrackAxis, tracks: Vec<TrackSize>,
  span }`, with a `wasamoc`-side `TrackSize` AST enum mirroring the
  `wasamo-ir` `TrackSize { Fixed(i32), Star(u32) }` of DD-M3-P5-002)
  rather than reusing `Member::PropertyBind` with a vector-shaped
  `Expr`. This keeps `Expr` strictly scalar and keeps `columns:` /
  `rows:` off the generic property/`IrProp` path, which is the AST-side
  expression of DD-M3-P5-001 carrier **c1** ("Grid track lists live
  outside `IrProp`, in a Grid-specific payload; `IrProp.value` stays
  strictly `IrLiteral`"). Lowering reads `Member::GridTracks` into the
  `KindPayload::Grid { columns, rows }` carrier; non-track Grid
  attributes and all `Cell` attributes continue through the existing
  `PropertyBind` → `IrProp` machinery. Exact field/enum names are
  finalised when the implementation bullets land and are reconciled
  with `wasamo-ir`'s `KindPayload` at that point.

  **Carried into the implementation bullets.** R-C (the
  `kind_payload` construction-site spread) is settled inside the T1
  `wasamoc` emit bullet and the T3 runtime-loader bullet, not in this
  spike; the shared textual IR shape produced by T1 emit feeds the T7
  `dsl_spec.md` §8 fold per the plan.

- **T1 — R-C construction-site discipline + carrier-c1 textual IR
  shape (2026-05-29).** Settles the two items the R-A spike deferred
  to the implementation bullets.

  **R-C disposition: explicit field, no `Default`.** Adding
  `IrNode.kind_payload: Option<KindPayload>` (DD-M3-P5-001 carrier c1)
  touches 5 construction sites workspace-wide: 1 in `wasamoc` lowering
  ([`wasamoc/src/lower.rs`](../../../../wasamoc/src/lower.rs) — the
  Grid arm sets `Some(KindPayload::Grid { .. })`, every other kind
  `None`), 1 production site + 3 round-trip-test sites in the runtime
  loader ([`wasamo-runtime/src/ir_loader.rs`](../../../../wasamo-runtime/src/ir_loader.rs)
  — all `None`; T3 sets the Grid payload). The IR types deliberately
  derive **no** `Default` (matching the existing no-`Default` style —
  no `..Default::default()` appears anywhere in the codebase today),
  so the new field surfaces every site at compile time rather than
  silently defaulting. The R-A spike risk-table option "derive
  `Default` / add an `IrNode::new` builder" was rejected on that
  consistency ground: a compile error at each site is the desired
  forcing function for a structural IR change.

  **Carrier-c1 textual IR shape (T7 §8 fold input).** `wasamoc emit`
  writes Grid track lists as keyword-led lines at the top of the
  Grid node body, parallel to the existing `prop` / `bind` / `on`
  line forms:

  ```text
  node Grid {
      tracks columns = 180 1* 2*
      tracks rows = 1* 1*
      node Cell {
          prop row = 0
          prop column = 0
          node Text { prop text = "header" }
      }
  }
  ```

  - `tracks <axis> = <track-list>` where `<axis>` is `columns` /
    `rows` and `<track-list>` is a space-separated sequence of fixed
    integers and `<weight>*` star tokens. Unit star is canonicalised
    to `1*` (the IR weight is explicit; mirrors the canonical
    color-emit alpha normalisation). Track lists never appear as
    `prop` entries — the carrier-c1 invariant (`IrProp.value` stays
    strictly `IrLiteral`).
  - `Cell` emits as an ordinary `node Cell { … }` subtree carrying
    `prop row`/`column`/`row-span`/`column-span`/`h-align`/`v-align`
    as standard `IrLiteral` entries (Int + Ident). The runtime loader
    flattens Cell subtrees into Grid's effective children + per-Cell
    placement in T3; T1 only fixes the emit shape.

  This textual shape is the concrete target for the T3 runtime-loader
  `tracks` parse and the T7 `dsl_spec.md` §8 grammar / AST fold (both
  deferred from Moment 1 because the carrier's textual form pins at
  implementation time).
</content>
</invoke>

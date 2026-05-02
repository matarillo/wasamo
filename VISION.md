# Wasamo Vision

**Status:** Pre-alpha, design in progress

> This document describes why Wasamo exists, what it prioritizes, and where it's headed.
> For implementation details see [docs/architecture.md](./docs/architecture.md); for phase-by-phase design decisions see [docs/decisions/](./docs/decisions/).

## 1. TL;DR

Wasamo is a Windows-only declarative UI framework. UI is written in an external DSL (`.ui` files) and consumed from any language through a stable C ABI — including non-managed languages such as C, Rust, Swift, Zig, and Go. Rendering uses Windows.UI.Composition (the Visual Layer) directly, and Mica/Acrylic, system theming, and the Windows type ramp are first-class DSL concepts.

Cross-platform support is explicitly out of scope. Two goals govern the project: quality of experience on Windows, and contribution to the multi-language OSS ecosystem.

## 2. Why this exists

### 2.1 The problem we observe

Building a modern native UI app for Windows today forces a major sacrifice no matter which framework you pick.

**WinUI 3 / XAML** delivers native Windows quality but is, in practice, deeply tied to the C# / .NET ecosystem. C++/WinRT is technically available, but the complexity of WinRT projection makes it impractical to consume from Rust, Swift, Zig, Go, and similar non-managed languages.

**Electron / Tauri** brings web productivity and ecosystem, at a heavy cost in memory, startup time, and binary size. HTML rendering also can't tap into what the Windows compositor offers — Mica/Acrylic, OS-level energy efficiency, vSync alignment with the system.

**Flutter** gives you a modern DSL (Dart) and high-quality rendering, but Windows is a second-class target — system integration (Mica, theming, native dialogs) is weak. It also locks you into Dart and gives up multi-language support entirely.

**Slint** is the closest spiritual sibling to this project, but its rendering abstraction sits at the pixel level for cross-platform reasons. That makes it structurally hard to take advantage of a retained compositor like the Visual Layer — independent compositor-thread rendering, vSync alignment with the OS, and integration with system materials are all properties Slint cannot easily inherit. Windows-specific vocabulary like Mica/Acrylic and system accent is also difficult to express through that abstraction.

### 2.2 The hypothesis

**"By committing to Windows-only and choosing an external DSL plus a C ABI runtime, we can resolve all of the above tensions at once."** This is the hypothesis Wasamo exists to test.

- Going Windows-only lets us call Visual Layer and TSF directly, without an abstraction tax
- An external DSL means we don't depend on host-language syntax features, so every language gets the same declarative experience
- A C ABI doesn't get in the way of third-party language bindings

`.ui` is the canonical declarative form, but the C ABI also permits host languages to construct UI directly. Internal DSLs built on top of bindings — Rust macros, Swift result builders, and the like — are welcomed as derivative shapes that serve language-specific developer experience; they do not replace `.ui` as the canonical form. The conditions under which `.ui` and the C ABI evolve in response to such experiments are recorded in [docs/decisions/vision-internal-dsl-policy.md](./docs/decisions/vision-internal-dsl-policy.md).

The project's purpose is to validate this hypothesis and carry the implementation to production-grade quality.

### 2.3 Non-goals

The following are explicitly *not* on the table. Letting these go is what makes the goals achievable.

- **macOS / Linux / Web support.** Cross-platform support fundamentally conflicts with the benefits of going Windows-only
- **Bug-for-bug compatibility with existing UI frameworks** (WinUI/WPF/SwiftUI/etc.). We'll learn from them, but we won't preserve their APIs or syntax
- **Game-grade frame rates.** We target the smoothness of a typical GUI app (60-120fps), not extreme optimization
- **Mobile/touch-only apps.** Touch input is supported, but the primary target is desktop with keyboard and mouse
- **A vast bundled widget library.** The core stays minimal; widget growth is left to the ecosystem

## 3. Target users

### 3.1 Primary users (developers building UIs)

**Teams shipping Windows-only business or internal tools.** Developers who know the deployment target is just Windows, who don't need to pay the cost of cross-platform support, and who want a lighter and more native option than Electron.

**Developers writing Windows desktop apps in non-managed languages** (Rust/Swift/Zig/Go and friends). Today they're stuck with GTK, Qt, egui, or iced — none of which deliver a Windows-native feel. Wasamo gives this audience a native-feeling Windows option for the first time.

**Indie developers building developer tools, IT admin utilities, launchers, viewers.** Domains where small binaries, fast startup, and low memory matter.

### 3.2 Secondary users (ecosystem contributors)

- Authors of language bindings
- Authors of design systems (component libraries built on the Wasamo DSL)
- IDE/editor extension developers (LSP, syntax highlighting, preview)
- Authors of alternative DSL implementations (alternative compilers, runtime ports)

### 3.3 Not the target

- Teams that need cross-platform support → consider Slint, Tauri, or Flutter
- Teams that want to stay inside the C# / .NET ecosystem → consider WinUI 3
- Teams that need browser deployment → out of scope for this project
- Game UI → consider a dedicated engine (Dear ImGui and similar)

## 4. Product principles

When design choices come into conflict, we resolve them in this order.

1. **Native Windows feel.** Mica/Acrylic, system theming, type ramp, and accent must look right by default. Animation is opt-in (consistent with the conventions of SwiftUI, Jetpack Compose, Flutter, and CSS); when invoked, it is compositor-driven and smooth without app effort.
2. **Minimum app code.** UI structure in the DSL, logic in the host language; both expressed in their shortest natural form. The underlying model is declarative and unidirectional: the view is a pure function of state (`view = f(state)`), state flows down through property bindings, and user interactions flow up as events handled by host-language callbacks.
3. **Lean resources.** Target <100ms cold start, <30MB memory, single-digit-MB binaries
4. **Multi-language support.** The C ABI is the primary boundary. Language-specific optimizations are secondary
5. **Contribution to the OSS ecosystem.** Permissive licensing, open specifications, hospitality toward third-party extensions

To make this concrete: if a Swift-specific optimization would compromise API neutrality, principle 4 wins and we don't take it. If beautiful default rendering inflates the binary, we weigh principle 1 against principle 3, favoring principle 1 in places that hit first impressions and principle 3 in internal implementation details.

## 5. Differentiators

| Dimension | Wasamo | WinUI 3 | Slint | Flutter | Electron |
|---|---|---|---|---|---|
| Windows feel (Mica/Acrylic) | ◎ in DSL | ◎ native | △ via abstraction | △ reproduced | × |
| Multi-language | ◎ C ABI first | △ .NET-centric | ○ multiple official | × Dart only | △ web stack |
| Lean resources | ◎ AOT + native | ○ | ◎ lean | △ heavy runtime | × |
| Declarative UI | ◎ external DSL | ○ XAML, verbose | ◎ external DSL | ○ internal DSL | △ HTML |
| OSS license | ◎ MIT/Apache | ○ MIT | ○ MIT/GPL/Commercial | ◎ BSD | ◎ MIT |
| Cross-platform | × non-goal | × | ◎ | ◎ | ◎ |

The position Wasamo carves out is **"Slint's design philosophy × XAML's Windows vocabulary × multi-language openness via C ABI."** Drop any one of those three and an existing OSS project covers the gap. Combine all three and there's no existing project that fits — that's the niche.

## 6. Architecture overview

```
┌─────────────────────────────────────────────────┐
│  App Code  (Rust / Swift / Zig / Go / C / ...)  │
│    business logic, state, callbacks             │
├─────────────────────────────────────────────────┤
│  Generated Bindings  (per-language, build-time) │
│    typed view handles, property accessors       │
├─────────────────────────────────────────────────┤
│  .ui DSL files                                  │
│    ↓ AOT compile (wasamoc)                      │
├─────────────────────────────────────────────────┤
│  Wasamo Runtime  (wasamo.dll, C ABI)            │
│    Reconciler / Layout / Property bindings      │
│    Animation / Input / IME / Accessibility      │
├─────────────────────────────────────────────────┤
│  Render Backend                                 │
│    Windows.UI.Composition (Visual Layer)        │
│    + DirectWrite + Direct2D + WIC               │
├─────────────────────────────────────────────────┤
│  OS:  Windows 10 1809+ (Win32 HWND host)        │
└─────────────────────────────────────────────────┘
```

Responsibilities by layer:

**App Code** is written in the host language. State, business logic, and external API calls live here. UI structure typically does not.

**Generated Bindings** are auto-generated from `.ui` files. Each language gets a typed API in its own idioms; under the hood they call the C ABI.

**.ui DSL** describes UI structure, properties, simple expressions, and reactivity. The `wasamoc` AOT compiler turns it into native code or an intermediate binary.

**Wasamo Runtime** is a single DLL exposed via C ABI. It owns the reconciler, layout, input handling, IME, and accessibility. The implementation language is Rust.

**Render Backend** calls Windows components (Visual Layer, DirectWrite, etc.) directly. There is no intermediate abstraction.

For the full story, see [docs/architecture.md](./docs/architecture.md).

## 7. Roadmap

Each milestone closes a single thesis — a hypothesis the milestone
verifies — rather than a feature checklist. The goal is to keep
verification scoped and avoid wishlist-style milestones.
[ROADMAP.md](./ROADMAP.md) is the SSOT for acceptance criteria; the
thesis summaries below are vision-level framing
([DD-V-010](./docs/decisions/vision-doc-system.md#dd-v-010--acceptance-criteria-ssot)).

- **M1 — Proof of concept** ✅ shipped 2026-05-01. Validated the core
  hypothesis (external DSL × C ABI × Visual Layer) end-to-end.
  See [CHANGELOG.md](./CHANGELOG.md).
- **M2 — Foundation.** Close the loop on the DSL side: `.ui` files
  drive the runtime through reactive state propagation, replacing
  M1's host-imperative widget tree construction.
- **M3 — DSL surface.** The DSL is expressive enough to write real
  layouts, and the `.ui` specification is published as a public
  draft.
- **M4 — Interaction stack.** Input, multi-window, IME, AccessKit,
  and TextField ship together because they share a focus model.
  Mica / Acrylic ships here so Wasamo's identity feature is
  demonstrable in the first contributor-facing showcase.
- **M5 — Identity & tooling.** Full theming surface, the official
  widget set beyond TextField, and the VS Code extension.
- **M6 — 1.0 (C ABI stabilization).** ABI freeze and SemVer
  commitment. Performance targets met. Polished showcase ships.
  C / Rust / Zig bindings mature.
- **Post-1.0.** Hot reload, higher-level animation DSL, advanced
  layout, system integration features, Swift / Go community bindings.

## 8. Success metrics

OSS projects aren't measured by revenue, but by **adoption and health**. We track these signals (target values will be set once we hit M3).

**Adoption signals**
- GitHub stars (as a rough indicator)
- Monthly downloads (crates.io, NuGet, etc.)
- Real product adoption (showcase entries)

**Community health**
- Monthly active contributors
- Median issue response time
- Median PR merge time
- Number of community-built bindings

**Technical quality**
- Performance target compliance at M3
- Regression test coverage
- Documentation coverage

These are **signals we observe**, not goals we set. Setting numerical goals too early creates a self-fulfilling trap, and at this stage even the choice of metric is unproven.

## 9. Governance and license

### 9.1 License

**Dual-licensed under MIT and Apache-2.0.**

Why:
- GPL/LGPL would block third-party binding authors, especially anyone integrating with closed-source commercial products
- Combining MIT with the patent grant from Apache-2.0 lowers the psychological barrier for enterprise use compared to MIT alone
- This dual layout is the de facto standard in the Rust ecosystem, so it's familiar to contributors we hope to attract

### 9.2 Decision-making

**Early stages (M1-M2).** BDFL (Benevolent Dictator) model. Design coherence comes first. Implementation decisions are recorded as Architecture Decision Records (ADRs) in [docs/decisions/](./docs/decisions/), one file per phase.

**M3 onward.** Gradual transition to RFC-based consensus. Major changes are discussed in documents under [docs/rfcs/](./docs/rfcs/), with adoption decided by core maintainer agreement.

**Post-1.0.** Fully open governance. We'll consider establishing a Technical Steering Committee.

### 9.3 Code of conduct

We adopt [Contributor Covenant 2.1](https://www.contributor-covenant.org/). A healthy community is a precondition for OSS success, and we treat it on par with technical quality.

### 9.4 Independence of the core specification

The `.ui` DSL specification is maintained as a document — [docs/dsl_spec.md](./docs/dsl_spec.md) — separate from the reference implementation. This:

- Leaves room for future third-party implementations
- Catches drift between specification and implementation early
- Signals the project's commitment to OSS principles

The C ABI header gets the same treatment once stabilized: a separate, normative specification document.

`.ui` is canonical; internal DSLs built on host-language bindings (Rust macros, Swift result builders, Zig `comptime`, Go builders, …) are welcomed as derivative shapes that serve language-specific developer experience without claiming canonical status. The project does not commit to keeping `.ui` at feature parity with the most expressive internal DSL. `.ui` is extended only when a proposed feature is both motivated by an end-user product capability that cannot be provided through bindings or design-system components, and expressible across all officially supported bindings. C ABI changes follow the same review process as any other ABI proposal under the M4 stability commitment. Full rationale and gating conditions are recorded in [docs/decisions/vision-internal-dsl-policy.md](./docs/decisions/vision-internal-dsl-policy.md).

### 9.5 Trademark and naming

We intend to register the "Wasamo" name and logo as trademarks in the future. The trademark would be held by the project's governance entity (TBD at the time of registration), with a permissive policy for community use. Forking is unconditionally allowed; what's restricted is the right of a fork to call itself "Wasamo."

## 10. Risks and assumptions

### 10.1 Technical risks

**TSF/IME integration complexity.** Japanese, Chinese, and Korean input goes through Text Services Framework, whose API is complex and under-documented. We'll lean on Flutter embedder and Chromium implementations as references, but expect quality to be limited in early releases.

**Future Visual Layer API changes.** The probability of Microsoft deprecating the Visual Layer is low but not zero. We keep the architecture open enough to swap in WinUI 3's DirectComposition abstraction as a fallback path.

**AccessKit / UIA fit.** AccessKit's Windows backend is still maturing. Edge cases may force us to write parts of UIA integration ourselves.

**Reconciler performance.** Maintaining 60fps on large lists (thousands of items) takes care. LazyList implementation is explicitly scheduled for M5 or later.

### 10.2 Strategic risks

**Microsoft delivers an equivalent experience via WinUI 3.** If WinUI 3 adds proper multi-language support — say, a C ABI projection or first-class Rust support — the rationale for Wasamo weakens. Based on the last decade of behavior, we judge this unlikely.

**Slint allows pluggable backends.** A Visual Layer backend in Slint would erode the differentiation. That said, Slint's design philosophy (cross-platform first) structurally conflicts with parts of native Windows feel, so we don't think it would be a complete substitute.

**Contributors don't show up.** This targets a niche audience (Windows-only, multi-language). Initial contributors might not materialize. We mitigate by (a) the lead maintainer committing to 2-3 years of solo work if needed, and (b) shipping showcase apps early so the project has something concrete to point at — anchored as a first contributor-outreach showcase in M4 (rough but identity-complete) and a polished showcase in M6 (1.0).

### 10.3 Assumptions

- **OS floor.** Windows 10 1809 (build 17763). The Visual Layer surface, Mica, and AccessKit all stabilize from this version
- **GPU requirement.** DirectX 11 equivalent. Older environments are out of scope
- **Implementation language.** The runtime is Rust. We considered C, but chose Rust for windows-rs, ownership, and ecosystem maturity
- **Distribution form.** The runtime is a single DLL. Language bindings ship separately

## 11. How to contribute

Wasamo is currently pre-alpha. You can help in several ways.

**Join the design discussion.** GitHub Discussions hosts active topics. We particularly welcome feedback on the validity of the [non-goals](#23-non-goals), the direction of the DSL syntax, and our priority calls.

**Contribute code.** [Good first issues](https://github.com/matarillo/wasamo/issues?q=label%3A%22good+first+issue%22) on the M1 roadmap are a reasonable starting point.

**Record a decision.** For M1-M2 implementation decisions, create an ADR in [docs/decisions/](./docs/decisions/) following the format in [docs/decisions/README.md](./docs/decisions/README.md). From M3 onward, substantial feature proposals follow the RFC process in [docs/rfcs/](./docs/rfcs/).

**Documentation and samples.** Examples for each language, tutorials, and best practices are always needed.

**Build a binding.** Official bindings are limited to C, Rust, and Zig (the three verified end-to-end in M1; their maturity is a 1.0 acceptance criterion in [ROADMAP.md](./ROADMAP.md)). For everything else (Swift, Go, Nim, Crystal, Odin, etc.), we encourage and support community-maintained bindings.

Channels:
- GitHub Issues / Discussions — design, bugs, feature requests
- (Future) Discord or Matrix server
- (Future) Mailing list

## 12. Glossary

**DSL (Domain Specific Language).** A language built for a specific domain. In Wasamo, this refers to the `.ui` declarative UI language.

**Visual Layer.** The retained-mode GPU compositor API exposed through Windows.UI.Composition. Instead of the app thread issuing draw calls each frame, the app registers a tree and the system handles compositing and animation on its own thread.

**Reconciler.** The mechanism that compares the previous and next view trees in a declarative UI framework, computing the diff (create/update/remove). Equivalent to React Fiber or SwiftUI's graph update engine. In Wasamo, this is a separate concern from the layout engine.

**AOT (Ahead-of-Time) compilation.** Compiling to native code before execution, as opposed to JIT. Wasamo applies this to `.ui` files at build time.

**C ABI.** A language-neutral function calling convention. A boundary defined by a C header that any language can call into.

**Mica / Acrylic.** Translucent materials introduced in Windows 11. Mica is a subtle tint sampled from the desktop background; Acrylic adds a frosted-glass blur.

**Type ramp.** The Windows design system's hierarchy of typographic styles by use (caption, body, bodyStrong, subtitle, title, titleLarge, display).

**TSF (Text Services Framework).** Windows' integrated text input framework. Used for IME (Input Method Editor) integration.

**AccessKit.** A Rust-based, cross-platform accessibility abstraction. On Windows it maps to UIA (UI Automation); on macOS to NSAccessibility; on Linux to AT-SPI.

**Visual Tree / View Tree.** The hierarchical structure that represents the UI. Wasamo maintains three layers — the declared tree (what the user wrote in `.ui`), the view tree (managed by the reconciler), and the visual tree (in Visual Layer).

## Appendix A: Related documents

- [README.md](./README.md) — Glanceable introduction
- [docs/architecture.md](./docs/architecture.md) — Technical architecture in depth
- [docs/dsl_spec.md](./docs/dsl_spec.md) — `.ui` DSL specification
- [ROADMAP.md](./ROADMAP.md) — Detailed milestones
- [CHANGELOG.md](./CHANGELOG.md) — What has shipped
- [CONTRIBUTING.md](./CONTRIBUTING.md) — Contribution guide
- [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) — Code of Conduct
- [docs/decisions/](./docs/decisions/) — Architecture Decision Records (ADRs)

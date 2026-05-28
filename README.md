# Wasamo

> Native Windows feel, from any language, declaratively.

Wasamo is a Windows-only declarative UI framework. You describe your UI in a `.ui` DSL and call into the runtime through a stable C ABI from any language. Rendering goes directly through Windows.UI.Composition (the Visual Layer), so Mica/Acrylic, system theming, and high-DPI composition all work out of the box.

```
┌──────────────────────────────────────────────┐
│  App (Rust / Swift / Zig / Go / C / ...)     │
│       ↕ generated bindings                   │
│  .ui DSL  →  AOT compiler                    │
│       ↕ C ABI                                │
│  Wasamo Runtime (Rust)                       │
│       ↕                                      │
│  Windows.UI.Composition + DirectWrite + TSF  │
└──────────────────────────────────────────────┘
```

## What it looks like

```
// counter.ui
component Counter inherits Window {
    title: "Counter"
    backdrop: mica
    theme: system

    in-out property <int> count: 0

    VStack {
        spacing: 12px
        padding: 24px

        Text {
            text: "Count: \{root.count}"
            font: title
        }
        Button {
            text: "Increment"
            style: accent
            clicked => { root.count += 1; }
        }
    }
}
```

The host language only handles bindings and logic.

```rust
fn main() {
    let ui = Counter::new();
    ui.show();
    Wasamo::run();
}
```

## Why Wasamo

- **Native Windows feel** — Mica/Acrylic, system theming, and the Windows type ramp are first-class concepts in the DSL
- **Language-agnostic** — Any language that can call C ABI is a first-class citizen: C, Rust, Swift, Zig, Go, and more
- **Lean on resources** — Targets <100ms cold start and <30MB memory. AOT compilation eliminates runtime overhead
- **Less code** — UI structure lives in the DSL, logic lives in your language. Both written in their shortest natural form
- **OSS-first** — Dual-licensed MIT/Apache-2.0; the DSL spec is maintained independently of the reference implementation

## How it compares

| | Wasamo | WinUI 3 | Slint | Flutter | Electron |
|---|---|---|---|---|---|
| Native Windows feel | ◎ | ◎ | △ | △ | × |
| Multi-language | ◎ | △ | ○ | × | △ |
| Lean resources | ◎ | ○ | ◎ | △ | × |
| OSS ecosystem | ◎ | △ | ○ | ◎ | ◎ |

See [VISION.md](./VISION.md#5-differentiators) for a full discussion.

## Status

**Pre-alpha.** M1 (proof of concept) shipped as
[v0.1.0](https://github.com/matarillo/wasamo/releases/tag/v0.1.0) on
2026-05-01; M2 (Foundation) shipped on 2026-05-11. The DSL now drives
the runtime through the M2 `wasamoc` -> IR -> `wasamo_load_ui` path, with
reactive state propagation in the C, Rust, and Zig counter examples. See
[CHANGELOG.md](./CHANGELOG.md#m2-foundation--shipped-2026-05-11) and
[process/milestone-2/plan.md](./process/milestone-2/plan.md). Not ready for
production use. Design discussion and contributions to M3 are welcome.

Future milestones live in [process/_roadmap.md](./process/_roadmap.md); shipped
milestones in [CHANGELOG.md](./CHANGELOG.md); ADRs
in [process/](./process/README.md).

## Requirements

- Windows 10 1809 (build 17763) or later
- A GPU capable of DirectX 11

## Quick start

> **M2 (Foundation).** The counter examples load
> [`examples/counter/counter.ui`](./examples/counter/counter.ui) through
> the DSL compiler and runtime IR loader. Broader DSL surface work starts
> in M3.

**Prerequisites:** Visual Studio 2022 Build Tools, CMake ≥ 3.21, Rust stable (MSVC target).

```bat
rem Clone and build the runtime DLL
git clone https://github.com/matarillo/wasamo.git
cd wasamo
cargo build --release --workspace

rem Build and run the Hello Counter example (C)
cmake -S examples/counter-c -B build/counter-c
cmake --build build/counter-c --config Release
copy target\release\wasamo.dll build\counter-c\Release\
build\counter-c\Release\counter.exe
```

The same counter is also available in Rust and Zig:

- [examples/counter-rust/](./examples/counter-rust/README.md) — Rust safe wrapper
- [examples/counter-zig/](./examples/counter-zig/README.md) — Zig binding

## Documentation

- [VISION.md](./VISION.md) — Why this project exists, what it values, how it's governed
- [docs/architecture.md](./docs/architecture.md) — Technical architecture in depth
- [docs/dsl_spec.md](./docs/dsl_spec.md) — The `.ui` DSL language specification
- [process/_roadmap.md](./process/_roadmap.md) — Future milestones and acceptance criteria
- [CHANGELOG.md](./CHANGELOG.md) — Shipped milestones
- [CONTRIBUTING.md](./CONTRIBUTING.md) — How to contribute
- [process/](./process/README.md) — Architecture Decision Records (ADRs)

## License

Dual-licensed under MIT or Apache-2.0, at your option. See [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE).

## Community

- GitHub Discussions — design discussion, use cases
- Issue Tracker — bug reports, feature requests
- Code of Conduct — we follow the [Contributor Covenant](./CODE_OF_CONDUCT.md)

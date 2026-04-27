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

**Pre-alpha.** This is a proof-of-concept stage project; not ready for production use. We welcome design discussion and contributions to the foundational implementation.

The roadmap lives in [ROADMAP.md](./ROADMAP.md). Design decisions are archived as RFCs under [docs/rfcs/](./docs/rfcs/).

## Requirements

- Windows 10 1809 (build 17763) or later
- A GPU capable of DirectX 11

## Quick start

```bash
# Coming soon. Will be published once milestone M1 is reached.
```

## Documentation

- [VISION.md](./VISION.md) — Why this project exists, what it values, how it's governed
- [docs/architecture.md](./docs/architecture.md) — Technical architecture in depth
- [docs/dsl_spec.md](./docs/dsl_spec.md) — The `.ui` DSL language specification
- [ROADMAP.md](./ROADMAP.md) — Milestones and acceptance criteria
- [CONTRIBUTING.md](./CONTRIBUTING.md) — How to contribute
- [docs/rfcs/](./docs/rfcs/) — Design decision archive

## License

Dual-licensed under MIT or Apache-2.0, at your option. See [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE).

## Community

- GitHub Discussions — design discussion, use cases
- Issue Tracker — bug reports, feature requests
- Code of Conduct — we follow the [Contributor Covenant](./CODE_OF_CONDUCT.md)

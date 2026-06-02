# NeuDel-II

A game engine built from scratch, targeting 2D and 2.5D games — platformers, metroidvanias, and Paper Mario-style turn-based RPGs.

No forks, no shortcuts — just raw Rust, a custom editor, and full control over every system. We're building this because we want to actually *understand* what's happening under the hood. Real rendering, real physics, real scripting — not a black box.

---

## What it is

**NeuDel-II** is two things:

1. **A game engine** — written in Rust. Handles rendering, physics, audio, and a full entity-component system. Supports scripting in Rust, Java, and Python.

2. **A studio editor** — a desktop app where you place assets on a canvas, build scenes, and forward your scripts straight to whatever editor you use (VS Code, JetBrains, anything). Also pure Rust — UI is [GPUI](https://www.gpui.rs/), no HTML, no JS, no bundler.

---

## Prerequisites

You need these installed before anything else:

- [Rust](https://rustup.rs/) via `rustup`. The repo pins **nightly** in `rust-toolchain.toml` (GPUI uses `cold_path`, an unstable intrinsic) — `rustup` reads that automatically.
- On macOS: **full Xcode** (App Store), not just the Command Line Tools. GPUI compiles Metal shaders at build time via `xcrun metal`, which only ships in the full Xcode bundle. After install, run `sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer`.
- On Linux: GPUI's system deps — `libxkbcommon`, `libwayland`, `libssl`, `pkg-config` (see the [GPUI README](https://github.com/zed-industries/zed/tree/main/crates/gpui)).
- On Windows: Microsoft Visual C++ Build Tools.

No Node.js. No npm. No Tauri.

---

## Installation

```bash
# 1. Clone the repo
git clone <repo-url>
cd engine-dev

# 2. Run it. (First build pulls GPUI from the Zed repo — slow, then cached.)
cargo run -p studio
```

The studio window opens. You're ready to build.

---

## Using the Studio

**Open a project**

Hit `Open Project...` and point it at any folder. The asset browser populates automatically.

**Place assets on the canvas**

Click any image in the asset browser — a banner appears on the canvas. Click anywhere on the canvas to drop the asset there as a new entity. Select it by clicking. The Inspector on the right lets you nudge transform values with `+`/`−` buttons.

**Write scripts**

Click any `.rs`, `.java`, or `.py` file in the asset browser — it opens in your configured external editor.

To change which editor opens your files: `Settings` in the menu bar.

**Save your scene**

`Save Scene` — saves to a `.ndscene` file (plain JSON) you can reload later.

---

## Project Structure

```
engine-dev/
├── engine/      ← Rust runtime crate (ECS, renderer, physics, audio, scripting)
├── studio/      ← Pure-Rust editor (GPUI)
│   └── src/
│       ├── model.rs       ← entity/component/asset types
│       ├── services/      ← filesystem, scene I/O, settings, editor launch
│       ├── state.rs       ← central studio state
│       └── ui/            ← menu bar, panels, modals
└── progress.md  ← Build log and technical decisions
```

---

## Roadmap

- [x] Studio editor — canvas, asset browser, inspector, scene save/load
- [x] Script file forwarding to external editor
- [x] Pure-Rust frontend (GPUI) — no JS/TS/HTML
- [ ] Engine runtime — wgpu sprite renderer, winit event loop
- [ ] Entity Component System — hecs
- [ ] 2D physics — rapier2d (platformer collisions, triggers)
- [ ] Tilemap — load Tiled `.tmj` maps, chunked rendering
- [ ] Sprite animation — spritesheet frame sequencer + state machine
- [ ] Camera — follow, lerp, screen shake, bounds
- [ ] Audio — rodio (SFX + looping BGM)
- [ ] Platformer controller — coyote time, jump buffer, wall-slide
- [ ] Scene / room manager — metroidvania-style room loading
- [ ] Turn-based battle FSM — for RPG combat
- [ ] Dialogue system — branching text, portraits
- [ ] Python scripting — PyO3
- [ ] Java scripting — JNI / GraalVM
- [ ] Studio ↔ Engine live connection (preview an actual engine viewport inside the studio canvas)

---

## Tech Stack

| Layer | Technology |
|---|---|
| Engine core | Rust |
| Rendering | wgpu |
| Physics | rapier2d |
| Audio | rodio |
| ECS | hecs |
| Python bridge | PyO3 |
| Studio UI | [GPUI](https://www.gpui.rs/) (Zed's UI framework) |
| File dialogs | rfd |
| Scene format | JSON (`.ndscene`) |

---

Built by the NeuDel-II team — NEU Project 3.

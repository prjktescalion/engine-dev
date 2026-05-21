# NeuDel-II

A game engine built from scratch, targeting 2D and 2.5D games — platformers, metroidvanias, and Paper Mario-style turn-based RPGs.

No forks, no shortcuts — just raw Rust, a custom editor, and full control over every system. We're building this because we want to actually *understand* what's happening under the hood. Real rendering, real physics, real scripting — not a black box.

---

## What it is

**NeuDel-II** is two things:

1. **A game engine** — written in Rust. Handles rendering, physics, audio, and a full entity-component system. Supports scripting in Rust, Java, and Python.

2. **A studio editor** — a desktop app where you place assets on a canvas, build scenes, and forward your scripts straight to whatever editor you use (VS Code, JetBrains, anything).

Right now the studio is live. The engine runtime is being built next.

---

## Prerequisites

You need these installed before anything else:

- [Rust](https://rustup.rs/) — `rustup` is the easiest way
- [Node.js](https://nodejs.org/) v18 or higher
- On macOS: Xcode Command Line Tools (`xcode-select --install`)
- On Linux: `webkit2gtk`, `libgtk-3-dev` (check [Tauri prerequisites](https://tauri.app/start/prerequisites/))
- On Windows: Microsoft Visual C++ Build Tools

---

## Installation

```bash
# 1. Clone the repo
git clone <repo-url>
cd engine-dev

# 2. Install frontend dependencies
cd studio
npm install

# 3. That's it. Run it.
npm run tauri dev
```

The studio window opens. You're ready to build.

---

## Using the Studio

**Open a project**

Hit `File → Open Project` and point it at any folder. It can be a new empty folder or an existing project. The asset browser populates automatically.

**Place assets on the canvas**

Drag any image (`.png`, `.jpg`) from the asset browser at the bottom onto the dark canvas. That's your first entity. Click it to select it, drag it to move it, edit the numbers in the Inspector panel to get precise control.

**Write scripts**

Right-click any `.rs`, `.java`, or `.py` file in the asset browser and hit **Open in Editor**. It opens directly in your configured editor. Double-clicking works too.

To change which editor opens your files: `Project → Settings`.

**Save your scene**

`File → Save Scene` — saves everything to a `.ndscene` file you can reload later.

---



## Project Structure

```
engine-dev/
├── engine/      ← Rust core (ECS, renderer, physics, audio, scripting)
├── studio/      ← Desktop editor (Tauri + React + PixiJS)
└── progress.md  ← Detailed build log and technical decisions
```

---

## Roadmap

- [x] Studio editor — canvas, asset browser, inspector, scene save/load
- [x] Script file forwarding to external editor
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
- [ ] Studio ↔ Engine live connection

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
| Editor shell | Tauri v2 |
| Editor UI | React + TypeScript |
| Canvas | PixiJS v8 |
| State | Zustand |

---

Built by the NeuDel-II team — NEU Project 3.

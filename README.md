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
  - On Xcode 26+ the Metal compiler is a separate download. If the studio build fails with `cannot execute tool 'metal' due to missing Metal Toolchain`, run `xcodebuild -runFirstLaunch` followed by `xcodebuild -downloadComponent MetalToolchain` (~700 MB).
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

**See it work in 5 seconds**

`Spawn Demo` (in the menu bar) drops three entities with velocities into the scene. Press `▶ Play`. The engine integrates positions at ~60 Hz, bounces them off the world bounds, and the canvas re-renders each tick. The pill on the right of the menu bar shows entity count and live ticks-per-second. `■ Stop` reverts to the authored transforms — editing isn't destructive.

**Open a project**

Hit `Open Project...` and point it at any folder. The asset browser populates automatically.

**Place assets on the canvas**

Click any image in the asset browser — a banner appears on the canvas. Click anywhere on the canvas to drop the asset there as a new entity. Select it by clicking. The Inspector on the right lets you nudge transform values with `+`/`−` buttons, and you can add a Velocity component (`+ Velocity`) to make the entity move when you press Play.

**Write scripts**

Click any `.rs`, `.java`, or `.py` file in the asset browser — it opens in your configured external editor.

To change which editor opens your files: `Settings` in the menu bar.

**Save your scene**

`Save Scene` — saves to a `.ndscene` file (plain JSON) you can reload later.

---

## Engine Core — ECS

The runtime's heart is a **from-scratch sparse-set ECS** (`engine/src/ecs/`). We evaluated Bevy and hecs and decided against both — Bevy alone adds minutes of build time and hundreds of dependencies for an engine whose whole point is understanding every layer ourselves.

Three pieces, in the EnTT style:

| Module | What it does |
|---|---|
| `ecs/entity.rs` | Generational entity ids + free-list allocator. Spawn/despawn are O(1); a stale handle can never alias a recycled slot — every lookup checks the generation. |
| `ecs/sparse_set.rs` | Per-component storage: dense `Vec<T>` (contiguous, cache-friendly to iterate) + sparse index map (O(1) insert/get/remove, swap-pop removal). |
| `ecs/world.rs` | A fixed roster of typed stores (name, transform, velocity, sprite) plus the movement system, which walks only entities that actually have a velocity. |

Why it matters: the old wireframe core (`Vec<Entity>` + linear `find`) made the studio's per-frame transform write-back O(n²). Measured on the bundled benchmark:

```bash
cargo run --release -p engine --example ecs_bench
```

| workload (10k entities) | naive `Vec<Entity>` | sparse-set | speedup |
|---|---|---|---|
| spawn + init | 40.7 ms | 0.8 ms | ~52× |
| tick ×600 | 6.4 ms | 5.6 ms | ~1.1× |
| per-id write-back ×60 frames | 1814.6 ms | 1.1 ms | **~1700×** |

At 100k entities the naive write-back doesn't finish in reasonable time; the sparse-set core does it in ~11 ms. The unit tests (`cargo test -p engine`) cover allocator recycling, stale-handle safety, swap-remove bookkeeping, and the bounce integrator.

---

## Engine Core — Rendering

The render core (`engine/src/renderer/`, behind the `render` cargo feature) is **winit + wgpu with hand-written WGSL** — the runtime owns its surface and shaders rather than extending gpui's paint system, so shading means real fragment shaders and a game binary doesn't link the editor framework. The full decision (including why wgpu clears the bar that Bevy didn't) is in `progress.md`. wgpu translates the WGSL at runtime, so the runtime never needs the Xcode Metal Toolchain.

| Module | What it does |
|---|---|
| `renderer/atlas.rs` | CPU SDF rasterizer + shelf packer — circles/rounded-rects/capsules become one R8 coverage texture at startup. No image assets. |
| `renderer/sprite.rs` | Instanced sprite batch: one draw call, quad synthesized in the vertex shader, design-space coordinates. |
| `renderer/lcd.rs` | LCD panel post-process (fragment shader): gradient, vignette, dot-matrix grain, posterization. |
| `renderer/gpu.rs` | Device/swapchain orchestration; two passes per frame (sprites → ink texture → LCD → screen). |

**Try it — the Game & Watch demo:**

```bash
cargo run -p ball
```

A standalone runtime binary (no studio, no gpui) running **Ball (1980)** on the engine ECS: two balls bounce along a 7-station arc on a discrete tick clock, ←/→ set the hand pose, catches score, three misses ends it, and the clock speeds up as you score. Every LCD segment is always drawn — unlit ones as faint ghosts, the signature unlit-LCD look. Space restarts, Esc quits.

Renderer throughput (`cargo run --release -p engine --features render --example sprite_bench`, Apple M2, headless): ~52M sprites/sec at 100k sprites/frame (1.9 ms); the demo's ~36 segments cost 0.1 ms.

---

## Project Structure

```
engine-dev/
├── engine/      ← Rust runtime crate
│   ├── src/ecs/       ← from-scratch sparse-set ECS (entity allocator, storage, world)
│   ├── src/renderer/  ← wgpu render core: shape atlas, sprite batch, LCD shader (feature "render")
│   ├── src/…          ← physics, audio, scripting (stubs)
│   └── examples/      ← ecs_bench, sprite_bench
├── ball/        ← standalone Game & Watch "Ball" runtime (winit + engine, no studio)
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
- [x] Engine runtime — ECS-lite (Vec<Entity>), 60Hz tick, Transform + Velocity integrator
- [x] Studio ↔ Engine live connection — Play/Pause/Stop drives the engine; canvas reflects live simulation
- [x] Entity Component System — custom sparse-set ECS, built from scratch (no Bevy, no hecs): generational entity ids, O(1) component access, cache-friendly system iteration. See [Engine Core](#engine-core--ecs).
- [x] Engine renderer — wgpu sprite renderer (instanced batch, procedural SDF atlas) + custom WGSL shading (LCD post-process). See [Rendering](#engine-core--rendering).
- [x] Standalone game runtime — `cargo run -p ball`: Game & Watch *Ball* on the engine ECS, fixed-timestep, ghost-segment LCD look
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
| Rendering | wgpu + custom WGSL (runtime-compiled — no Metal Toolchain needed) |
| Windowing (runtime) | winit |
| Physics | rapier2d |
| Audio | rodio |
| ECS | custom sparse-set (from scratch — no Bevy/hecs) |
| Python bridge | PyO3 |
| Studio UI | [GPUI](https://www.gpui.rs/) (Zed's UI framework) |
| File dialogs | rfd |
| Scene format | JSON (`.ndscene`) |

---

Built by the NeuDel-II team — NEU Project 3.

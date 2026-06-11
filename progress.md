# NeuDel-II — Build Progress

## Project Overview

Custom game engine built from scratch for NEU Project 3.

**Goal:** Full ownership of the tech stack — no forking. Deep understanding of engine internals with a focus on security, performance, and multi-language scripting support.

**Languages:**
- **Engine core** → Rust
- **Scripting** → Rust (native), Java (planned via JNI), Python (planned via PyO3)

---

## Repository Layout

```
engine-dev/
├── engine/                        # Rust core engine crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                 # Module declarations
│       ├── ecs/mod.rs             # Entity Component System (stub)
│       ├── renderer/mod.rs        # wgpu renderer (stub)
│       ├── physics/mod.rs         # rapier2d physics (stub)
│       ├── audio/mod.rs           # rodio audio (stub)
│       └── scripting/mod.rs       # Rust/Java/Python bridges (stub)
├── studio/                        # Tauri + React editor
│   ├── src-tauri/                 # Rust/Tauri backend
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   ├── capabilities/
│   │   │   └── default.json       # Tauri v2 permission grants
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs             # Plugin registration + command handlers
│   │       └── commands/
│   │           ├── mod.rs
│   │           ├── fs.rs          # list_dir, create_script, read_image
│   │           ├── editor.rs      # open_in_editor
│   │           ├── scene.rs       # save_scene, load_scene
│   │           └── settings.rs    # get_settings, save_settings
│   └── src/                       # React/TypeScript frontend
│       ├── main.tsx               # App entry, ErrorBoundary, index.css
│       ├── App.tsx
│       ├── App.css                # All component styles (dark theme)
│       ├── index.css              # CSS custom properties + resets
│       ├── types/
│       │   └── engine.ts          # Entity, Component, Asset, Settings types
│       ├── store/
│       │   ├── sceneStore.ts      # Zustand — entities, selection, scene
│       │   ├── assetStore.ts      # Zustand — project root, asset tree
│       │   └── settingsStore.ts   # Zustand — editor prefs, synced to disk
│       └── components/
│           ├── ErrorBoundary.tsx
│           ├── layout/
│           │   └── AppShell.tsx   # 4-panel IDE layout
│           ├── menubar/
│           │   └── MenuBar.tsx    # File / Project menus
│           ├── canvas/
│           │   ├── usePixiApp.ts  # PixiJS Application lifecycle hook
│           │   └── SceneCanvas.tsx# Grid, drag-drop, entity select/move
│           ├── hierarchy/
│           │   └── HierarchyPanel.tsx # Entity tree, rename, delete
│           ├── inspector/
│           │   └── InspectorPanel.tsx # Transform fields, open scripts
│           ├── assets/
│           │   ├── AssetBrowser.tsx   # Project file tree, tabbed panel
│           │   └── AssetItem.tsx      # Draggable item, context menu
│           └── settings/
│               └── EditorSettings.tsx # Editor preference modal
├── Cargo.toml                     # Workspace root (engine crate)
├── progress.md                    # ← this file
└── .gitignore
```

---

## Phase 1 — Studio Editor ✅ COMPLETE

### What was built

**Tauri backend commands** (`src-tauri/src/commands/`)

| Command | File | What it does |
|---|---|---|
| `list_dir` | `fs.rs` | Recursive directory scan → typed `Asset[]` tree (dirs first, skips `target/`, `node_modules/`, dotfiles) |
| `create_script` | `fs.rs` | Writes a language-appropriate script stub (Rust / Java / Python) |
| `read_image` | `fs.rs` | Reads image from disk, returns base64 data URL (avoids Tauri asset-protocol config) |
| `open_in_editor` | `editor.rs` | Shells out to `code`, `idea`, or a custom binary with the file path |
| `save_scene` | `scene.rs` | Serialises scene JSON → `.ndscene` file |
| `load_scene` | `scene.rs` | Reads `.ndscene` → JSON string for React to parse |
| `get_settings` | `settings.rs` | Loads `settings.json` from Tauri app-data dir |
| `save_settings` | `settings.rs` | Writes `settings.json` to Tauri app-data dir |

**React frontend panels**

```
┌───────────────────────────────────────────────────────────────────┐
│  NeuDel-II  File  Project                          [scene name]   │  ← MenuBar
├─────────────┬─────────────────────────────────┬───────────────────┤
│ Hierarchy   │         Scene Canvas            │    Inspector      │
│             │  (PixiJS — dark grid)           │  Transform:       │
│ ◈ Entity 1  │  drag image from Assets → drop  │    X  Y           │
│ ◈ Entity 2  │  click sprite → select          │    Scale X  Y     │
│             │  drag sprite → reposition       │    Rotation       │
│             │                                 │  Sprite: path     │
│             │                                 │  Scripts: ↗ open  │
├─────────────┴─────────────────────────────────┴───────────────────┤
│  [ Assets ]  [ Console ]                                          │  ← Bottom panel
│  📁 project-root/                                                 │
│    🖼 sprite.png   (draggable to canvas)                          │
│    📄 player.rs    (right-click → Open in Editor)                 │
│    📄 ai.py        (double-click → Open in Editor)               │
└───────────────────────────────────────────────────────────────────┘
```

**Key behaviours**

- Drop an image from the asset browser → spawns an entity on the canvas at the drop position
- Click a sprite → highlights it in the Hierarchy, populates the Inspector
- Drag a selected sprite → moves it, Inspector X/Y updates live
- Edit Inspector fields (X, Y, scaleX, scaleY, rotation) → PixiJS sprite syncs immediately
- Right-click `.rs` / `.java` / `.py` in Asset Browser → **Open in Editor** (or double-click)
- Inspector "↗" button on a script component → opens the file in the configured editor
- **File → Open Project** → folder picker → populates Asset Browser
- **File → Save Scene / Load Scene** → persists entity layout to `.ndscene` JSON
- **Project → Settings** → modal to switch editor (VS Code / JetBrains / custom path)
- Settings are persisted to the OS app-data directory across restarts
- Grid redraws on window resize via `ResizeObserver`
- `ErrorBoundary` wraps the whole app — crashes show a readable error + Retry button instead of a blank page

**Scene file format** (`.ndscene`)

```json
{
  "version": 1,
  "name": "My Scene",
  "entities": [
    {
      "id": "a1b2c3d4",
      "name": "Player",
      "components": [
        { "type": "transform", "x": 320, "y": 240, "scaleX": 1, "scaleY": 1, "rotation": 0 },
        { "type": "sprite", "assetPath": "/path/to/player.png", "dataUrl": "data:image/png;base64,..." },
        { "type": "script", "filePath": "/path/to/player.rs", "lang": "rust" }
      ]
    }
  ]
}
```

### Tech stack decisions

| Decision | Choice | Reason |
|---|---|---|
| Desktop shell | Tauri v2 | Rust backend, lighter than Electron, native OS file dialogs |
| Canvas | PixiJS v8 | Mature 2D renderer, WebGL/WebGPU, good drag-drop support |
| State | Zustand | Minimal boilerplate, works well outside React tree |
| Image loading | Rust `read_image` → base64 data URL | Avoids Tauri asset-protocol scope config; simpler and portable |
| File dialogs | `@tauri-apps/plugin-dialog` from frontend | Cleanest API, no extra Rust command needed |
| Script opening | `std::process::Command` in Rust backend | No shell-plugin permissions needed; works for any editor |
| Styling | Plain CSS custom properties | No framework overhead, dark theme fully controlled |

### Bugs fixed during development

1. **Blank page** — `index.css` was not imported in `main.tsx`, so all CSS variables (`--bg-0`, `--text`, etc.) were undefined. Layout rendered with no colors, appearing blank.
2. **PixiJS StrictMode race** — React StrictMode runs cleanup+remount once in dev. The first app's async `init().then()` resolved after its cleanup ran, appending a dead canvas. Fixed with a `cancelled` flag.
3. **Tauri capabilities** — Used non-existent permission names (`fs:allow-create-dir`). Simplified to only `dialog:default` since all file I/O goes through custom Rust commands.
4. **Workspace conflict** — `studio/src-tauri` was being picked up by the root `Cargo.toml` workspace. Fixed by adding `[workspace]` to the Tauri crate's `Cargo.toml`.

---

## Genre Focus — 2D / 2.5D

The engine targets:
- **Platformers** — tight character control, precise collision, coyote time, jump buffering
- **Metroidvanias** — room-based world map, ability gates, map tracking, save points
- **2.5D (Paper Mario style)** — 2D gameplay with layered depth: sprites sorted by Y/Z, billboard sprites, perspective parallax
- **Turn-based RPGs** — battle state machine, dialogue trees, inventory, party system

This scopes out: 3D mesh rendering, skeletal animation, deferred lighting, 3D physics. Everything is built around sprite batching, 2D physics, and strong scene/state management.

---

## Phase 2 — Engine Runtime ⏳ IN PROGRESS

### 2026-06 — Wireframe runtime + studio play loop

The `engine` crate went from stubs to a live runtime: an `Engine` owns a `World`,
ticks at ~60 Hz, and the (now pure-GPUI) studio drives it — `Spawn Demo` seeds
moving entities, `▶ Play` compiles the authored scene into the engine,
`■ Stop` reverts to authored transforms. Canvas re-renders every tick with a
live TPS pill. (See README "Using the Studio".)

### 2026-06-10 — Custom sparse-set ECS core (Bevy/hecs dropped)

**Decision:** the roadmap said "swap the `Vec<Entity>` wireframe for hecs."
We instead built the ECS from scratch and dropped the planned hecs/Bevy
dependency entirely. Bevy is very heavy after building (hundreds of transitive
deps, minutes of compile time), and the whole point of NeuDel-II is owning the
internals. The result is ~3 small modules with zero new dependencies.

**Architecture** (`engine/src/ecs/`, EnTT-style sparse-set ECS):

| Module | Design |
|---|---|
| `entity.rs` | `EntityId { index, generation }` + free-list allocator. O(1) spawn/despawn (despawn didn't exist before). Stale handles can't alias a recycled slot — every lookup validates the generation. |
| `sparse_set.rs` | `SparseSet<T>`: dense `Vec<T>` + parallel owner list + sparse index map. O(1) insert/get/remove; removal swap-pops so dense storage never has holes; insert evicts stale-generation leftovers so zombies can't survive in dense storage. |
| `world.rs` | Fixed roster of typed stores — `names`, `transforms`, `velocities`, `sprites`. No `TypeId`-erased registry: at this scale a named field per store is simpler *and* faster, and adding a component type is a three-line change. The movement system iterates the velocity store densely (only moving entities, contiguous memory) and probes transforms O(1). |

**Why it matters:** the old core's `get()` was a linear `find`, so the studio's
per-frame transform write-back (`step_engine`) was O(n²). Benchmark
(`cargo run --release -p engine --example ecs_bench`; the old core is inlined
in the example as the baseline):

| workload | naive 10k | sparse-set 10k | naive 100k | sparse-set 100k |
|---|---|---|---|---|
| spawn + init | 40.7 ms | 0.8 ms (52×) | 3083 ms | 6.2 ms (494×) |
| tick ×600 | 6.4 ms | 5.6 ms (1.1×) | 63.4 ms | 58.8 ms (1.1×) |
| write-back ×60 | 1814 ms | 1.1 ms (**1710×**) | unfeasible (O(n²)) | 10.7 ms |

Ticking was already linear in the old core, hence the modest 1.1× there — the
win is lookup-heavy paths, which is exactly what the studio bridge does every
frame.

**API change:** `World` no longer exposes a public `Vec<Entity>` with
`Option<T>` fields. Component access goes through typed accessors —
`transform(id)` / `transform_mut(id)`, `set_velocity(id, v)`,
`set_sprite(id, s)`, `name(id)`, plus `despawn(id)` / `is_alive(id)`.
`studio/src/state.rs` (`compile_into_engine`, `step_engine`) was migrated; the
`Engine` facade (`spawn` / `reset` / `tick` / `entity_count`) was unchanged.

**Tests:** 10 unit tests (`cargo test -p engine`) — allocator recycling +
generation bumps, double-despawn rejection, swap-remove bookkeeping,
stale-handle misses, zombie eviction, default components on spawn, the bounce
integrator, and dead-entity no-ops.

**Toolchain gotcha (macOS / Xcode 26):** the studio build broke with
`cannot execute tool 'metal' due to missing Metal Toolchain` — Xcode 26 ships
the Metal compiler as a separate component. Fix: `xcodebuild -runFirstLaunch`
(repairs first-launch components; the download fails without it), then
`xcodebuild -downloadComponent MetalToolchain` (~700 MB). Documented in the
README prerequisites.

### Remaining Phase 2 plan

Target genres drive the priority order of systems.

### 2A — Core loop
| System | Library | Notes |
|---|---|---|
| Windowing + event loop | `winit` | Drive the game loop, handle input events |
| 2D renderer | `wgpu` | Sprite batcher — textured quads, Z-sorted draw calls, WebGL-level feature set |
| ECS | custom (done ✅) | From-scratch sparse-set ECS — see the 2026-06-10 entry above. Component roster grows with: Animator, RigidBody2D, Collider2D, Script |
| Input | `winit` events | Keyboard, mouse, gamepad (via `gilrs`) — action mapping layer on top |
| Asset loader | custom | Load PNG/JPG textures, audio files, tilemaps, scene files from disk |

### 2B — 2D game systems
| System | Library | Notes |
|---|---|---|
| Physics | `rapier2d` | Rigidbodies, colliders (rect, circle, capsule), collision events |
| Tilemap | custom | Load Tiled-format `.tmj` JSON maps, render chunked tile layers |
| Sprite animation | custom | Spritesheet frame sequencer, state machine (idle → walk → jump) |
| Camera | custom | Follow target, smooth lerp, screen shake, bounds clamping |
| Parallax | custom | Multiple background layers at different scroll speeds |
| Audio | `rodio` | SFX one-shots, looping BGM, volume/pitch control |

### 2C — Genre-specific systems
| System | Notes | Needed for |
|---|---|---|
| Scene / room manager | Load/unload rooms, persistent world state, door transitions | Metroidvania |
| Ability gate system | Track unlocked abilities, open/lock paths | Metroidvania |
| Platformer controller | Coyote time, jump buffer, wall-slide, run | Platformer |
| Turn-based battle FSM | Phases: player turn → enemy turn → resolve → reward | RPG |
| Dialogue system | Text box, speaker portrait, branching choices | RPG / all |
| Inventory + items | Item registry, pickup, equip, use | RPG |
| Save system | Serialise world state to disk | All |

### 2D — 2.5D rendering tricks
| Technique | How |
|---|---|
| Y-sort draw order | Sprites further up the screen draw behind sprites lower down — gives depth illusion |
| Z-layer system | Background / world / foreground / UI layers, each with its own sort pass |
| Billboard sprites | Sprites always face the camera — Paper Mario effect in a 3D-positioned scene |
| Shadow blobs | Simple ellipse shadow under characters, scaled by height |
| Normal map lighting | Optional: `wgpu` pixel shader reads a normal map texture for 2D lit sprites |

---

## Phase 3 — Rendering & Standalone Runtime ✅ COMPLETE

### 2026-06-10 — Decision: renderer architecture (gpui paint vs. own surface)

**The question:** Phase 3 needs textured sprites, alpha blending, and at least
one real shading effect (an LCD-panel look for the Game & Watch demo), hosted
in a standalone runtime binary that doesn't depend on the studio.

**Option A — extend gpui's paint system.** Reuses the window/event/Metal
plumbing we already ship in the studio; fastest path. Rejected because (1)
gpui has no public hook for custom fragment shaders — "shading" would mean
stacks of alpha-blended quads or forking gpui, and the whole point of this
phase is first-class shading; (2) a standalone game runtime would link the
entire editor UI framework (gpui pulls Zed's tree via git) for a window and a
draw loop; (3) it re-inherits the gpui-specific gotchas (`font-kit`
NoopTextSystem, build-time Metal shader compilation).

**Option B — the runtime owns its surface and shaders.** Chosen, as
`winit` (window + events) + `wgpu` (GPU API) + custom WGSL. Within B we
weighed raw Metal via `objc2` against `wgpu`, by the same bar as the Bevy
decision:

- Raw Metal is the maximal "own every layer" play, but it's thousands of
  lines of unsafe Objective-C glue, macOS-only (the README promises
  Linux/Windows), and re-couples builds to the Xcode Metal Toolchain.
- The Bevy rejection was about *frameworks that own your architecture*.
  `wgpu` is a portable GPU API wrapper — batching, atlases, render passes,
  and shaders all stay ours. It's the same category of dependency as
  `rapier2d` or `rodio`, and `wgpu`/`winit` have been the declared renderer
  plan in `Cargo.toml` and the roadmap since Phase 1.
- Bonus: wgpu's naga translates WGSL → MSL **at runtime**, so the runtime
  binary has no build-time `xcrun metal` dependency — the Metal Toolchain
  gotcha disappears for the game runtime.

**Shape of the implementation:** the render core lives in the engine crate
(`engine/src/renderer/`) behind a `render` cargo feature so the studio (which
depends on `engine`) doesn't compile wgpu. The runtime binary depends on
`engine` with `features = ["render"]` plus `winit`. The LCD look (ghost
segments, gradient, vignette, posterize, dot grain) is a fullscreen
post-process fragment shader over an offscreen "ink" target, not a stack of
quads.

### 2026-06-10 — Render core + standalone Game & Watch *BALL* runtime

**Render core** (`engine/src/renderer/`, behind the `render` feature):

| Module | Design |
|---|---|
| `atlas.rs` | CPU-side SDF rasterizer + shelf packer. Shapes (circle / rounded rect / rotated capsule) are rasterized at startup into one R8 coverage texture with 1px antialiased edges and 2px bleed padding — **no image assets anywhere**. Pure math, unit-tested (packing overlap, UV↔pixel mapping, rotated-capsule bounds). |
| `sprite.rs` | Instanced batch pass: one draw call per frame, the quad synthesized in the vertex shader from `vertex_index` (no vertex buffer), per-sprite rect/UV/tint in a grow-on-demand instance buffer. Sprites are authored in design-space units; a uniform maps to NDC. Target-agnostic — draws into any view. |
| `lcd.rs` + `shaders/lcd.wgsl` | The shading effect this phase exists for: a fullscreen-triangle post pass compositing the sprite pass's offscreen "ink" over a procedural LCD panel — greenish-grey vertical gradient, soft vignette, 3px dot-matrix gutter grain, 48-level posterization. Segment ghosting is *not* done here — ghosts are just unlit sprites at α≈0.085 — but the panel look is pure fragment shader. |
| `gpu.rs` | Surface-owning orchestrator: adapter/device/swapchain setup (prefers a non-sRGB format so shader output is literal), ink-target lifecycle on resize, two-pass frame submission. Winit-agnostic via `wgpu::SurfaceTarget`. |

As predicted in the decision entry: WGSL is translated by naga **at runtime**,
so neither building nor running the `ball` binary touches `xcrun metal` — the
Metal Toolchain gotcha is studio-only now.

**Standalone runtime** (`ball/`, `cargo run -p ball` — no studio, no gpui):

- `main.rs` — winit `ApplicationHandler`, 960×640 fixed window, fixed-timestep
  accumulator decoupled from vsync. The tick interval is re-read every step
  because scoring shortens it (420 ms → 140 ms floor, −6 ms per point).
- `game.rs` — *Ball* (1980) rules on the engine ECS: two balls bounce along a
  7-station arc one station per tick, out of phase (arrivals every 3 ticks in
  an R-R-L-L rhythm). ←/→ set the hand pose; right pose at the arrival
  station = catch (score++), wrong = miss; three misses = game over; Space
  restarts. Balls are world entities whose `Transform` mirrors their station;
  the game-specific `BallState` component lives in a *game-owned*
  `SparseSet<BallState>` keyed by the same `EntityId`s — exactly how the
  fixed-roster world was meant to be extended from outside.
- `layout.rs` — the LCD segment map: 7 arc stations, the juggler (head, torso,
  ground, two exclusive arm poses with hand cues), two 7-segment score digits
  (encoding table), three miss markers. **Every segment renders every frame**;
  unlit ones at ghost alpha — the unlit-LCD look the phase demanded.

**Tests:** 12 new (`cargo test -p ball`, plus 4 atlas tests in the engine) —
catch/miss resolution, the R-R-L-L rhythm, game-over freeze, speed-ramp floor,
restart semantics, station↔Transform sync, station geometry, digit-table
sanity, ghost-vs-lit sprite counts, UV propagation.

**Benchmark** (`cargo run --release -p engine --features render --example
sprite_bench`, headless offscreen target, Apple M2, 240-frame average,
instance list rebuilt every frame):

| sprites/frame | ms/frame | throughput |
|---|---|---|
| 100 | 0.104 | 1.0M/s |
| 1,000 | 0.104 | 9.6M/s |
| 10,000 | 0.230 | 43.5M/s |
| 100,000 | 1.926 | 51.9M/s |

Flat to ~1k sprites (fixed submit overhead dominates), then ~52M sprites/sec
asymptotic — the Ball demo's ~36 segments are 0.1 ms. The single-draw-call
design has headroom for tilemaps and particle-heavy scenes phases away.

**Gotchas:** none new. The swapchain format is deliberately non-sRGB
(`find(|f| !f.is_srgb())`) so the LCD shader's authored colors land on screen
unmodified; if a future target only offers sRGB formats the constants will
need a gamma pass.

## Phase 4 — Scripting Layer (NOT STARTED)

- **Rust** — compiled as dynamic libraries, hot-reloaded via `libloading`
- **Python** — embedded CPython via `pyo3`; good for AI, dialogue logic, rapid prototyping
- **Java** — JNI or GraalVM Polyglot; good for complex game logic and team members with Java background
- All scripts expose `init()` and `update(delta: f64)`
- Engine exposes a stable API surface to all three runtimes (entity query, component read/write, event emit)

---

## How to Run

```bash
# Start the studio editor (pure Rust — no npm since the GPUI rewrite)
cd engine-dev
cargo run -p studio

# Engine crate: tests and the ECS benchmark
cargo test -p engine
cargo run --release -p engine --example ecs_bench

# Game & Watch BALL demo (standalone runtime — no studio)
cargo run -p ball
cargo test -p ball
cargo run --release -p engine --features render --example sprite_bench
```

**First steps in the studio:**
1. File → Open Project → pick any folder containing images (`.png`, `.jpg`) and scripts (`.rs`, `.java`, `.py`)
2. Drag an image from the Asset browser (bottom panel) onto the canvas
3. Click the sprite to select it — Inspector shows Transform fields
4. Drag the sprite to reposition; edit numbers in Inspector for precise values
5. Right-click a script file in the Asset browser → **Open in Editor**
6. Project → Settings to change which editor is used

---

## Dependencies

### Rust workspace (`engine/`)
| Crate | Version | Purpose |
|---|---|---|
| `wgpu` | 22 | GPU rendering (future) |
| `winit` | 0.30 | Windowing (future) |
| `rapier2d` | 0.22 | Physics (future) |
| `rodio` | 0.20 | Audio (future) |
| `pyo3` | 0.25 | Python scripting bridge (future) |

### Tauri backend (`studio/src-tauri/`)
| Crate | Version | Purpose |
|---|---|---|
| `tauri` | 2 | Desktop app framework |
| `tauri-plugin-opener` | 2 | Open URLs/files |
| `tauri-plugin-fs` | 2 | Filesystem plugin (registered) |
| `tauri-plugin-dialog` | 2 | Native file/folder dialogs |
| `base64` | 0.22 | Encode images for frontend |
| `serde` / `serde_json` | 1 | JSON serialisation |

### Frontend (`studio/`)
| Package | Purpose |
|---|---|
| `pixi.js` | 2D WebGL canvas renderer |
| `zustand` | Lightweight state management |
| `@tauri-apps/api` | Tauri IPC (`invoke`) |
| `@tauri-apps/plugin-dialog` | File/folder picker dialogs |
| `@tauri-apps/plugin-fs` | (installed, reserved for future use) |
| `@tauri-apps/plugin-shell` | (installed, reserved for future use) |
| `react` + `react-dom` | UI framework |
| `typescript` | Type safety |
| `vite` | Dev server + bundler |

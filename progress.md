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

## Phase 2 — Engine Runtime (NOT STARTED)

Target genres drive the priority order of systems.

### 2A — Core loop
| System | Library | Notes |
|---|---|---|
| Windowing + event loop | `winit` | Drive the game loop, handle input events |
| 2D renderer | `wgpu` | Sprite batcher — textured quads, Z-sorted draw calls, WebGL-level feature set |
| ECS | `hecs` | Archetypes store: Transform2D, Sprite, Animator, RigidBody2D, Collider2D, Script |
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

## Phase 3 — Scripting Layer (NOT STARTED)

- **Rust** — compiled as dynamic libraries, hot-reloaded via `libloading`
- **Python** — embedded CPython via `pyo3`; good for AI, dialogue logic, rapid prototyping
- **Java** — JNI or GraalVM Polyglot; good for complex game logic and team members with Java background
- All scripts expose `init()` and `update(delta: f64)`
- Engine exposes a stable API surface to all three runtimes (entity query, component read/write, event emit)

---

## How to Run

```bash
# Start the studio editor (dev mode)
cd engine-dev/studio
npm run tauri dev

# Check the engine crate compiles
cd engine-dev
cargo check -p engine
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
| `hecs` | 0.10 | ECS (future) |
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

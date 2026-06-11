//! Render core — from-scratch sprite renderer with a custom shading pass.
//!
//! Architecture (see the 2026-06-10 decision entry in `progress.md`): the
//! runtime owns its render surface via `winit` + `wgpu` with hand-written
//! WGSL — gpui stays an editor concern, and "shading" means real fragment
//! shaders, not stacked quads. Everything is behind the `render` cargo
//! feature so the studio doesn't compile wgpu.
//!
//! - [`atlas`]: CPU-side SDF rasterizer + shelf packer — shapes (circle /
//!   rounded rect / capsule) become one R8 coverage texture; no image assets.
//! - [`sprite`]: instanced batch pass — one draw call for all sprites, quad
//!   synthesized in the vertex shader, design-space coordinates.
//! - [`lcd`]: fullscreen post-process — the Game & Watch LCD panel look
//!   (gradient, vignette, dot-matrix gutter, posterize) as a fragment shader.
//! - [`gpu`]: surface-owning orchestrator (device/swapchain/frame loop);
//!   winit-agnostic via `wgpu::SurfaceTarget`.
//!
//! Throughput numbers: `cargo run --release -p engine --features render
//! --example sprite_bench` (headless — no window needed).

#[cfg(feature = "render")]
pub mod atlas;
#[cfg(feature = "render")]
pub mod gpu;
#[cfg(feature = "render")]
pub mod lcd;
#[cfg(feature = "render")]
pub mod sprite;

#[cfg(feature = "render")]
pub use atlas::{Atlas, Region, Shape};
#[cfg(feature = "render")]
pub use gpu::GpuRenderer;
#[cfg(feature = "render")]
pub use sprite::{SpriteInstance, SpritePass};

use crate::ecs::World;

/// Wireframe-era stub still embedded in [`crate::Engine`] for the studio's
/// benefit — the studio draws its canvas with gpui and only reads the world.
/// The real GPU path is [`gpu::GpuRenderer`], constructed by the runtime
/// binary that owns a window.
pub struct Renderer {
    pub clear_color: [f32; 4],
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            clear_color: [0.04, 0.05, 0.08, 1.0],
        }
    }

    pub fn draw(&mut self, _world: &World) {
        // The studio reads `world` directly; standalone runtimes use GpuRenderer.
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

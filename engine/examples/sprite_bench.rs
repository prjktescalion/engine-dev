//! Renderer throughput bench: sprites per frame through the instanced batch.
//!
//! Run with: cargo run --release -p engine --features render --example sprite_bench
//!
//! Headless — renders to an offscreen texture, no window or compositor in the
//! loop, so the numbers are the renderer's own cost: per-frame instance list
//! build (CPU), instance buffer upload, one instanced draw, submit, and a
//! blocking wait for the GPU. The Ball demo draws ~36 sprites; the interesting
//! question is how far the single-draw-call design scales past that.

use std::hint::black_box;
use std::time::Instant;

use engine::renderer::{Atlas, Shape, SpriteInstance, SpritePass};

const FRAMES: u32 = 240;
const TARGET: (u32, u32) = (960, 640);

fn main() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("no GPU adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("bench device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
        },
        None,
    ))
    .expect("device");

    let atlas = Atlas::build(
        &[
            Shape::Circle { radius: 9.0 },
            Shape::RoundedRect { width: 14.0, height: 5.0, corner: 2.0 },
            Shape::Capsule { length: 40.0, radius: 5.0, angle: 0.5 },
        ],
        256,
    );

    let target = device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("bench target"),
            size: wgpu::Extent3d {
                width: TARGET.0,
                height: TARGET.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut pass = SpritePass::new(
        &device,
        &queue,
        &atlas,
        wgpu::TextureFormat::Rgba8Unorm,
        (TARGET.0 as f32, TARGET.1 as f32),
    );

    println!(
        "sprite bench — adapter: {} | {} frames per row, {}x{} offscreen",
        adapter.get_info().name,
        FRAMES,
        TARGET.0,
        TARGET.1
    );
    println!("  {:<14} {:>12} {:>14}", "sprites/frame", "ms/frame", "sprites/sec");

    for n in [100usize, 1_000, 10_000, 100_000] {
        // Warm up buffer growth outside the timed loop.
        let warm = build_frame(&atlas, n, 0);
        let mut encoder = device.create_command_encoder(&Default::default());
        pass.draw(&device, &queue, &mut encoder, &target, &warm, true);
        queue.submit([encoder.finish()]);
        device.poll(wgpu::Maintain::Wait);

        let start = Instant::now();
        for frame in 0..FRAMES {
            // Rebuild the instance list every frame — that's the real
            // per-frame CPU path, not just a reupload of a static buffer.
            let sprites = build_frame(&atlas, n, frame);
            let mut encoder = device.create_command_encoder(&Default::default());
            pass.draw(&device, &queue, &mut encoder, &target, &sprites, true);
            queue.submit([encoder.finish()]);
            black_box(&sprites);
        }
        device.poll(wgpu::Maintain::Wait);
        let ms = start.elapsed().as_secs_f64() * 1000.0 / FRAMES as f64;
        println!(
            "  {:<14} {:>12.3} {:>14.1}M",
            n,
            ms,
            n as f64 / ms / 1000.0
        );
    }
}

/// Deterministic pseudo-scene: n sprites scattered over the target, cycling
/// through the atlas shapes, alpha varying like lit/ghost segments.
fn build_frame(atlas: &Atlas, n: usize, frame: u32) -> Vec<SpriteInstance> {
    (0..n)
        .map(|i| {
            let region = atlas.region(i % atlas.region_count());
            let h = (i as u32).wrapping_mul(2654435761).wrapping_add(frame);
            SpriteInstance {
                pos: [
                    (h % TARGET.0) as f32,
                    (h / 7 % TARGET.1) as f32,
                ],
                size: [region.size.0 as f32, region.size.1 as f32],
                uv_min: region.uv_min,
                uv_max: region.uv_max,
                color: [0.07, 0.08, 0.07, if i % 3 == 0 { 0.92 } else { 0.085 }],
            }
        })
        .collect()
}

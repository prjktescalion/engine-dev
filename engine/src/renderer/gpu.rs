//! Surface-owning renderer: device/queue/surface setup plus per-frame
//! orchestration of the two passes (sprites → ink texture → LCD post → swapchain).
//!
//! Generic over the surface target (anything wgpu accepts — the runtime hands
//! in an `Arc<winit::Window>`), so this module has no winit dependency. The
//! headless benchmark skips this type entirely and drives [`SpritePass`]
//! against an offscreen texture.

use super::atlas::Atlas;
use super::lcd::LcdPass;
use super::sprite::{SpriteInstance, SpritePass};

pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    sprites: SpritePass,
    lcd: LcdPass,
    ink_view: wgpu::TextureView,
}

impl GpuRenderer {
    /// Bring up the GPU and both passes. `design` is the logical coordinate
    /// space sprites are authored in; the window aspect should match it.
    pub fn new(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
        atlas: &Atlas,
        design: (f32, f32),
    ) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(target)?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| anyhow::anyhow!("no compatible GPU adapter"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("engine device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))?;

        // Prefer a non-sRGB swapchain so shader output is what lands on
        // screen; the LCD shader authors its colors directly.
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let ink_view = Self::make_ink(&device, config.width, config.height);
        let sprites = SpritePass::new(&device, &queue, atlas, wgpu::TextureFormat::Rgba8Unorm, design);
        let mut lcd = LcdPass::new(&device, format);
        lcd.bind_ink(&device, &ink_view, (config.width, config.height));

        Ok(Self {
            device,
            queue,
            surface,
            config,
            sprites,
            lcd,
            ink_view,
        })
    }

    fn make_ink(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("lcd ink target"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        tex.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.ink_view = Self::make_ink(&self.device, self.config.width, self.config.height);
        self.lcd
            .bind_ink(&self.device, &self.ink_view, (self.config.width, self.config.height));
    }

    /// Render one frame: batch `sprites` into the ink target, then run the
    /// LCD post pass into the swapchain.
    pub fn render(&mut self, sprites: &[SpriteInstance]) -> Result<(), wgpu::SurfaceError> {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture()?
            }
            Err(e) => return Err(e),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });
        self.sprites
            .draw(&self.device, &self.queue, &mut encoder, &self.ink_view, sprites, true);
        self.lcd.draw(&mut encoder, &view);
        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }
}

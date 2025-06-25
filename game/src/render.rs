use std::sync::Arc;

use anyhow::Context;
use winit::window::Window;

pub struct Renderer {
    pub(crate) window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>,
    is_surface_configured: bool,
}

impl Renderer {
    pub(crate) async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&Default::default());

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter.request_device(&Default::default()).await?;

        let config = surface
            .get_default_config(
                &adapter,
                window.inner_size().width.max(1),
                window.inner_size().height.max(1),
            )
            .with_context(|| "Unable to get default surface config")?;

        #[cfg(not(target_arch = "wasm32"))]
        surface.configure(&device, &config);

        Ok(Self {
            device,
            queue,
            window,
            surface,
            config,
            is_surface_configured: cfg!(not(target_arch = "wasm32")),
        })
    }

    pub(crate) fn render(&mut self, game: &crate::game::Game) -> bool {
        if !self.is_surface_configured {
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
        
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => match e {
                wgpu::SurfaceError::Outdated => {
                    log::debug!("Outdated");
                    return true
                },
                e => {
                    log::error!("{e}");
                    return false;
                }
            }
        };
        
        let view = frame.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit([encoder.finish()]);
        frame.present();

        true
    }
    
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.is_surface_configured = true;
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }
}

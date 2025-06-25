mod bindings;
pub mod buffer;
mod quad;
mod uniform;
pub mod vertex;

use std::sync::Arc;

use anyhow::Context;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::render::{
    bindings::{CameraBinder, TextureBinder},
    buffer::BackedBuffer,
    quad::QuadPipeline,
    uniform::CameraData,
    vertex::{InstanceColor2d, Vertex2d},
};

const PLAYER_COLORS: &[glam::Vec4] = &[
    glam::vec4(1.0, 0.0, 0.0, 1.0),
    glam::vec4(0.0, 1.0, 0.0, 1.0),
    glam::vec4(0.0, 0.0, 1.0, 1.0),
    glam::vec4(1.0, 1.0, 0.0, 1.0),
    glam::vec4(0.0, 1.0, 1.0, 1.0),
    glam::vec4(1.0, 0.0, 1.0, 1.0),
];

pub struct Renderer {
    pub(crate) window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>,
    is_surface_configured: bool,

    // game specific
    quad_pipeline: QuadPipeline,
    player_vertices: BackedBuffer<Vertex2d>,
    player_indices: BackedBuffer<u32>,
    player_instances: BackedBuffer<InstanceColor2d>,
    camera_buffer: BackedBuffer<CameraData>,
    camera_binding: bindings::CameraBinding,
    player_texture_binding: bindings::TextureBinding,
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

        let camera_binder = CameraBinder::new(&device);
        let texture_binder = TextureBinder::new(&device);

        let quad_pipeline =
            QuadPipeline::new(&device, config.format, &camera_binder, &texture_binder);

        let player_vertices = BackedBuffer::with_data(
            &device,
            vec![
                Vertex2d::new(glam::vec2(-5.0, -5.0), glam::vec2(0.0, 0.0)),
                Vertex2d::new(glam::vec2(5.0, -5.0), glam::vec2(1.0, 0.0)),
                Vertex2d::new(glam::vec2(5.0, 5.0), glam::vec2(1.0, 1.0)),
                Vertex2d::new(glam::vec2(-5.0, 5.0), glam::vec2(0.0, 1.0)),
            ],
            wgpu::BufferUsages::VERTEX,
        );
        let player_indices =
            BackedBuffer::with_data(&device, vec![0, 1, 2, 0, 2, 3], wgpu::BufferUsages::INDEX);
        let player_instances = BackedBuffer::with_capacity(&device, 8, wgpu::BufferUsages::VERTEX);

        let camera_buffer = BackedBuffer::with_data(
            &device,
            vec![CameraData::IDENTITY],
            wgpu::BufferUsages::UNIFORM,
        );
        let camera_binding = camera_binder.bind(&device, &camera_buffer);

        let player_texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::wgt::TextureDataOrder::MipMajor,
            &[255, 255, 255, 255],
        );
        let player_texture_view = player_texture.create_view(&Default::default());
        let default_sampler = device.create_sampler(&Default::default());
        let player_texture_binding =
            texture_binder.bind(&device, &player_texture_view, &default_sampler);

        Ok(Self {
            device,
            queue,
            window,
            surface,
            config,
            is_surface_configured: cfg!(not(target_arch = "wasm32")),
            quad_pipeline,
            player_vertices,
            player_indices,
            player_instances,
            camera_buffer,
            camera_binding,
            player_texture_binding,
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
                    return true;
                }
                e => {
                    log::error!("{e}");
                    return false;
                }
            },
        };

        let view = frame.texture.create_view(&Default::default());

        {
            self.player_instances.clear();
            let mut instances_batch = self.player_instances.batch(&self.device, &self.queue);
            for (i, player) in game.players().iter().enumerate() {
                instances_batch.push(InstanceColor2d::new(
                    player.position,
                    PLAYER_COLORS[i % PLAYER_COLORS.len()],
                ));
            }
        }

        {
            self.camera_buffer
                .update(&self.queue, |data| data[0].update(game.active_camera()));
        }

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.quad_pipeline.draw(
                &mut pass,
                &self.camera_binding,
                &self.player_texture_binding,
                &self.player_vertices,
                &self.player_indices,
                &self.player_instances,
            );
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

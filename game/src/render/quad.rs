use wgpu::BlendState;

use crate::render::{
    bindings::{CameraBinder, CameraBinding, TextureBinder, TextureBinding},
    buffer::BackedBuffer,
    vertex::{InstanceColor2d, Vertex2d},
};

pub struct QuadPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl QuadPipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        camera_binder: &CameraBinder,
        texture_binder: &TextureBinder,
    ) -> Self {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[camera_binder.layout(), texture_binder.layout()],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("quad.wgsl"));
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("QuadPipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex2d::VERTEX_LAYOUT, InstanceColor2d::VERTEX_LAYOUT],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });
        Self { pipeline }
    }

    pub fn draw<'a, 'b: 'a>(
        &'a self,
        pass: &'a mut wgpu::RenderPass<'b>,
        camera: &'a CameraBinding,
        texture: &'a TextureBinding,
        vertices: &'a BackedBuffer<Vertex2d>,
        indices: &'a BackedBuffer<u32>,
        instances: &'a BackedBuffer<InstanceColor2d>,
    ) {
        if instances.len() == 0 {
            return;
        }

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera.bind_group(), &[]);
        pass.set_bind_group(1, texture.bind_group(), &[]);
        pass.set_index_buffer(indices.slice(), wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(0, vertices.slice());
        pass.set_vertex_buffer(1, instances.slice());
        pass.draw_indexed(0..indices.len(), 0, 0..instances.len());
    }
}

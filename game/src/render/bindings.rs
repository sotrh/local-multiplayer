use crate::render::{buffer::BackedBuffer, uniform::CameraData};

pub struct CameraBinder {
    layout: wgpu::BindGroupLayout,
}

impl CameraBinder {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("CameraBinder"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        Self { layout }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        camera: &BackedBuffer<CameraData>,
    ) -> CameraBinding {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("CameraBinding"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera.buffer().as_entire_binding(),
                },
            ],
        });
        CameraBinding { bind_group }
    }
}

pub struct CameraBinding {
    bind_group: wgpu::BindGroup,
}

impl CameraBinding {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct TextureBinder {
    layout: wgpu::BindGroupLayout,
}

impl TextureBinder {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TextureBinder"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        Self { layout }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        texture: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> TextureBinding {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("TextureBinding"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        TextureBinding { bind_group }
    }
}

pub struct TextureBinding {
    bind_group: wgpu::BindGroup,
}

impl TextureBinding {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

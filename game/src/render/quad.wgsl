// use bindings::TextureBinder;

struct Vertex2d {
    @location(0)
    position: vec2<f32>,
    @location(1)
    uv: vec2<f32>,
}

struct InstanceColor2d {
    @location(2)
    position: vec2<f32>,
    @location(3)
    color: vec4<f32>,
}

struct VsOut {
    @builtin(position)
    frag_position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
    @location(1)
    color: vec4<f32>,
}

struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var tex: texture_2d<f32>;

@group(1)
@binding(1)
var samp: sampler;

@vertex
fn vs_main(vertex: Vertex2d, instance: InstanceColor2d) -> VsOut {
    return VsOut(
        camera.view_proj * vec4(vertex.position + instance.position, 0.0, 1.0),
        vertex.uv,
        instance.color,
    );
}

@fragment
fn fs_main(vs: VsOut) -> @location(0) vec4<f32> {
    return vs.color * textureSample(tex, samp, vs.uv);
}
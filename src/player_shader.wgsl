struct InstanceInput {
    @location(2) position: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    //顶点位置
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

//标记为vertex shader
@vertex
fn vs_main(
    model: VertexInput,
    instacne: InstanceInput,
) -> VertexOutput {
    //var：可变   需要声明类型
    //let：不可变 可以推断类型
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    let world_position = model.position + instacne.position;
    out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
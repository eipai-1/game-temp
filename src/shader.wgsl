struct InstanceInput {
    @location(5) position: vec3<f32>,
    @location(6) block_type: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    //顶点位置
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) layer: u32,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct MaterialUniform {
    index: vec4<u32>,
}

//标记为vertex shader
@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    //var：可变   需要声明类型
    //let：不可变 可以推断类型
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    let world_position = model.position + instance.position;
    if instance.block_type == 0 {
        out.clip_position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return out;
    } else {
        out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    }

    let face_index = vertex_index / 4u;
    let material_index = instance.block_type * 6u + face_index;
    out.layer = block_materials[material_index].index[0];
    
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d_array<f32>;

@group(1) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var<uniform> block_materials: array<MaterialUniform, 30>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords, in.layer);
}
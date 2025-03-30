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

struct ScreenSize {
    size: vec2<f32>
}

@group(0) @binding(0)
var<uniform> screen_size: ScreenSize;

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
    let scale = 0.59;

    let y_reverted_position = vec3<f32>(
        model.position.x,
        -model.position.y,
        model.position.z,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    let face_index = vertex_index / 4u;

    if instance.block_type == 0 || (face_index != 2u && face_index != 1u && face_index != 5u) {
        out.clip_position = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return out;
    }

    // 第一步：定义旋转角度（可调整）
    let x_rotation_angle = 35.264; // 度数，arctan(1/√2)，标准等轴视角
    let y_rotation_angle = 45.0;   // 度数

    // 第二步：将角度转换为弧度
    let x_angle_rad = x_rotation_angle * 3.14159 / 180.0;
    let y_angle_rad = y_rotation_angle * 3.14159 / 180.0;

    // 第三步：构建绕X轴的旋转矩阵
    let rx_matrix = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, cos(x_angle_rad), sin(x_angle_rad), 0.0),
        vec4<f32>(0.0, -sin(x_angle_rad), cos(x_angle_rad), 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );

    // 第四步：构建绕Y轴的旋转矩阵
    let ry_matrix = mat4x4<f32>(
        vec4<f32>(cos(y_angle_rad), 0.0, -sin(y_angle_rad), 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(sin(y_angle_rad), 0.0, cos(y_angle_rad), 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );

   let scale_matrix = mat4x4<f32>(
        vec4<f32>(scale, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, scale, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );

    let transform_matrix = rx_matrix * ry_matrix * scale_matrix;

    let transformed_position = transform_matrix * vec4<f32>(y_reverted_position, 1.0);

    out.clip_position = vec4<f32>(
        (transformed_position.x + instance.position.x) / screen_size.size.x * 2.0 - 1.0,
        1.0 - (transformed_position.y + instance.position.y) / screen_size.size.y * 2.0,
        0.0,
        1.0,
    );

    let material_index = instance.block_type * 6u + face_index;
    out.layer = block_materials[material_index].index[0];
    
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d_array<f32>;

@group(1) @binding(1)
var s_diffuse: sampler;

// 为了符合步长要求 使用结构体
@group(2) @binding(0)
var<uniform> block_materials: array<MaterialUniform, 30>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords, in.layer);
}
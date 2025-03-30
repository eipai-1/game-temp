struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct InstanceInput {
    @location(5) position: vec2<f32>,
}

struct ScreenSize {
    size: vec2<f32>
}

@group(0) @binding(0)
var<uniform> screen_size: ScreenSize;

@vertex
fn vs_main(ver_input: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(
        (ver_input.position.x + instance.position.x) / screen_size.size.x * 2.0 - 1.0,
        1.0 - (ver_input.position.y + instance.position.y) / screen_size.size.y * 2.0,
        0.0,
        1.0,
    );
    out.color = ver_input.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
struct VerInput {
	@location(0) position: vec3<f32>,
}

struct VerOutput {
	@builtin(position) clip_position: vec4<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct WireframeUniform {
	position: vec3<f32>,
	_padding: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> wf_uniform: WireframeUniform;

@vertex
fn vs_main(model: VerInput) -> VerOutput{
	var out: VerOutput;
	let world_position = model.position + wf_uniform.position;
	out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);

	return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32>{
	return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
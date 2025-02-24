struct VerInput {
	@location(0) position: vec3<f32>,
}

struct VerOutput {
	@builtin(position) clip_position: vec4<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(model: VerInput) -> VerOutput{
	var out: VerOutput;
	out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);

	return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32>{
	return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
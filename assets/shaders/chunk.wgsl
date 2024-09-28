// this is a modified combination of a bunch of bevy examples:
// https://github.com/bevyengine/bevy/blob/latest/assets/shaders/custom_vertex_attribute.wgsl
// https://github.com/bevyengine/bevy/blob/latest/assets/shaders/array_texture.wgsl
// https://github.com/bevyengine/bevy/blob/741803d8c98c627a1039815931b27aef147248f9/assets/shaders/extended_material.wgsl

#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_core_pipeline::tonemapping::tone_mapping
#import bevy_pbr::{
	pbr_types::PbrInput,
	pbr_fragment::pbr_input_from_standard_material,
}
#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
	prepass_io::{VertexOutput, FragmentOutput},
	pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::forward_io::{VertexOutput, FragmentOutput}
#endif

@group(2) @binding(100) var my_array_texture: texture_2d_array<f32>;
@group(2) @binding(101) var my_array_texture_sampler: sampler;

struct Vertex {
	@builtin(instance_index) instance_index: u32,
	@location(0) position: vec3<f32>,
	// @location(1) normal: vec3<f32>,
	@location(2) uv: vec2<f32>,
	@location(7) block_id: u32,
};

struct CustomVertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) position: vec4<f32>,
	@location(1) normal: vec3<f32>, // required, for some reason
	@location(2) uv: vec2<f32>,
	@location(6) idk: u32,
	@location(7) block_id: u32,
};


@vertex
fn vertex(vertex: Vertex) -> CustomVertexOutput {
	var out: CustomVertexOutput;
	out.clip_position = mesh_position_local_to_clip(
		get_world_from_local(vertex.instance_index),
		vec4<f32>(vertex.position, 1.0),
	);
	out.position = out.clip_position;
	out.normal = vec3<f32>(1.0, 0.0, 0.0); // shader requires normal, but ignores it
	out.uv = vertex.uv;
	out.idk = 10u;
	out.block_id = vertex.block_id;
	return out;
}

@fragment
fn fragment(
	@builtin(front_facing) is_front: bool,
	@location(7) block_id: u32,
	in: VertexOutput,
) -> FragmentOutput {
	// generate a PbrInput struct from the StandardMaterial bindings
	var pbr_input = pbr_input_from_standard_material(in, is_front);
	// get color from array texture
	pbr_input.material.base_color = textureSample(my_array_texture, my_array_texture_sampler, in.uv, i32(block_id));

#ifdef PREPASS_PIPELINE
	// in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
	let out = deferred_output(in, pbr_input);
#else
	var out: FragmentOutput;
	out.color = pbr_input.material.base_color;
#endif

	return out;
}

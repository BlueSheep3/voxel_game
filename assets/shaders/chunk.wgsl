// this is a modified combination of a bunch of bevy examples:
// https://github.com/bevyengine/bevy/blob/latest/assets/shaders/custom_vertex_attribute.wgsl
// https://github.com/bevyengine/bevy/blob/latest/assets/shaders/array_texture.wgsl
// https://github.com/bevyengine/bevy/blob/741803d8c98c627a1039815931b27aef147248f9/assets/shaders/extended_material.wgsl

#import bevy_pbr::mesh_functions::get_world_from_local
#import bevy_pbr::mesh_functions::mesh_position_local_to_clip
#import bevy_core_pipeline::tonemapping::tone_mapping
#import bevy_pbr::pbr_types::PbrInput
#import bevy_pbr::pbr_fragment::pbr_input_from_standard_material
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::forward_io::FragmentOutput

@group(2) @binding(100) var block_textures: texture_2d_array<f32>;
@group(2) @binding(101) var block_texture_sampler: sampler;
@group(2) @binding(102) var face_texture_indices: texture_1d<u32>;

struct Vertex {
	@builtin(instance_index) instance_index: u32,
	@location(0) position: vec3<f32>,
	// @location(1) normal: vec3<f32>,
	@location(2) uv: vec2<f32>,
	@location(7) start_texture_index: u32,
	@location(8) rect_width: u32,
};

struct CustomVertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) position: vec4<f32>,
	@location(1) normal: vec3<f32>, // required, for some reason
	@location(2) uv: vec2<f32>,
	@location(6) idk: u32, // required, for some reason
	@location(7) start_texture_index: u32,
	@location(8) rect_width: u32,
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
	out.idk = 10u; // required, for some reason
	out.start_texture_index = vertex.start_texture_index;
	out.rect_width = vertex.rect_width;
	return out;
}

@fragment
fn fragment(
	@builtin(front_facing) is_front: bool,
	@location(7) start_texture_index: u32,
	@location(8) rect_width: u32,
	in: VertexOutput,
) -> FragmentOutput {
	// generate a PbrInput struct from the StandardMaterial bindings
	var pbr_input = pbr_input_from_standard_material(in, is_front);

	// get the index of this blocks texture
	let uv_offset = u32(in.uv.x) + u32(in.uv.y) * rect_width;
	let face_index = start_texture_index + uv_offset;
	let block_index = textureLoad(face_texture_indices, face_index, 0).r;

	// sample the actual texture this face is going to have
	pbr_input.material.base_color = textureSample(
		block_textures, block_texture_sampler, in.uv, u32(block_index)
	);

	var out: FragmentOutput;
	out.color = pbr_input.material.base_color;

	return out;
}

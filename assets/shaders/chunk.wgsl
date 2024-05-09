// this is a modified combination of:
// https://github.com/bevyengine/bevy/blob/latest/assets/shaders/array_texture.wgsl
// with some vertex shader things from:
// https://github.com/bevyengine/bevy/blob/latest/assets/shaders/custom_vertex_attribute.wgsl

#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}
#import bevy_pbr::{
	forward_io::VertexOutput,
	mesh_view_bindings::view,
	pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
	pbr_functions as fns,
}
#import bevy_core_pipeline::tonemapping::tone_mapping

@group(2) @binding(0) var my_array_texture: texture_2d_array<f32>;
@group(2) @binding(1) var my_array_texture_sampler: sampler;

struct Vertex {
	@builtin(instance_index) instance_index: u32,
	@location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) uv: vec2<f32>,
	@location(7) block_id: u32,
};

struct CustomVertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) position: vec4<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) uv: vec2<f32>,
	@location(6) idk: u32,
	@location(7) block_id: u32,
};


@vertex
fn vertex(vertex: Vertex) -> CustomVertexOutput {
	var out: CustomVertexOutput;
	out.clip_position = mesh_position_local_to_clip(
		get_model_matrix(vertex.instance_index),
		vec4<f32>(vertex.position, 1.0),
	);
	out.position = out.clip_position;
	out.normal = vertex.normal;
	out.uv = vertex.uv;
	out.idk = u32(10);
	out.block_id = vertex.block_id;
	return out;
}

@fragment
fn fragment(
	@builtin(front_facing) is_front: bool,
	@location(7) block_id: u32,
	mesh: VertexOutput,
) -> @location(0) vec4<f32> {
	return textureSample(my_array_texture, my_array_texture_sampler, mesh.uv, i32(block_id));

	// Prepare a 'processed' StandardMaterial by sampling all textures to resolve
	// the material members
	// var pbr_input: PbrInput = pbr_input_new();

	// pbr_input.material.base_color = textureSample(my_array_texture, my_array_texture_sampler, mesh.uv, layer);
// #ifdef VERTEX_COLORS
// 	pbr_input.material.base_color = pbr_input.material.base_color * mesh.color;
// #endif

// 	let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

// 	pbr_input.frag_coord = mesh.position;
// 	pbr_input.world_position = mesh.world_position;
// 	pbr_input.world_normal = fns::prepare_world_normal(
// 		mesh.world_normal,
// 		double_sided,
// 		is_front,
// 	);

// 	pbr_input.is_orthographic = view.projection[3].w == 1.0;

// 	pbr_input.N = fns::apply_normal_mapping(
// 		pbr_input.material.flags,
// 		mesh.world_normal,
// 		double_sided,
// 		is_front,
// #ifdef VERTEX_TANGENTS
// #ifdef STANDARD_MATERIAL_NORMAL_MAP
// 		mesh.world_tangent,
// #endif
// #endif
// 		mesh.uv,
// 		view.mip_bias,
// 	);
// 	pbr_input.V = fns::calculate_view(mesh.world_position, pbr_input.is_orthographic);

	// return tone_mapping(fns::apply_pbr_lighting(pbr_input), view.color_grading);
}

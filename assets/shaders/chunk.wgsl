// this shader is mostly copied from:
// https://playspacefarer.com/voxel-array-textures/

// this is a modified combination of
// https://github.com/bevyengine/bevy/blob/latest/assets/shaders/custom_vertex_attribute.wgsl
// and
// https://github.com/bevyengine/bevy/blob/latest/assets/shaders/array_texture.wgsl

// #import bevy_pbr::mesh_view_bindings
// #import bevy_pbr::pbr_types
// #import bevy_pbr::mesh_bindings

// #import bevy_pbr::utils
// #import bevy_pbr::clustered_forward
// #import bevy_pbr::lighting
// #import bevy_pbr::shadows
// #import bevy_pbr::pbr_functions

// @group(1) @binding(0)
// var chunk_texture: texture_2d_array<f32>;

// @group(1) @binding(1)
// var chunk_sampler: sampler;

// @group(1) @binding(2)
// var pbr_texture: texture_2d_array<f32>;

// @group(1) @binding(3)
// var pbr_sampler: sampler;

// // NOTE: Bindings must come before functions that use them!
// #import bevy_pbr::mesh_functions

// struct Vertex {
// 	@location(0) position: vec3<f32>,
// 	@location(1) normal: vec3<f32>,
// #ifdef VERTEX_UVS
// 	@location(2) uv: vec2<f32>,
// #endif
// #ifdef VERTEX_TANGENTS
// 	@location(3) tangent: vec4<f32>,
// #endif
// #ifdef VERTEX_COLORS
// 	@location(4) color: vec4<f32>,
// #endif
// #ifdef SKINNED
// 	@location(5) joint_indices: vec4<u32>,
// 	@location(6) joint_weights: vec4<f32>,
// #endif
// 	@location(7) base_indice: u32,
// 	// @location(8) overlay_indice: u32,
// };

// #import bevy_pbr::mesh_vertex_output

// struct VertexOutput {
// 	@builtin(position) clip_position: vec4<f32>,
	
// 	@location(5) base_indice: u32,
// 	// @location(6) overlay_indice: u32,
// };

// @vertex
// fn vertex(vertex: Vertex) -> VertexOutput {
// 	var out: VertexOutput;
// #ifdef SKINNED
// 	var model = skin_model(vertex.joint_indices, vertex.joint_weights);
// 	out.world_normal = skin_normals(model, vertex.normal);
// #else
// 	var model = mesh.model;
// 	out.world_normal = mesh_normal_local_to_world(vertex.normal);
// #endif
// 	out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
// #ifdef VERTEX_UVS
// 	out.uv = vertex.uv;
// #endif
// #ifdef VERTEX_TANGENTS
// 	out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
// #endif
// #ifdef VERTEX_COLORS
// 	out.color = vertex.color;
// #endif
// 	out.clip_position = mesh_position_world_to_clip(out.world_position);

// 	out.base_indice = vertex.base_indice;
// 	// out.overlay_indice = vertex.overlay_indice;

// 	return out;
// }

// #import bevy_pbr::mesh_vertex_output

// struct FragmentInput {
// 	@builtin(front_facing) is_front: bool,
// 	@builtin(position) frag_coord: vec4<f32>,
	
// 	@location(5) base_indice: u32,
// 	// @location(6) overlay_indice: u32,
// };

// @fragment
// fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
// 	var pbr_input: PbrInput = pbr_input_new();

// 	// base color with overlay

// 	let b = textureSample(chunk_texture, chunk_sampler, in.uv, i32(in.base_indice));
// 	// let o = textureSample(chunk_texture, chunk_sampler, in.uv, i32(in.overlay_indice));

// 	let bg_r = b.r * b.a;
// 	let bg_g = b.g * b.a;
// 	let bg_b = b.b * b.a;

// 	// let fg_r = o.r * o.a;
// 	// let fg_g = o.g * o.a;
// 	// let fg_b = o.b * o.a;

// 	let color_r = fg_r + bg_r; // * (1.0 - o.a);
// 	let color_g = fg_g + bg_g; // * (1.0 - o.a);
// 	let color_b = fg_b + bg_b; // * (1.0 - o.a);

// 	let color = vec4<f32>(color_r, color_g, color_b, 1.0);

// 	pbr_input.material.base_color = color;

// 	// pbr values with overlay

// 	let pbr = textureSample(pbr_texture, pbr_sampler, in.uv, i32(in.base_indice));
// 	// let pbr_o = textureSample(pbr_texture, pbr_sampler, in.uv, i32(in.overlay_indice));

// 	let pbr_bg_r = pbr.r * b.a;
// 	let pbr_bg_g = pbr.g * b.a;
// 	let pbr_bg_b = pbr.b * b.a;

// 	// let pbr_fg_r = pbr_o.r * o.a;
// 	// let pbr_fg_g = pbr_o.g * o.a;
// 	// let pbr_fg_b = pbr_o.b * o.a;

// 	let pbr_r = pbr_fg_r + pbr_bg_r; // * (1.0 - o.a);
// 	let pbr_g = pbr_fg_g + pbr_bg_g; // * (1.0 - o.a);
// 	let pbr_b = pbr_fg_b + pbr_bg_b; // * (1.0 - o.a);

// 	pbr_input.material.perceptual_roughness = pbr_r;
// 	pbr_input.material.metallic = pbr_g;
// 	pbr_input.material.reflectance = pbr_b;

// #ifdef VERTEX_COLORS
// 	pbr_input.material.base_color = pbr_input.material.base_color * in.color;
// #endif

// 	pbr_input.frag_coord = in.frag_coord;
// 	pbr_input.world_position = in.world_position;
// 	pbr_input.world_normal = prepare_world_normal(
// 		in.world_normal,
// 		false,
// 		in.is_front,
// 	);

// 	pbr_input.is_orthographic = view.projection[3].w == 1.0;

// 	pbr_input.N = apply_normal_mapping(
// 		pbr_input.material.flags,
// 		pbr_input.world_normal,
// #ifdef VERTEX_TANGENTS
// #ifdef STANDARDMATERIAL_NORMAL_MAP
// 		in.world_tangent,
// #endif
// #endif
// #ifdef VERTEX_UVS
// 		in.uv,
// #endif
// 	);
// 	pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

// 	return pbr(pbr_input);
// }





// #import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}
// #import bevy_pbr::{
// 	mesh_view_bindings::view,
// 	pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
// 	pbr_functions as fns,
// }
// #import bevy_core_pipeline::tonemapping::tone_mapping

// @group(2) @binding(0) var my_array_texture: texture_2d_array<f32>;
// @group(2) @binding(1) var my_array_texture_sampler: sampler;

// // struct ChunkMaterial {
// // 	color: vec4<f32>,
// // };
// // @group(2) @binding(0) var<uniform> material: ChunkMaterial;

// struct Vertex {
// 	@builtin(instance_index) instance_index: u32,
// 	@location(0) position: vec3<f32>,
// 	@location(1) normal: vec4<f32>,
// 	@location(2) uv: vec2<f32>,
// 	@location(7) block_id: u32,
// };

// struct VertexOutput {
// 	@builtin(position) clip_position: vec4<f32>,
// 	@location(0) position: vec3<f32>,
// 	@location(1) normal: vec4<f32>,
// 	@location(2) uv: vec2<f32>,
// 	@location(7) block_id: vec4<f32>,
// };

// @vertex
// fn vertex(vertex: Vertex) -> VertexOutput {
// 	var out: VertexOutput;
// 	out.clip_position = mesh_position_local_to_clip(
// 		get_model_matrix(vertex.instance_index),
// 		vec4<f32>(vertex.position, 1.0),
// 	);
// 	out.position = vertex.position;
// 	out.normal = vertex.normal;
// 	out.uv = vertex.uv;
// 	out.block_id = vertex.block_id;
// 	return out;
// }

// struct FragmentInput {
// 	@builtin(front_facing) is_front: bool,
// 	mesh: VertexOutput,
// };

// @fragment
// fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
// 	let layer = input.mesh.block_id;

// 	// Prepare a 'processed' StandardMaterial by sampling all textures to resolve
// 	// the material members
// 	var pbr_input: PbrInput = pbr_input_new();

// 	pbr_input.material.base_color = textureSample(my_array_texture, my_array_texture_sampler, input.mesh.uv, layer);
// #ifdef VERTEX_COLORS
// 	pbr_input.material.base_color = pbr_input.material.base_color * input.mesh.color;
// #endif

// 	let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

// 	pbr_input.frag_coord = input.mesh.position;
// 	pbr_input.world_position = input.mesh.world_position;
// 	pbr_input.world_normal = fns::prepare_world_normal(
// 		input.mesh.world_normal,
// 		double_sided,
// 		input.is_front,
// 	);

// 	pbr_input.is_orthographic = view.projection[3].w == 1.0;

// 	pbr_input.N = fns::apply_normal_mapping(
// 		pbr_input.material.flags,
// 		input.mesh.world_normal,
// 		double_sided,
// 		input.is_front,
// #ifdef VERTEX_TANGENTS
// #ifdef STANDARD_MATERIAL_NORMAL_MAP
// 		input.mesh.world_tangent,
// #endif
// #endif
// 		input.mesh.uv,
// 		view.mip_bias,
// 	);
// 	pbr_input.V = fns::calculate_view(input.mesh.world_position, pbr_input.is_orthographic);

// 	return tone_mapping(fns::apply_pbr_lighting(pbr_input), view.color_grading);
// }
















#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
    pbr_functions as fns,
}
#import bevy_core_pipeline::tonemapping::tone_mapping

@group(2) @binding(0) var my_array_texture: texture_2d_array<f32>;
@group(2) @binding(1) var my_array_texture_sampler: sampler;

@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let layer = i32(mesh.world_position.x) & 0x3;

    // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
    // the material members
    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.material.base_color = textureSample(my_array_texture, my_array_texture_sampler, mesh.uv, layer);
#ifdef VERTEX_COLORS
    pbr_input.material.base_color = pbr_input.material.base_color * mesh.color;
#endif

    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    pbr_input.frag_coord = mesh.position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.world_normal = fns::prepare_world_normal(
        mesh.world_normal,
        double_sided,
        is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = fns::apply_normal_mapping(
        pbr_input.material.flags,
        mesh.world_normal,
        double_sided,
        is_front,
#ifdef VERTEX_TANGENTS
#ifdef STANDARD_MATERIAL_NORMAL_MAP
        mesh.world_tangent,
#endif
#endif
        mesh.uv,
        view.mip_bias,
    );
    pbr_input.V = fns::calculate_view(mesh.world_position, pbr_input.is_orthographic);

    return tone_mapping(fns::apply_pbr_lighting(pbr_input), view.color_grading);
}

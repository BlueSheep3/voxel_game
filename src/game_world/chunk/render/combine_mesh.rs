use bevy::{
	// math::Vec4Swizzles,
	prelude::{/* Mat3, */ Mesh, Transform, Vec3},
	render::{
		mesh::{Indices, VertexAttributeValues},
		render_asset::RenderAssetUsages,
		render_resource::PrimitiveTopology,
	},
};

use crate::block_model::ATTRIBUTE_BASE_VOXEL_INDICES;

// PERF combine meshes immedeatly instead of first collecting
// them into a vec to only allocate each mesh once

// this functions entire code is copied from:
// https://gist.github.com/DGriffin91/e63e5f7a90b633250c2cf4bf8fd61ef8
// and then modified to only include necessary things
pub fn combine_meshes(meshes: &[Mesh]) -> Mesh {
	let mut mesh = Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::default(),
	);

	let mut positions: Vec<[f32; 3]> = Vec::new();
	// let mut normals: Vec<[f32; 3]> = Vec::new();
	let mut uvs: Vec<[f32; 2]> = Vec::new();
	let mut voxel_indices: Vec<u32> = Vec::new();
	let mut indices: Vec<u32> = Vec::new();

	let mut indices_offset = 0;

	for mesh in meshes {
		let Indices::U32(mesh_indices) = &mesh.indices().unwrap() else {
			continue;
		};

		let mat = Transform::IDENTITY.compute_matrix();

		let positions_len;

		if let Some(VertexAttributeValues::Float32x3(vert_positions)) =
			&mesh.attribute(Mesh::ATTRIBUTE_POSITION)
		{
			positions_len = vert_positions.len();
			for p in vert_positions {
				positions.push(mat.transform_point3(Vec3::from(*p)).into());
			}
		} else {
			panic!("no positions")
		}

		if let Some(VertexAttributeValues::Float32x2(vert_uv)) =
			&mesh.attribute(Mesh::ATTRIBUTE_UV_0)
		{
			for uv in vert_uv {
				uvs.push(*uv);
			}
		} else {
			panic!("no uvs")
		}

		// Comment below taken from mesh_normal_local_to_world() in mesh_functions.wgsl regarding
		// transform normals from local to world coordinates:

		// NOTE: The mikktspace method of normal mapping requires that the world normal is
		// re-normalized in the vertex shader to match the way mikktspace bakes vertex tangents
		// and normal maps so that the exact inverse process is applied when shading. Blender, Unity,
		// Unreal Engine, Godot, and more all use the mikktspace method. Do not change this code
		// unless you really know what you are doing.
		// http://www.mikktspace.com/

		// let inverse_transpose_model = mat.inverse().transpose();
		// let inverse_transpose_model = Mat3 {
		// 	x_axis: inverse_transpose_model.x_axis.xyz(),
		// 	y_axis: inverse_transpose_model.y_axis.xyz(),
		// 	z_axis: inverse_transpose_model.z_axis.xyz(),
		// };

		// if let Some(VertexAttributeValues::Float32x3(vert_normals)) =
		// 	&mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
		// {
		// 	for n in vert_normals {
		// 		normals.push(
		// 			inverse_transpose_model
		// 				.mul_vec3(Vec3::from(*n))
		// 				.normalize_or_zero()
		// 				.into(),
		// 		);
		// 	}
		// } else {
		// 	panic!("no normals")
		// }

		if let Some(VertexAttributeValues::Uint32(vis)) =
			&mesh.attribute(ATTRIBUTE_BASE_VOXEL_INDICES)
		{
			for vi in vis {
				voxel_indices.push(*vi);
			}
		} else {
			panic!("no voxel indices")
		}

		for i in mesh_indices {
			indices.push(*i + indices_offset);
		}
		indices_offset += positions_len as u32;
	}

	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
	// mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
	mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
	mesh.insert_attribute(ATTRIBUTE_BASE_VOXEL_INDICES, voxel_indices);
	mesh.insert_indices(Indices::U32(indices));

	mesh
}

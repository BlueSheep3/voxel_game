use super::combine_mesh::combine_meshes;
use crate::{
	block::BlockId,
	block_model::{BlockModel, BlockModelCuboid},
	face::{Face, FaceMap, FacesMask},
	game_world::chunk::{Chunk, CHUNK_LENGTH},
	pos::UVec3Utils,
};
use bevy::{
	prelude::*,
	render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use std::collections::HashMap;

struct BlockMeshInfo {
	/// the shape of the cube mesh
	cuboid: BlockModelCuboid<Rect>,
	/// which faces of the cubes should not be rendered to improve preformance
	culled: FacesMask,
	/// how much this block is offset from `(0,0,0)` in this chunk
	pos: UVec3,
}

pub fn create_chunk_mesh(
	chunk: &Chunk,
	neighbour_chunks: &FaceMap<Chunk>,
	block_models: &HashMap<BlockId, BlockModel<Rect>>,
) -> Mesh {
	let meshes = chunk
		.blocks
		.iter_xyz()
		.flat_map(|(pos, block)| {
			let block_model = block_models.get(&block.id).unwrap_or_else(|| {
				panic!("tried to get the model of block with id {:?}", block.id)
			});
			block_model
				.cuboids
				.iter()
				.map(move |cuboid| (pos, cuboid, block_model.should_cull))
		})
		.flat_map(|(pos, cuboid, should_cull)| {
			let culled = if should_cull {
				get_culled_faces_at(chunk, neighbour_chunks, pos, block_models)
			} else {
				// lazy approach of not culling anything if it's not a full block
				// TODO cull those faces that are still covered up
				FacesMask::none()
			};

			// dont need to create a mesh if everything is culled away
			if culled.is_all() {
				return None;
			}

			let info = BlockMeshInfo {
				cuboid: cuboid.clone(),
				culled,
				pos,
			};
			let mesh = create_cube_mesh(info);
			Some(mesh)
		})
		.collect::<Vec<_>>();

	combine_meshes(&meshes)
}

fn get_culled_faces_at(
	chunk: &Chunk,
	neighbour_chunks: &FaceMap<Chunk>,
	pos: UVec3,
	block_models: &HashMap<BlockId, BlockModel<Rect>>,
) -> FacesMask {
	let mut culled = FacesMask::none();

	macro_rules! cull {
		($axis:ident, $offset:expr, $face:ident, [$border:expr, $other_border:expr]) => {
			if pos.$axis == $border as u32 {
				let chunk = neighbour_chunks.get(Face::$face);
				let mut adjacent_pos = pos;
				adjacent_pos.$axis = $other_border as u32;
				let model = block_models.get(&chunk.blocks[adjacent_pos].id).unwrap();
				if model.should_cull {
					culled.set(Face::$face);
				}
			} else {
				let pos = IVec3::try_from(pos).unwrap() + $offset;
				let pos: UVec3 = pos.try_into().unwrap();
				let model = block_models.get(&chunk.blocks[pos].id).unwrap();
				if model.should_cull {
					culled.set(Face::$face);
				}
			}
		};
	}

	cull!(x, IVec3::X, Right, [CHUNK_LENGTH - 1, 0]);
	cull!(x, -IVec3::X, Left, [0, CHUNK_LENGTH - 1]);
	cull!(y, IVec3::Y, Up, [CHUNK_LENGTH - 1, 0]);
	cull!(y, -IVec3::Y, Down, [0, CHUNK_LENGTH - 1]);
	cull!(z, IVec3::Z, Back, [CHUNK_LENGTH - 1, 0]);
	cull!(z, -IVec3::Z, Forward, [0, CHUNK_LENGTH - 1]);

	culled
}

/// creates the mesh for a cube with different face UVs and culling
fn create_cube_mesh(block_mesh_info: BlockMeshInfo) -> Mesh {
	let cuboid = block_mesh_info.cuboid;
	let culled = block_mesh_info.culled;
	let offset = block_mesh_info.pos;

	let mut cube_mesh = Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::default(),
	);

	let positions = get_cube_mesh_positions(cuboid.min, cuboid.max, offset, culled);
	cube_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

	let uvs = get_cube_mesh_uvs(&cuboid, culled);
	cube_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

	// normals are only required for lighting and this game uses a custom lighting engine
	// let normals = get_cube_mesh_normals();
	// cube_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

	let tris = get_cube_mesh_tris(culled);
	cube_mesh.insert_indices(Indices::U32(tris));

	cube_mesh
}

fn get_cube_mesh_positions(
	min: Vec3,
	max: Vec3,
	offset: UVec3,
	culled: FacesMask,
) -> Vec<[f32; 3]> {
	macro_rules! ignore_culled {
		($culled:ident; $(($face:ident, $([$x:tt, $y:tt, $z:tt]),*));* $(;)?) => {{
			let mut vec = Vec::new();
			$(
				if !$culled.contains(Face::$face) {
					vec.extend([$([
						num_to_main_max!($x, x),
						num_to_main_max!($y, y),
						num_to_main_max!($z, z),
					]),*]);
				}
			)*
			vec
		}};
	}

	macro_rules! num_to_main_max {
		(0, $axis:ident) => {
			min.$axis
		};
		(1, $axis:ident) => {
			max.$axis
		};
	}

	let mut positions = ignore_culled! {
		culled;
		(Right, [1, 1, 1], [1, 0, 1], [1, 0, 0], [1, 1, 0]);
		(Left, [0, 1, 0], [0, 0, 0], [0, 0, 1], [0, 1, 1]);
		(Up, [0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]);
		(Down, [0, 0, 1], [0, 0, 0], [1, 0, 0], [1, 0, 1]);
		(Back, [0, 1, 1], [0, 0, 1], [1, 0, 1], [1, 1, 1]);
		(Forward, [1, 1, 0], [1, 0, 0], [0, 0, 0], [0, 1, 0]);
	};

	let offset = offset.to_vec3();

	for [x, y, z] in &mut positions {
		*x += offset.x;
		*y += offset.y;
		*z += offset.z;
	}

	positions
}

fn get_cube_mesh_uvs(block_model: &BlockModelCuboid<Rect>, culled: FacesMask) -> Vec<[f32; 2]> {
	// Set-up UV coordinated to point to the upper (V < 0.5), "dirt+grass" part of the texture.
	// Take a look at the custom image (assets/textures/array_texture.png)
	// so the UV coords will make more sense
	// Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping

	let sides = block_model.sides;

	let mut uvs = Vec::new();

	fn extend_uvs(
		uvs: &mut Vec<[f32; 2]>,
		positions: &FaceMap<Rect>,
		face: Face,
		culled: FacesMask,
	) {
		if culled.contains(face) {
			return;
		}
		let Rect { min, max } = positions.get(face);
		let Vec2 { x: x0, y: y0 } = *min;
		let Vec2 { x: x1, y: y1 } = *max;
		uvs.extend([[x0, y0], [x0, y1], [x1, y1], [x1, y0]]);
	}

	extend_uvs(&mut uvs, &sides, Face::Right, culled);
	extend_uvs(&mut uvs, &sides, Face::Left, culled);
	extend_uvs(&mut uvs, &sides, Face::Up, culled);
	extend_uvs(&mut uvs, &sides, Face::Down, culled);
	extend_uvs(&mut uvs, &sides, Face::Back, culled);
	extend_uvs(&mut uvs, &sides, Face::Forward, culled);

	uvs
}

// not required for this game, because this game uses a custom lighting engine
#[allow(dead_code)]
fn get_cube_mesh_normals() -> Vec<[f32; 3]> {
	// For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
	// the surface.
	// Normals are required for correct lighting calculations.
	// Each array represents a normalized vector, which length should be equal to 1.0.

	// NOTE this is outdated and does not cull anything

	#[rustfmt::skip]
	let normals = vec![
		// Normals for the right side (towards +x)
		[1.0, 0.0, 0.0],
		[1.0, 0.0, 0.0],
		[1.0, 0.0, 0.0],
		[1.0, 0.0, 0.0],
		// Normals for the left side (towards -x)
		[-1.0, 0.0, 0.0],
		[-1.0, 0.0, 0.0],
		[-1.0, 0.0, 0.0],
		[-1.0, 0.0, 0.0],
		// Normals for the up side (towards +y)
		[0.0, 1.0, 0.0],
		[0.0, 1.0, 0.0],
		[0.0, 1.0, 0.0],
		[0.0, 1.0, 0.0],
		// Normals for the down side (towards -y)
		[0.0, -1.0, 0.0],
		[0.0, -1.0, 0.0],
		[0.0, -1.0, 0.0],
		[0.0, -1.0, 0.0],
		// Normals for the back side (towards +z)
		[0.0, 0.0, 1.0],
		[0.0, 0.0, 1.0],
		[0.0, 0.0, 1.0],
		[0.0, 0.0, 1.0],
		// Normals for the forward side (towards -z)
		[0.0, 0.0, -1.0],
		[0.0, 0.0, -1.0],
		[0.0, 0.0, -1.0],
		[0.0, 0.0, -1.0],
	];

	normals
}

fn get_cube_mesh_tris(culled: FacesMask) -> Vec<u32> {
	// Create the triangles out of the 24 vertices we created.
	// To construct a square, we need 2 triangles, therefore 12 triangles in total.
	// To construct a triangle, we need the indices of its 3 defined vertices, adding them one
	// by one, in a counter-clockwise order (relative to the position of the viewer, the order
	// should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
	// Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
	// further examples and the implementation of the built-in shapes.

	let mut vec = Vec::new();
	let mut i = 0;

	for face in Face::all() {
		if culled.contains(face) {
			continue;
		}
		vec.extend([i, i + 1, i + 3, i + 2, i + 3, i + 1]);
		i += 4;
	}

	vec
}

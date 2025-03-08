use super::combine_mesh::combine_meshes;
use crate::{
	axis::AxisMap,
	bench::BenchName,
	block::BlockId,
	block_model::{BlockFaceData, BlockModel, ATTRIBUTE_BASE_VOXEL_INDICES},
	face::{Face, FaceMap},
	game_world::chunk::{Chunk, CHUNK_LENGTH},
	pos::BlockInChunkPos,
};
use bevy::{
	prelude::*,
	render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use std::{collections::HashMap, time::Instant};

struct BlockFaceInfo {
	/// which way this face is facing
	face: Face,
	/// the shape and texture of this face
	data: BlockFaceData<usize>,
	/// how much this block is offset from `(0,0,0)` in this chunk
	pos: BlockInChunkPos,
}

pub fn create_chunk_mesh(
	chunk: Chunk,
	blocks_mask: Box<AxisMap<[[u64; CHUNK_LENGTH]; CHUNK_LENGTH]>>,
	block_models: HashMap<BlockId, BlockModel<usize>>,
) -> Mesh {
	// benchmarking of creating the chunk mesh
	let start_time = Instant::now();

	// currently this has to iterate over the masks twice per axis, since there are 2 faces.
	// im not sure if this has worse (maybe better?) performance than if
	// you iterated over every axis exactly once.
	let culled_blocks_mask = FaceMap::from_map(|face| {
		// `>> 1` because the negative chunk neighbour's block takes up 1 bit at the edge.
		// the cast to `u32` cuts of the positive chunk neighbour's bit.
		let culling = if face.is_pos() {
			|mask: u64| ((mask & !(mask >> 1)) >> 1) as u32
		} else {
			|mask: u64| ((mask & !(mask << 1)) >> 1) as u32
		};
		// TODO test wether the usage of `map` is bad for performance
		blocks_mask[face.axis()].map(|array| array.map(culling))
	});

	let mut all_faces = FaceMap::from_map(|_| Vec::new());
	for (face, array2d) in culled_blocks_mask.iter_face() {
		for (i, array) in array2d.iter().enumerate() {
			for (j, mask) in array.iter().enumerate() {
				let mut mask = *mask;
				let mut k = 0;
				while mask != 0 {
					let zeros = mask.trailing_zeros();
					mask = (mask >> zeros) & !1;
					k += zeros;

					// FIXME use the correct coordinates
					let pos = BlockInChunkPos {
						x: i as u8,
						y: j as u8,
						z: k as u8,
					};
					let block = chunk.blocks[pos];
					let block_model = block_models.get(&block.id).unwrap_or_else(|| {
						panic!("tried to get the model of block with id {:?}", block.id)
					});
					// mapping the block positions here might seem like unnecessary
					// duplication of this data, but most blocks only have a single face
					// per direction, meaning that most of the time this wont be duplicated.
					// the only way to not duplicate the `pos` would be to store the actual
					// Vec inside `all_faces`, which would introduce indirection and all
					// the extra data needed to store a Vec.
					all_faces[face].extend(block_model.faces[face].iter().map(|data| (data, pos)));
				}
			}
		}
	}

	let meshes = all_faces.iter_face().flat_map(|(face, datas)| {
		datas.iter().map({
			move |&(&data, pos)| {
				let info = BlockFaceInfo { face, data, pos };
				create_face_mesh(info)
			}
		})
	});

	let combined_meshes = combine_meshes(meshes);

	crate::bench::push_time(BenchName::CreateChunkMesh, start_time.elapsed());

	combined_meshes
}

fn create_face_mesh(info: BlockFaceInfo) -> Mesh {
	let BlockFaceInfo { face, data, pos } = info;

	let mut cube_mesh = Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::default(),
	);

	let positions = get_face_positions(face, data.min, data.max, pos);
	cube_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

	// let uvs = get_cube_mesh_uvs(&cuboid, culled);
	// let uvs = get_temp_const_uvs(culled);
	// in the future block models may define different uvs, this is temporary
	let Rect { min, max } = Rect::from_corners(Vec2::ZERO, Vec2::ONE);
	let Vec2 { x: x0, y: y0 } = min;
	let Vec2 { x: x1, y: y1 } = max;
	let uvs = vec![[x0, y0], [x0, y1], [x1, y1], [x1, y0]];
	cube_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

	// normals are only required for lighting and this game uses a custom lighting engine
	// let normals = get_face_mesh_normals();
	// cube_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

	// let voxel_indices = get_cube_mesh_voxel_indices(&cuboid, culled);
	// all 4 vertices must have the same voxel index, since they belong to the same face
	let voxel_indices = vec![data.side as u32; 4];
	cube_mesh.insert_attribute(ATTRIBUTE_BASE_VOXEL_INDICES, voxel_indices);

	// since we are constructing a single face, we already
	// know the exact indeces for the 2 triangles
	let tris = vec![0, 1, 3, 2, 3, 1];
	cube_mesh.insert_indices(Indices::U32(tris));

	cube_mesh
}

fn get_face_positions(face: Face, min: Vec3, max: Vec3, pos: BlockInChunkPos) -> Vec<[f32; 3]> {
	macro_rules! min_max {
		($([$x:tt, $y:tt, $z:tt]),* $(,)?) => {{
			vec![$([
				num_to_min_max!($x, x),
				num_to_min_max!($y, y),
				num_to_min_max!($z, z),
			]),*]
		}};
	}

	macro_rules! num_to_min_max {
		(0, $axis:ident) => {
			min.$axis
		};
		(1, $axis:ident) => {
			max.$axis
		};
	}

	let mut positions = match face {
		Face::Right => min_max!([1, 1, 1], [1, 0, 1], [1, 0, 0], [1, 1, 0]),
		Face::Left => min_max!([0, 1, 0], [0, 0, 0], [0, 0, 1], [0, 1, 1]),
		Face::Up => min_max!([0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]),
		Face::Down => min_max!([0, 0, 1], [0, 0, 0], [1, 0, 0], [1, 0, 1]),
		Face::Back => min_max!([0, 1, 1], [0, 0, 1], [1, 0, 1], [1, 1, 1]),
		Face::Forward => min_max!([1, 1, 0], [1, 0, 0], [0, 0, 0], [0, 1, 0]),
	};

	let offset = Vec3::from(pos);
	for [x, y, z] in &mut positions {
		*x += offset.x;
		*y += offset.y;
		*z += offset.z;
	}

	positions
}

use super::combine_mesh::combine_meshes;
use crate::{
	axis::{Axis, AxisMap},
	bench::BenchName,
	block::{
		prelude::{Air, BlockTrait, BlockWithoutData},
		Block, BlockId,
	},
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

pub type ChunkArray2D<T> = [[T; CHUNK_LENGTH]; CHUNK_LENGTH];

pub type ChunkPadding = FaceMap<Box<ChunkArray2D<Block>>>;

pub fn create_chunk_mesh(
	chunk: Chunk,
	chunk_padding: ChunkPadding,
	block_models: HashMap<BlockId, BlockModel<usize>>,
) -> Mesh {
	// benchmarking of creating the chunk mesh
	let start_time = Instant::now();

	let bitmask_time = Instant::now();
	let (blocks_mask, non_culled_mask) = get_blocks_bitmask(&chunk, &block_models, chunk_padding);
	crate::bench::push_time(BenchName::BitMask, bitmask_time.elapsed());

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

	// extract the actual BlockFaceData from these bitmasks and the chunk data
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

					let pos = bitmask_pos_to_world(face.axis(), i, j, k);
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
	// basically the same thing as the for loop above,
	// but adjusted for `non_culled_mask`
	for (x, array) in non_culled_mask.iter().enumerate() {
		for (y, mask) in array.iter().enumerate() {
			let mut mask = *mask;
			let mut z = 0;
			while mask != 0 {
				let zeros = mask.trailing_zeros();
				mask = (mask >> zeros) & !1;
				z += zeros;

				let pos = BlockInChunkPos::new(x as u8, y as u8, z as u8);
				let block = chunk.blocks[pos];
				let block_model = block_models.get(&block.id).unwrap_or_else(|| {
					panic!("tried to get the model of block with id {:?}", block.id)
				});
				for face in Face::all() {
					// see previous comment about this
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

pub fn chunk_padding_from_neighbour_chunks(neighbour_chunks: FaceMap<&Chunk>) -> ChunkPadding {
	let mut chunk_padding =
		FaceMap::from_map(|_| Box::new([[Air::BLOCK; CHUNK_LENGTH]; CHUNK_LENGTH]));

	macro_rules! neighbours {
		($(($a:ident, $b:ident) in ($axis:expr, $axis_name:ident)
		=> [$x:expr, $y:expr, $z:expr]);* $(;)?) => {
			$(
			for $a in 0..CHUNK_LENGTH {
				for $b in 0..CHUNK_LENGTH {
					let mut pos = BlockInChunkPos::new($x, $y, $z);

					pos.$axis_name = CHUNK_LENGTH as u8 - 1;
					let block = neighbour_chunks[$axis.face_neg()].blocks[pos];
					chunk_padding[$axis.face_neg()][$a][$b] = block;

					pos.$axis_name = 0;
					let block = neighbour_chunks[$axis.face_pos()].blocks[pos];
					chunk_padding[$axis.face_pos()][$a][$b] = block;
				}
			}
			)*
		};
	}
	neighbours! {
		(y, z) in (Axis::X, x) => [0, y as u8, z as u8];
		(x, z) in (Axis::Y, y) => [x as u8, 0, z as u8];
		(x, y) in (Axis::Z, z) => [x as u8, y as u8, 0];
	}
	chunk_padding
}

/// Will generate 3 bitmasks for this chunk, one for each axis.
/// The bits just mean whether a cullable block is there or not.
/// The 0th bit is an edge block of the negative facing neighbour chunk,
/// while the 33rd bit is the edge block of the opposite chunk,
/// and all bits in between (1st to 32nd inclusive) are the current chunk.
fn get_blocks_bitmask(
	chunk: &Chunk,
	block_models: &HashMap<BlockId, BlockModel<usize>>,
	chunk_padding: ChunkPadding,
) -> (Box<AxisMap<ChunkArray2D<u64>>>, Box<ChunkArray2D<u32>>) {
	// start out with a completely empty mask
	let mut blocks_mask = <AxisMap<ChunkArray2D<u64>>>::default();
	let mut non_culled_mask = <ChunkArray2D<u32>>::default();

	let inner_time = Instant::now();
	// fill in the current chunk
	for (pos, block) in chunk.blocks.iter_xyz() {
		let BlockInChunkPos { x, y, z } = pos;
		let [x, y, z] = [x as usize, y as usize, z as usize];
		if block_models[&block.id].should_cull {
			blocks_mask[Axis::X][y][z] |= 1 << (x + 1);
			blocks_mask[Axis::Y][x][z] |= 1 << (y + 1);
			blocks_mask[Axis::Z][x][y] |= 1 << (z + 1);
		} else if block.id != Air::BLOCK_ID {
			non_culled_mask[x][y] |= 1 << z;
		}
	}
	crate::bench::push_time(BenchName::BitMaskInner, inner_time.elapsed());

	let border_time = Instant::now();
	// fill in the edges of the neighbouring chunks
	macro_rules! neighbours {
		($(($a:ident, $b:ident) in ($axis:expr, $axis_name:ident));* $(;)?) => {
			$(
			for $a in 0..CHUNK_LENGTH {
				for $b in 0..CHUNK_LENGTH {
					let block = chunk_padding[$axis.face_neg()][$a][$b];
					if block_models[&block.id].should_cull {
						blocks_mask[$axis][$a][$b] |= 1;
					}
					let block = chunk_padding[$axis.face_pos()][$a][$b];
					if block_models[&block.id].should_cull {
						blocks_mask[$axis][$a][$b] |= 1 << (CHUNK_LENGTH + 1);
					}
				}
			}
			)*
		};
	}
	neighbours! {
		(y, z) in (Axis::X, x);
		(x, z) in (Axis::Y, y);
		(x, y) in (Axis::Z, z);
	}
	crate::bench::push_time(BenchName::BitMaskBorder, border_time.elapsed());

	// box the blocks_mask, so that its cheap to move around,
	// because its a *lot* of data
	// TODO check if this has better performance if this is put into a box earlier
	(Box::new(blocks_mask), Box::new(non_culled_mask))
}

fn bitmask_pos_to_world(axis: Axis, i: usize, j: usize, k: u32) -> BlockInChunkPos {
	// i and j are the two coordinates that are normally looped over.
	// i represent an axis lower than j.
	// k represent the axis along the bitmask itself (along `axis`).

	let [i, j, k] = [i as u8, j as u8, k as u8];

	let [x, y, z] = match axis {
		Axis::X => [k, i, j],
		Axis::Y => [i, k, j],
		Axis::Z => [i, j, k],
	};

	BlockInChunkPos { x, y, z }
}

fn create_face_mesh(info: BlockFaceInfo) -> Mesh {
	let BlockFaceInfo { face, data, pos } = info;

	let mut cube_mesh = Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::default(),
	);

	let positions = get_face_positions(face, data.min, data.max, pos);
	cube_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

	// in the future block models may define different uvs, this is temporary
	let Rect { min, max } = Rect::from_corners(Vec2::ZERO, Vec2::ONE);
	let Vec2 { x: x0, y: y0 } = min;
	let Vec2 { x: x1, y: y1 } = max;
	let uvs = vec![[x0, y0], [x0, y1], [x1, y1], [x1, y0]];
	cube_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

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

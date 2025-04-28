use super::combine_mesh::combine_meshes;
use crate::{
	axis::{Axis, AxisMap},
	bench::BenchName,
	block::{
		prelude::{Air, BlockTrait, BlockWithoutData},
		Block, BlockId,
	},
	block_model::{
		chunk_material::{ATTRIBUTE_RECT_WIDTH, ATTRIBUTE_START_TEXTURE_INDEX},
		BlockModel,
	},
	face::{Face, FaceMap},
	game_world::chunk::{Chunk, CHUNK_LENGTH},
	pos::BlockInChunkPos,
};
use bevy::{
	prelude::*,
	render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use itertools::Either;
use std::{collections::HashMap, time::Instant};

pub type ChunkArray2D<T> = [[T; CHUNK_LENGTH]; CHUNK_LENGTH];

pub type ChunkPadding = FaceMap<Box<ChunkArray2D<Block>>>;

#[derive(Clone, Copy)]
struct BlockFaceData {
	min: Vec3,
	max: Vec3,
	start_texture_index: u32,
	rect_width: u32,
}

#[derive(Debug, Default)]
pub struct FaceTextureIndices(Vec<u8>);

impl FaceTextureIndices {
	pub fn push_index(&mut self, index: u32) {
		// wgsl is guaranteed to use little-endian:
		// https://gpuweb.github.io/gpuweb/wgsl/#internal-value-layout
		self.0.extend_from_slice(&index.to_le_bytes());
	}

	pub fn len(&self) -> usize {
		self.0.len() / 4
	}

	pub fn into_bytes(self) -> Vec<u8> {
		self.0
	}
}

pub fn create_chunk_mesh(
	chunk: Chunk,
	chunk_padding: ChunkPadding,
	block_models: HashMap<BlockId, BlockModel<usize>>,
) -> (Mesh, FaceTextureIndices) {
	// benchmarking of creating the chunk mesh
	let start_time = Instant::now();

	// let bitmask_time = Instant::now();
	let (blocks_mask, non_culled_mask) = get_blocks_bitmask(&chunk, &block_models, chunk_padding);
	// crate::bench::push_time(BenchName::BitMask, bitmask_time.elapsed());

	let culled_blocks_mask = FaceMap::from_map(|face| {
		// `>> 1` because the negative chunk neighbour's block takes up 1 bit at the edge.
		// the cast to `u32` cuts of the positive chunk neighbour's bit.
		let culling = if face.is_pos() {
			|mask: u64| ((mask & !(mask >> 1)) >> 1) as u32
		} else {
			|mask: u64| ((mask & !(mask << 1)) >> 1) as u32
		};
		blocks_mask[face.axis()].map(|array| array.map(culling))
	});

	// extract the actual BlockFaceData from these bitmasks and the chunk data
	let mut all_faces = FaceMap::from_map(|_| Vec::new());
	let mut face_texture_indices = FaceTextureIndices::default();
	let greedy_mask = build_greedy_bitmask(&culled_blocks_mask);
	insert_greedy_face_data(
		&mut all_faces,
		greedy_mask,
		&mut face_texture_indices,
		&chunk,
		&block_models,
	);
	insert_non_culled_face_data(
		&mut all_faces,
		non_culled_mask,
		&mut face_texture_indices,
		&chunk,
		&block_models,
	);

	let meshes = all_faces
		.iter_face()
		.flat_map(|(face, datas)| datas.iter().map(move |&data| create_face_mesh(face, data)));

	let combined_meshes = combine_meshes(meshes);

	crate::bench::push_time(BenchName::CreateChunkMesh, start_time.elapsed());

	(combined_meshes, face_texture_indices)
}

pub fn chunk_padding_from_neighbour_chunks(neighbour_chunks: FaceMap<&Chunk>) -> ChunkPadding {
	let mut chunk_padding =
		FaceMap::from_map(|_| Box::new([[Air::BLOCK; CHUNK_LENGTH]; CHUNK_LENGTH]));

	macro_rules! neighbours {
		($(($a:ident, $b:ident) in ($axis:expr, $with_axis:ident)
		=> [$x:expr, $y:expr, $z:expr]);* $(;)?) => {
			$(
			for $a in 0..CHUNK_LENGTH {
				for $b in 0..CHUNK_LENGTH {
					let pos = BlockInChunkPos::try_new($x, $y, $z).unwrap();

					// pos is already 0 at $axis
					// let pos = pos.$with_axis(0).unwrap();
					let block = neighbour_chunks[$axis.face_pos()].blocks[pos];
					chunk_padding[$axis.face_pos()][$a][$b] = block;

					let pos = pos.$with_axis(CHUNK_LENGTH - 1).unwrap();
					let block = neighbour_chunks[$axis.face_neg()].blocks[pos];
					chunk_padding[$axis.face_neg()][$a][$b] = block;
				}
			}
			)*
		};
	}
	neighbours! {
		(y, z) in (Axis::X, with_x) => [0, y, z];
		(x, z) in (Axis::Y, with_y) => [x, 0, z];
		(x, y) in (Axis::Z, with_z) => [x, y, 0];
	}
	chunk_padding
}

fn build_greedy_bitmask(
	culled_blocks_mask: &FaceMap<ChunkArray2D<u32>>,
) -> Box<FaceMap<ChunkArray2D<u32>>> {
	let mut greedy_mask = Box::<FaceMap<ChunkArray2D<u32>>>::default();
	for (face, array2d) in culled_blocks_mask.iter_face() {
		for (i, array) in array2d.iter().enumerate() {
			for (j, mask) in array.iter().enumerate() {
				let mut mask = *mask;
				let mut k = 0;
				while mask != 0 {
					let zeros = mask.trailing_zeros();
					mask = (mask >> zeros) & !1;
					k += zeros;

					// PERF swapping i and j for the z axis makes it so each bitmask
					// is oriented horizontally instead of vertically.
					// this is better for performance, since horizontal strips
					// are more common than vertical ones.
					// the x axis is already oriented in the horizontal direction,
					// and there is no bias for the y axis.
					match face.axis() {
						Axis::X | Axis::Y => greedy_mask[face][k as usize][i] |= 1 << j,
						Axis::Z => greedy_mask[face][k as usize][j] |= 1 << i,
					}
				}
			}
		}
	}
	greedy_mask
}

fn insert_greedy_face_data(
	all_faces: &mut FaceMap<Vec<BlockFaceData>>,
	mut greedy_mask: Box<FaceMap<ChunkArray2D<u32>>>,
	face_texture_indices: &mut FaceTextureIndices,
	chunk: &Chunk,
	block_models: &HashMap<BlockId, BlockModel<usize>>,
) {
	for (face, array2d) in greedy_mask.iter_mut_face() {
		for (i, array) in array2d.iter_mut().enumerate() {
			for j in 0..array.len() {
				let mut mask_copy = array[j];
				let mut k = 0;
				while mask_copy != 0 {
					let zeros = mask_copy.trailing_zeros();
					mask_copy >>= zeros;
					k += zeros;

					let ones = mask_copy.trailing_ones();
					mask_copy = mask_copy.checked_shr(ones).unwrap_or(0);
					let from = k;
					k += ones;

					// this entire strip of blocks, as a bitmask.
					// `<<` doesnt overflow in the way you would expect,
					// so we use `checked_shl` here instead.
					// `from != 32`, because otherwise `mask_copy == 0`, so the
					// left shift there will never overflow (in any problematic way).
					let ones_exp2 = 1_u32.checked_shl(ones).unwrap_or(0);
					let strip_mask = (ones_exp2.wrapping_sub(1)) << from;

					// expand the strip into a rectangle,
					// and unset any bits along the way
					array[j] &= !strip_mask;
					let mut strip_expand = 0;
					for next_mask in &mut array[(j + 1)..] {
						if *next_mask & strip_mask == strip_mask {
							strip_expand += 1;
							*next_mask &= !strip_mask;
						} else {
							break;
						}
					}

					#[rustfmt::skip]
					let from_pos = bitmask_pos_to_world(
						face.axis(),
						[i, j, from as usize]
					).unwrap();

					let to_pos = bitmask_pos_to_world(
						face.axis(),
						[i, j + strip_expand, (from + ones - 1) as usize],
					)
					.unwrap();

					let from_vec3 = Vec3::from(from_pos);
					// expand `to` out orthogonally to the faces normal,
					// because: if `from_pos == to_pos` you still want
					// a face that is 1x1 instead of 0x0
					let to_vec3 = Vec3::from(to_pos) + (1 - face.normal().abs()).as_vec3();
					let to_vec3 = if face.is_pos() {
						to_vec3 + face.normal().as_vec3()
					} else {
						to_vec3
					};

					let (start_texture_index, rect_width) = push_face_texture_indices(
						face_texture_indices,
						from_pos,
						to_pos,
						face,
						chunk,
						block_models,
					);
					let data = BlockFaceData {
						min: from_vec3,
						max: to_vec3,
						start_texture_index,
						rect_width,
					};
					all_faces[face].push(data);
				}
			}
		}
	}
}

fn push_face_texture_indices(
	face_texture_indices: &mut FaceTextureIndices,
	from_pos: BlockInChunkPos,
	to_pos: BlockInChunkPos,
	face: Face,
	chunk: &Chunk,
	block_models: &HashMap<BlockId, BlockModel<usize>>,
) -> (u32, u32) {
	let axis = face.axis();
	let start_texture_index = face_texture_indices.len();
	// using inclusive ranges here would be nicer,
	// but they dont implement ExactSizeIterator,
	// so they dont have a len() function.
	#[rustfmt::skip]
	let [iter_i, iter_j] = match axis {
		Axis::X => [from_pos.z()..(to_pos.z() + 1), from_pos.y()..(to_pos.y() + 1)],
		Axis::Y => [from_pos.x()..(to_pos.x() + 1), from_pos.z()..(to_pos.z() + 1)],
		Axis::Z => [from_pos.x()..(to_pos.x() + 1), from_pos.y()..(to_pos.y() + 1)],
	};
	// for some directions an iter has to be reversed.
	// i dont really know why, i just kinda tried it out.
	let [iter_i, iter_j] = match face {
		Face::Right | Face::Forward => [Either::Right(iter_i.rev()), Either::Right(iter_j.rev())],
		Face::Up => [Either::Left(iter_i), Either::Left(iter_j)],
		_ => [Either::Left(iter_i), Either::Right(iter_j.rev())],
	};
	let k = from_pos.get(axis);
	for j in iter_j {
		for i in iter_i.clone() {
			let pos = BlockInChunkPos::try_from(match axis {
				Axis::X => [k, j, i],
				Axis::Y => [i, k, j],
				Axis::Z => [i, j, k],
			})
			.unwrap();
			let block = chunk.blocks[pos];
			let block_model = block_models.get(&block.id).unwrap_or_else(|| {
				panic!("tried to get the model of block with id {:?}", block.id)
			});
			let [face_data] = block_model.faces[face].as_slice() else {
				panic!(
					"every culled block must consist of only single faces, \
					but block with id {:?} had {} faces",
					block.id,
					block_model.faces[face].len()
				);
			};
			face_texture_indices.push_index(face_data.side as u32);
		}
	}
	(start_texture_index as u32, iter_i.len() as u32)
}

fn insert_non_culled_face_data(
	all_faces: &mut FaceMap<Vec<BlockFaceData>>,
	non_culled_mask: Box<ChunkArray2D<u32>>,
	face_texture_indices: &mut FaceTextureIndices,
	chunk: &Chunk,
	block_models: &HashMap<BlockId, BlockModel<usize>>,
) {
	for (x, array) in non_culled_mask.iter().enumerate() {
		for (y, mask) in array.iter().enumerate() {
			let mut mask = *mask;
			let mut z = 0;
			while mask != 0 {
				let zeros = mask.trailing_zeros();
				mask = (mask >> zeros) & !1;
				z += zeros;

				let pos = BlockInChunkPos::try_new(x, y, z as usize).unwrap();
				let block = chunk.blocks[pos];
				let block_model = block_models.get(&block.id).unwrap_or_else(|| {
					panic!("tried to get the model of block with id {:?}", block.id)
				});
				let pos = Vec3::from(pos);
				for face in Face::all() {
					for &data in &block_model.faces[face] {
						let face_index = face_texture_indices.len() as u32;
						face_texture_indices.push_index(data.side as u32);
						let face_data = BlockFaceData {
							min: data.min + pos,
							max: data.max + pos,
							start_texture_index: face_index,
							rect_width: 1,
						};
						all_faces[face].push(face_data);
					}
				}
			}
		}
	}
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
	let mut blocks_mask = Box::<AxisMap<ChunkArray2D<u64>>>::default();
	let mut non_culled_mask = Box::<ChunkArray2D<u32>>::default();

	// let inner_time = Instant::now();
	// fill in the current chunk
	for (pos, block) in chunk.blocks.iter_xyz() {
		let [x, y, z] = [pos.x(), pos.y(), pos.z()];
		if block_models[&block.id].should_cull {
			blocks_mask[Axis::X][y][z] |= 1 << (x + 1);
			blocks_mask[Axis::Y][x][z] |= 1 << (y + 1);
			blocks_mask[Axis::Z][x][y] |= 1 << (z + 1);
		} else if block.id != Air::BLOCK_ID {
			non_culled_mask[x][y] |= 1 << z;
		}
	}
	// crate::bench::push_time(BenchName::BitMaskInner, inner_time.elapsed());

	// let border_time = Instant::now();
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
	// crate::bench::push_time(BenchName::BitMaskBorder, border_time.elapsed());

	(blocks_mask, non_culled_mask)
}

fn bitmask_pos_to_world(axis: Axis, [i, j, k]: [usize; 3]) -> Option<BlockInChunkPos> {
	let [x, y, z] = match axis {
		Axis::X => [i, j, k],
		Axis::Y => [j, i, k],
		// z axis doesnt follow pattern, because (input) i and j are swapped.
		// the correct order would be [j, k, i] without the swap.
		Axis::Z => [k, j, i],
	};

	BlockInChunkPos::try_new(x, y, z)
}

fn create_face_mesh(face: Face, data: BlockFaceData) -> Mesh {
	let mut cube_mesh = Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::default(),
	);

	let positions = get_face_positions(face, data.min, data.max);
	cube_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

	// in the future block models may define different uvs, this is temporary
	let Vec2 { x: x0, y: y0 } = Vec2::ZERO;
	let Vec2 { x: x1, y: y1 } = match face.axis() {
		Axis::X => Vec2::new(data.max.z - data.min.z, data.max.y - data.min.y),
		Axis::Y => Vec2::new(data.max.x - data.min.x, data.max.z - data.min.z),
		Axis::Z => Vec2::new(data.max.x - data.min.x, data.max.y - data.min.y),
	};
	let uvs = vec![[x0, y0], [x0, y1], [x1, y1], [x1, y0]];
	cube_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

	// all vertices use the same starting value and width.
	// the offset comes from the uv position.
	let start_texture_indices = vec![data.start_texture_index; 4];
	cube_mesh.insert_attribute(ATTRIBUTE_START_TEXTURE_INDEX, start_texture_indices);
	let rect_widths = vec![data.rect_width; 4];
	cube_mesh.insert_attribute(ATTRIBUTE_RECT_WIDTH, rect_widths);

	// since we are constructing a single face, we already
	// know the exact indeces for the 2 triangles
	let tris = vec![0, 1, 3, 2, 3, 1];
	cube_mesh.insert_indices(Indices::U32(tris));

	cube_mesh
}

fn get_face_positions(face: Face, min: Vec3, max: Vec3) -> Vec<[f32; 3]> {
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

	match face {
		Face::Right => min_max!([1, 1, 1], [1, 0, 1], [1, 0, 0], [1, 1, 0]),
		Face::Left => min_max!([0, 1, 0], [0, 0, 0], [0, 0, 1], [0, 1, 1]),
		Face::Up => min_max!([0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]),
		Face::Down => min_max!([0, 0, 1], [0, 0, 0], [1, 0, 0], [1, 0, 1]),
		Face::Back => min_max!([0, 1, 1], [0, 0, 1], [1, 0, 1], [1, 1, 1]),
		Face::Forward => min_max!([1, 1, 0], [1, 0, 0], [0, 0, 0], [0, 1, 0]),
	}
}

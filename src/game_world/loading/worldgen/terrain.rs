use super::Seed;
use crate::{
	block::prelude::*,
	game_world::chunk::{BlockArray, Chunk, GenerationStage, IsLoaded, CHUNK_LENGTH},
	pos::{BlockInChunkPos, ChunkPos},
};
use noise::{NoiseFn, Perlin};

/// will create a new chunk with the [`Terrain`](GenerationStage::Terrain) GenerationStage.
pub fn generate_chunk_terrain(chunk_pos: ChunkPos, seed: Seed, loaded: IsLoaded) -> Chunk {
	let mut chunk = Chunk {
		blocks: Box::new(BlockArray::ALL_AIR),
		loaded,
		// this hasnt been generated yet, but will be by the rest of the function
		generation_state: GenerationStage::Terrain,
	};

	let perlin = Perlin::new(seed);

	for x in 0..CHUNK_LENGTH as u8 {
		for z in 0..CHUNK_LENGTH as u8 {
			let world_pos = chunk_pos.to_block_pos();
			let x_block = x as i32 + world_pos.x;
			let z_block = z as i32 + world_pos.z;
			let y_block = get_height_at(x_block, z_block, &perlin);
			let y_in_chunk = y_block - world_pos.y;
			let clamped = (y_in_chunk + 1).clamp(0, CHUNK_LENGTH as i32) as u8;

			for y in 0..clamped {
				let diff = y as i32 - y_in_chunk;
				let mut block = match diff {
					..-3 => Stone::BLOCK,
					-3..0 => Dirt::BLOCK,
					0 => GrassBlock::BLOCK,
					// should be unreachable because y doesnt go this high
					1.. => continue,
				};
				let block_pos = [x_block, y as i32 + world_pos.y, z_block];
				if is_cave_air(block_pos, &perlin) {
					block = Air::BLOCK;
				} else if block == Stone::BLOCK && is_random_cobblestone(block_pos, &perlin) {
					block = Cobblestone::BLOCK;
				}
				let pos = BlockInChunkPos::new(x, y, z);
				chunk.blocks[pos] = block;
			}
		}
	}

	chunk
}

pub fn get_height_at(x: i32, z: i32, perlin: &Perlin) -> i32 {
	const HORIZONTAL_STRETCH_0: f64 = 74.379;
	const VERTICAL_STRETCH_0: f64 = 23.748;
	const HORIZONTAL_STRETCH_1: f64 = 21.174;
	const VERTICAL_STRETCH_1: f64 = 4.849;

	let x0 = x as f64 / HORIZONTAL_STRETCH_0;
	let z0 = z as f64 / HORIZONTAL_STRETCH_0;
	let y0 = perlin.get([x0, z0]) * VERTICAL_STRETCH_0;
	let x1 = x as f64 / HORIZONTAL_STRETCH_1;
	let z1 = z as f64 / HORIZONTAL_STRETCH_1;
	let y1 = perlin.get([x1, z1]) * VERTICAL_STRETCH_1;
	(y0 + y1) as i32
}

fn is_random_cobblestone([x, y, z]: [i32; 3], perlin: &Perlin) -> bool {
	const STRETCH: [f64; 3] = [33.6521, 20.4731, 26.9035];
	const THRESHOLD: f64 = 0.5;
	let [x, y, z] = [x as f64, y as f64, z as f64];
	perlin.get([x * STRETCH[0], y * STRETCH[1], z * STRETCH[2]]) > THRESHOLD
}

fn is_cave_air([x, y, z]: [i32; 3], perlin: &Perlin) -> bool {
	const STRETCH: [f64; 3] = [0.058, 0.053, 0.050];
	const THRESHOLD: f64 = 0.5;
	let [x, y, z] = [x as f64, y as f64, z as f64];
	perlin.get([x * STRETCH[0], y * STRETCH[1], z * STRETCH[2]]) > THRESHOLD
}

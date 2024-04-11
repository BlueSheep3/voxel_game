use crate::{
	block::prelude::*,
	game_world::chunk::{BlockArray, Chunk, IsLoaded, CHUNK_LENGTH},
	pos::ChunkPos,
};
use noise::{NoiseFn, Perlin};

pub type Seed = u32;

pub fn generate_chunk(chunk_pos: ChunkPos, seed: Seed, loaded: IsLoaded) -> Chunk {
	let mut chunk = Chunk {
		blocks: Box::new(BlockArray::ALL_AIR),
		loaded,
	};

	let perlin = Perlin::new(seed);

	for x in 0..CHUNK_LENGTH {
		for z in 0..CHUNK_LENGTH {
			let world_pos = chunk_pos.to_block_pos();
			let x_block = x as i32 + world_pos.0.x;
			let z_block = z as i32 + world_pos.0.z;
			let y_block = get_height_at(x_block, z_block, &perlin);
			let y_in_chunk = y_block - world_pos.0.y;

			if y_in_chunk < 0 {
				continue;
			} else if y_in_chunk >= CHUNK_LENGTH as i32 {
				for y in 0..CHUNK_LENGTH {
					chunk.blocks[[x, y, z]] = Dirt::BLOCK;
				}
			} else {
				chunk.blocks[[x, y_in_chunk as usize, z]] = GrassBlock::BLOCK;
				for y in 0..(y_in_chunk as usize) {
					chunk.blocks[[x, y, z]] = Dirt::BLOCK;
				}
			}
		}
	}

	chunk
}

const HORIZONTAL_STRETCH: f64 = 74.379;
const VERTICAL_STRETCH: f64 = 23.748;

fn get_height_at(x: i32, z: i32, perlin: &Perlin) -> i32 {
	let x = x as f64 / HORIZONTAL_STRETCH;
	let z = z as f64 / HORIZONTAL_STRETCH;
	(perlin.get([x, z]) * VERTICAL_STRETCH) as i32
}

pub fn get_height_at_with_seed(x: i32, z: i32, seed: Seed) -> i32 {
	let perlin = Perlin::new(seed);
	get_height_at(x, z, &perlin)
}

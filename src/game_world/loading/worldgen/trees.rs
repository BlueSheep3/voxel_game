use crate::{
	block::prelude::*,
	game_world::{
		chunk::{Chunk, GenerationStage, IsLoaded, CHUNK_LENGTH},
		GameWorld,
	},
	pos::{BlockPos, ChunkPos},
};
use bevy::math::IVec3;
use rand::{prelude::StdRng, Rng, SeedableRng};

pub fn generate_trees(world: &mut GameWorld, chunk_pos: ChunkPos) {
	// TODO generate multiple trees instead of just one

	let x = get_random(chunk_pos.to_block_pos(), 7832957017391).gen_range(0..CHUNK_LENGTH);
	let z = get_random(chunk_pos.to_block_pos(), 9870402726984).gen_range(0..CHUNK_LENGTH);
	let Some(chunk) = world.chunks.get_mut(&chunk_pos) else {
		bevy::log::error!(
			"trying to generate trees in a chunk that doesnt exist (at {})",
			chunk_pos
		);
		return;
	};
	let Some(y) = find_y_of_grass_block(x, z, chunk) else {
		// this will happen if this chunk is in a cave or just the sky
		return;
	};
	chunk.generation_state = GenerationStage::Trees;

	let block_pos = chunk_pos.to_block_pos() + IVec3::new(x as i32, y as i32, z as i32);
	place_block_at(world, block_pos, Dirt::BLOCK);
	let height = get_random(block_pos, 8749103747).gen_range(4..7);
	for i in 1..=height {
		let pos = block_pos + IVec3::new(0, i, 0);
		place_block_at(world, pos, Log::BLOCK);
	}
	let highest_pos = block_pos + IVec3::new(0, height, 0);

	// TODO place leaves in a ~beatiful~ way
	#[rustfmt::skip]
	let leaf_offsets = [
		[-1,  0, -1], [ 0,  0, -1], [ 1,  0, -1],
		[-1,  0,  0],               [ 1,  0,  0],
		[-1,  0,  1], [ 0,  0,  1], [ 1,  0,  1],

		[-1,  1, -1], [ 0,  1, -1], [ 1,  1, -1],
		[-1,  1,  0], [ 0,  1,  0], [ 1,  1,  0],
		[-1,  1,  1], [ 0,  1,  1], [ 1,  1,  1],
	];
	for offset in leaf_offsets {
		let pos = highest_pos + IVec3::from_array(offset);
		place_block_at(world, pos, Leaves::BLOCK);
	}
}

fn place_block_at(world: &mut GameWorld, block_pos: BlockPos, new_block: Block) {
	if let Some(block) = world.get_block_at_mut(block_pos) {
		*block = new_block;
	} else {
		let chunk_pos = block_pos.to_chunk_pos();
		let loaded = IsLoaded::NOT_LOADED;
		let chunk = super::generate_chunk_terrain(chunk_pos, world.seed, loaded);
		world.chunks.insert(chunk_pos, chunk);
		let Some(block) = world.get_block_at_mut(block_pos) else {
			bevy::log::error!(
				"somehow, generating a chunk at {} didnt allow placing a block at {}; skipping block placement",
				chunk_pos, block_pos
			);
			return;
		};
		*block = new_block;
	}
}

fn get_random(block_pos: BlockPos, salt: u64) -> StdRng {
	let [x, y, z] = [
		block_pos.0.x as u64,
		block_pos.0.y as u64,
		block_pos.0.z as u64,
	];
	let seed = x.wrapping_add(y << 6).wrapping_add(z << 12);
	let seed = seed.wrapping_add(salt);
	StdRng::seed_from_u64(seed)
}

// coordinates are relative to chunk
fn find_y_of_grass_block(x: usize, z: usize, chunk: &Chunk) -> Option<usize> {
	// maybe this should just use the same height map as terrain gen?

	for y in 0..CHUNK_LENGTH {
		let pos = [x, y, z];
		if chunk.blocks[pos] == GrassBlock::BLOCK {
			return Some(y);
		}
	}
	None
}

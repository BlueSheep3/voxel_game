mod terrain;
mod trees;

use self::terrain::get_height_at;
use crate::{
	game_world::{
		chunk::{GenerationStage, IsLoaded},
		GameWorld,
	},
	pos::ChunkPos,
};
use noise::Perlin;

pub use self::terrain::generate_chunk_terrain;
pub use self::trees::generate_trees;

pub type Seed = u32;

// guarantees that `world.chunks.get(&pos)` will be `Some`
pub fn fully_generate_chunk(world: &mut GameWorld, pos: ChunkPos, loaded: IsLoaded) {
	let chunk = generate_chunk_terrain(pos, world.seed, loaded);
	world.chunks.insert(pos, chunk);
	generate_trees(world, pos);
}

/// will perform all neccessary generation steps needed to get the
/// chunk to [`GenerationStage::COMPLETE`], and will say that it is
/// not loaded if a new chunk is created via this function.
pub fn continue_generation_of_chunk(world: &mut GameWorld, pos: ChunkPos) {
	let Some(chunk) = world.chunks.get(&pos) else {
		fully_generate_chunk(world, pos, IsLoaded::NOT_LOADED);
		return;
	};

	match chunk.generation_state {
		GenerationStage::Nothing => fully_generate_chunk(world, pos, IsLoaded::NOT_LOADED),
		GenerationStage::Terrain => generate_trees(world, pos),
		GenerationStage::COMPLETE => (),
	}
}

pub fn get_height_at_with_seed(x: i32, z: i32, seed: Seed) -> i32 {
	let perlin = Perlin::new(seed);
	get_height_at(x, z, &perlin)
}

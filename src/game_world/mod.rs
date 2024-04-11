pub mod chunk;
mod loading;

use self::{
	chunk::{Chunk, CHUNK_LENGTH},
	loading::worldgen,
};
use crate::{
	block::Block,
	pos::{BlockPos, ChunkPos, Vec3Utils},
	savedata, GlobalState,
};
use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

pub use self::loading::worldgen::get_height_at_with_seed;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((chunk::ChunkPlugin, loading::LoadingPlugin))
			// TODO load world in a more sensible way
			.insert_resource(savedata::load_game_world("debug_world").unwrap())
			.add_systems(
				Update,
				(save_game_world, load_game_world).run_if(in_state(GlobalState::InWorld)),
			);
	}
}

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize)]
pub struct GameWorld {
	/// a map from a position in chunk space to a chunk
	pub chunks: HashMap<ChunkPos, Chunk>,
	/// a value used to generate new chunks
	pub seed: worldgen::Seed,
	// TODO store entities and other stuff here
}

fn save_game_world(input: Res<ButtonInput<KeyCode>>, game_world: Res<GameWorld>) {
	if input.just_pressed(KeyCode::KeyO) {
		savedata::save_game_world("debug_world", &game_world).unwrap();
	}
}

// TODO load world in a more sensible way
fn load_game_world(input: Res<ButtonInput<KeyCode>>, mut game_world: ResMut<GameWorld>) {
	if input.just_pressed(KeyCode::KeyL) {
		*game_world = savedata::load_game_world("debug_world").unwrap();
	}
}

impl GameWorld {
	#[allow(dead_code)]
	pub fn get_chunk_at_world_pos(&self, pos: Vec3) -> Option<&Chunk> {
		self.chunks.get(&pos.to_chunk_pos())
	}

	#[allow(dead_code)]
	pub fn get_chunk_at_world_pos_mut(&mut self, pos: Vec3) -> Option<&mut Chunk> {
		self.chunks.get_mut(&pos.to_chunk_pos())
	}

	/// gets the block at the given integer position in block space
	pub fn get_block_at(&self, pos: BlockPos) -> Option<&Block> {
		let chunk_pos = pos.to_chunk_pos();
		let chunk = self.chunks.get(&chunk_pos)?;
		let IVec3 { x, y, z } = pos.0.rem_euclid(IVec3::splat(CHUNK_LENGTH as i32));
		let [x, y, z] = [x as usize, y as usize, z as usize];
		Some(&chunk.blocks[[x, y, z]])
	}

	pub fn get_block_at_mut(&mut self, pos: BlockPos) -> Option<&mut Block> {
		let chunk_pos = pos.to_chunk_pos();
		let chunk = self.chunks.get_mut(&chunk_pos)?;
		let IVec3 { x, y, z } = pos.0.rem_euclid(IVec3::splat(CHUNK_LENGTH as i32));
		let [x, y, z] = [x as usize, y as usize, z as usize];
		Some(&mut chunk.blocks[[x, y, z]])
	}
}

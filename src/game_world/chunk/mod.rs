mod block_array;
mod generation_stage;
mod is_loaded;
mod render;

use crate::pos::ChunkPos;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use self::{
	block_array::{BlockArray, CHUNK_LENGTH},
	generation_stage::GenerationStage,
	is_loaded::IsLoaded,
};

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(render::RenderPlugin)
			.add_event::<ChunkUpdateEvent>();
	}
}

#[derive(Event)]
pub struct ChunkUpdateEvent {
	pub chunk_pos: ChunkPos,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
	pub blocks: Box<BlockArray>,
	#[serde(skip)]
	pub loaded: IsLoaded,
	pub generation_state: GenerationStage,
}

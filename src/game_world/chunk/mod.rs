mod render;

use crate::{block::prelude::*, pos::ChunkPos};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(render::RenderPlugin)
			.add_event::<ChunkUpdateEvent>();
	}
}

pub const CHUNK_LENGTH: usize = 32;
use CHUNK_LENGTH as LEN;

#[derive(Event)]
pub struct ChunkUpdateEvent {
	pub chunk_pos: ChunkPos,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
	pub blocks: Box<BlockArray>,
	#[serde(skip)]
	pub loaded: IsLoaded,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockArray([[[Block; LEN]; LEN]; LEN]);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IsLoaded {
	// NOTE current impls have a lot of overlap, since the IsLoaded
	// will be more complicated in the future
	is_visible: bool,
}

impl BlockArray {
	pub const ALL_AIR: Self = Self([[[Air::BLOCK; LEN]; LEN]; LEN]);

	pub fn iter_xyz(&self) -> impl Iterator<Item = (UVec3, Block)> + '_ {
		(0..LEN)
			.flat_map(|x| (0..LEN).flat_map(move |y| (0..LEN).map(move |z| [x, y, z])))
			.map(|[x, y, z]| (UVec3::new(x as u32, y as u32, z as u32), self.0[x][y][z]))
	}
}

impl IsLoaded {
	pub const PLAYER_LOADED: Self = Self { is_visible: true };
	pub const NOT_LOADED: Self = Self { is_visible: false };

	pub fn is_player_loaded(&self) -> bool {
		self.is_visible
	}

	pub fn set_player_loaded(&mut self, value: bool) {
		self.is_visible = value;
	}
}

impl Default for IsLoaded {
	fn default() -> Self {
		Self::NOT_LOADED
	}
}

impl Index<[usize; 3]> for BlockArray {
	type Output = Block;

	fn index(&self, [x, y, z]: [usize; 3]) -> &Self::Output {
		&self.0[x][y][z]
	}
}

impl IndexMut<[usize; 3]> for BlockArray {
	fn index_mut(&mut self, [x, y, z]: [usize; 3]) -> &mut Self::Output {
		&mut self.0[x][y][z]
	}
}

impl Index<UVec3> for BlockArray {
	type Output = Block;

	fn index(&self, UVec3 { x, y, z }: UVec3) -> &Self::Output {
		&self.0[x as usize][y as usize][z as usize]
	}
}

impl IndexMut<UVec3> for BlockArray {
	fn index_mut(&mut self, UVec3 { x, y, z }: UVec3) -> &mut Self::Output {
		&mut self.0[x as usize][y as usize][z as usize]
	}
}

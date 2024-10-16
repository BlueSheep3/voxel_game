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
	pub generation_state: GenerationStage,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockArray([[[Block; LEN]; LEN]; LEN]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IsLoaded {
	is_simple_loaded: bool,
	is_visible: bool,
}

/// describes how much of the chunk has **already** been generated.<br>
/// for example: has the basic shape of the terrain been generated? or the trees?
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenerationStage {
	/// nothing of the chunk has been generated.
	/// it is literally empty.
	#[default]
	Nothing,
	/// the basic shape of it has been generated.
	/// for example: the dirt and stone blocks, including the cave shapes.
	Terrain,
	/// the trees have been placed in.
	Trees,
}

impl GenerationStage {
	pub const COMPLETE: Self = Self::Trees;
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
	pub const SIMPLE_LOADED: Self = Self {
		is_simple_loaded: true,
		is_visible: false,
	};
	pub const NOT_LOADED: Self = Self {
		is_simple_loaded: false,
		is_visible: false,
	};
	#[allow(dead_code)]
	pub const VISIBLE: Self = Self {
		is_simple_loaded: true,
		is_visible: true,
	};

	// having these methods feels kind of reduntant

	pub fn is_simple_loaded(&self) -> bool {
		self.is_simple_loaded
	}

	pub fn set_simple_loaded(&mut self, value: bool) {
		self.is_simple_loaded = value;
		// if a chunk isn't loaded at all it should also be invisible
		if !value {
			self.is_visible = false;
		}
	}

	pub fn is_visible(&self) -> bool {
		self.is_visible
	}

	pub fn set_visible(&mut self, value: bool) {
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

use super::{BlockInChunkPos, ChunkPos};
use crate::{face::Face, game_world::chunk::CHUNK_LENGTH};
use bevy::math::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

impl BlockPos {
	pub fn new(x: i32, y: i32, z: i32) -> Self {
		Self { x, y, z }
	}

	pub fn to_world_pos(self) -> Vec3 {
		Vec3::new(self.x as f32, self.y as f32, self.z as f32)
	}

	pub fn to_chunk_pos(self) -> ChunkPos {
		ChunkPos::new(
			(self.x as f32 / CHUNK_LENGTH as f32).floor() as i32,
			(self.y as f32 / CHUNK_LENGTH as f32).floor() as i32,
			(self.z as f32 / CHUNK_LENGTH as f32).floor() as i32,
		)
	}

	pub fn to_block_in_chunk_pos(self) -> BlockInChunkPos {
		BlockInChunkPos::new(
			self.x.rem_euclid(CHUNK_LENGTH as i32) as u8,
			self.y.rem_euclid(CHUNK_LENGTH as i32) as u8,
			self.z.rem_euclid(CHUNK_LENGTH as i32) as u8,
		)
	}

	/// gets all block positions that touch this block, meaning diagonals are not counted
	pub fn neighbours(self) -> impl Iterator<Item = Self> {
		Face::all()
			.into_iter()
			.map(move |face| self + face.normal())
	}
}

impl From<IVec3> for BlockPos {
	fn from(value: IVec3) -> Self {
		let IVec3 { x, y, z } = value;
		Self { x, y, z }
	}
}

impl From<BlockPos> for IVec3 {
	fn from(value: BlockPos) -> Self {
		let BlockPos { x, y, z } = value;
		Self { x, y, z }
	}
}

impl Add<IVec3> for BlockPos {
	type Output = Self;

	fn add(self, rhs: IVec3) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

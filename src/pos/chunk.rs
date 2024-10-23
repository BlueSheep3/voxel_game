use super::BlockPos;
use crate::{face::Face, game_world::chunk::CHUNK_LENGTH};
use bevy::math::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkPos {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

impl ChunkPos {
	pub fn new(x: i32, y: i32, z: i32) -> Self {
		Self { x, y, z }
	}

	pub fn to_block_pos(self) -> BlockPos {
		BlockPos::new(
			self.x * CHUNK_LENGTH as i32,
			self.y * CHUNK_LENGTH as i32,
			self.z * CHUNK_LENGTH as i32,
		)
	}

	pub fn to_world_pos(self) -> Vec3 {
		Vec3::new(
			self.x as f32 * CHUNK_LENGTH as f32,
			self.y as f32 * CHUNK_LENGTH as f32,
			self.z as f32 * CHUNK_LENGTH as f32,
		)
	}

	/// gets all chunk positions that touch this chunk, meaning diagonals are not counted
	pub fn neighbours(self) -> impl Iterator<Item = Self> {
		Face::all().map(move |face| self + face.normal())
	}

	pub fn distance_squared(self, rhs: Self) -> u32 {
		((self.x - rhs.x).pow(2) + (self.y - rhs.y).pow(2) + (self.z - rhs.z).pow(2)) as u32
	}
}

impl From<IVec3> for ChunkPos {
	fn from(value: IVec3) -> Self {
		let IVec3 { x, y, z } = value;
		Self { x, y, z }
	}
}

impl From<ChunkPos> for IVec3 {
	fn from(value: ChunkPos) -> Self {
		let ChunkPos { x, y, z } = value;
		Self { x, y, z }
	}
}

impl Add<IVec3> for ChunkPos {
	type Output = Self;

	fn add(self, rhs: IVec3) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

use crate::game_world::chunk::CHUNK_LENGTH;
use bevy::{
	math::UVec3,
	prelude::{IVec3, Vec3},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos(pub IVec3);

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkPos(pub IVec3);

pub trait Vec3Utils {
	fn floor_to_ivec3(self) -> IVec3;
	fn to_block_pos(self) -> BlockPos;
	fn to_chunk_pos(self) -> ChunkPos;
}

impl Vec3Utils for Vec3 {
	fn floor_to_ivec3(self) -> IVec3 {
		IVec3::new(
			self.x.floor() as i32,
			self.y.floor() as i32,
			self.z.floor() as i32,
		)
	}

	fn to_block_pos(self) -> BlockPos {
		BlockPos(self.floor_to_ivec3())
	}

	fn to_chunk_pos(self) -> ChunkPos {
		ChunkPos((self / CHUNK_LENGTH as f32).floor_to_ivec3())
	}
}

pub trait UVec3Utils {
	fn to_vec3(self) -> Vec3;
}

impl UVec3Utils for UVec3 {
	fn to_vec3(self) -> Vec3 {
		Vec3::new(self.x as f32, self.y as f32, self.z as f32)
	}
}

pub trait IVec3Utils {
	fn to_vec3(self) -> Vec3;
}

impl IVec3Utils for IVec3 {
	fn to_vec3(self) -> Vec3 {
		Vec3::new(self.x as f32, self.y as f32, self.z as f32)
	}
}

impl BlockPos {
	pub fn new(x: i32, y: i32, z: i32) -> Self {
		Self(IVec3::new(x, y, z))
	}

	pub fn to_world_pos(self) -> Vec3 {
		Vec3::new(self.0.x as f32, self.0.y as f32, self.0.z as f32)
	}

	pub fn to_chunk_pos(self) -> ChunkPos {
		ChunkPos(IVec3::new(
			(self.0.x as f32 / CHUNK_LENGTH as f32).floor() as i32,
			(self.0.y as f32 / CHUNK_LENGTH as f32).floor() as i32,
			(self.0.z as f32 / CHUNK_LENGTH as f32).floor() as i32,
		))
	}
}

impl ChunkPos {
	pub fn new(x: i32, y: i32, z: i32) -> Self {
		Self(IVec3::new(x, y, z))
	}

	pub fn to_block_pos(self) -> BlockPos {
		BlockPos(IVec3::new(
			self.0.x * CHUNK_LENGTH as i32,
			self.0.y * CHUNK_LENGTH as i32,
			self.0.z * CHUNK_LENGTH as i32,
		))
	}

	pub fn to_world_pos(self) -> Vec3 {
		Vec3::new(
			self.0.x as f32 * CHUNK_LENGTH as f32,
			self.0.y as f32 * CHUNK_LENGTH as f32,
			self.0.z as f32 * CHUNK_LENGTH as f32,
		)
	}
}

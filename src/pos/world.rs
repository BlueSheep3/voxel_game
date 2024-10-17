use super::{BlockPos, ChunkPos};
use crate::game_world::chunk::CHUNK_LENGTH;
use bevy::math::{IVec3, Vec3};

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
		BlockPos::from(self.floor_to_ivec3())
	}

	fn to_chunk_pos(self) -> ChunkPos {
		ChunkPos::from((self / CHUNK_LENGTH as f32).floor_to_ivec3())
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

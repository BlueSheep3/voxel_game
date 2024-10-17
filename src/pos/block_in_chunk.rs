use bevy::math::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use std::num::TryFromIntError;

// FIXME you can create BlockInChunkPos instances that
// are not within the chunk using new or try_from

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockInChunkPos {
	pub x: u8,
	pub y: u8,
	pub z: u8,
}

impl BlockInChunkPos {
	pub fn new(x: u8, y: u8, z: u8) -> Self {
		Self { x, y, z }
	}
}

impl From<BlockInChunkPos> for IVec3 {
	fn from(value: BlockInChunkPos) -> Self {
		Self {
			x: value.x as i32,
			y: value.y as i32,
			z: value.z as i32,
		}
	}
}

impl From<BlockInChunkPos> for Vec3 {
	fn from(value: BlockInChunkPos) -> Self {
		Self {
			x: value.x as f32,
			y: value.y as f32,
			z: value.z as f32,
		}
	}
}

impl TryFrom<IVec3> for BlockInChunkPos {
	type Error = TryFromIntError;

	fn try_from(value: IVec3) -> Result<Self, Self::Error> {
		Ok(Self {
			x: value.x.try_into()?,
			y: value.y.try_into()?,
			z: value.z.try_into()?,
		})
	}
}

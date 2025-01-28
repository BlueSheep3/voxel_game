use crate::{
	block::{
		block_trait::{BlockTrait, BlockWithoutData},
		BlockData, BlockId,
	},
	cuboid::Cuboid,
};
use bevy::math::Vec3;
use std::fmt::Debug;

pub struct Stairs;

impl BlockTrait for Stairs {
	const BLOCK_ID: BlockId = BlockId(8);

	unsafe fn from_data(_data: BlockData) -> Self {
		Self
	}

	fn is_replacable(&self) -> bool {
		false
	}

	fn get_collision(&self) -> Vec<Cuboid> {
		vec![
			Cuboid {
				min: Vec3::ZERO,
				max: Vec3::new(1.0, 0.5, 1.0),
			},
			Cuboid {
				min: Vec3::new(0.5, 0.5, 0.0),
				max: Vec3::new(1.0, 1.0, 1.0),
			},
		]
	}
}

// SAFETY: Stairs is a Unit Type
unsafe impl BlockWithoutData for Stairs {}

impl Debug for Stairs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, stringify!(Stairs))
	}
}

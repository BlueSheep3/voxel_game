use crate::{
	block::{
		block_trait::{BlockTrait, BlockWithoutData},
		BlockData, BlockId,
	},
	cuboid::Cuboid,
};
use bevy::math::Vec3;
use std::fmt::Debug;

pub struct Dirt;

impl BlockTrait for Dirt {
	const BLOCK_ID: BlockId = BlockId(2);

	unsafe fn from_data(_data: BlockData) -> Self {
		Self
	}

	fn is_replacable(&self) -> bool {
		false
	}

	fn get_collision(&self) -> Vec<Cuboid> {
		vec![Cuboid {
			min: Vec3::ZERO,
			max: Vec3::ONE,
		}]
	}
}

// SAFETY: Dirt is a Unit Type
unsafe impl BlockWithoutData for Dirt {}

impl Debug for Dirt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, stringify!(Dirt))
	}
}

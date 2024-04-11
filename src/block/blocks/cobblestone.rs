use crate::{
	block::{
		block_trait::{BlockTrait, BlockWithoutData},
		BlockData, BlockId,
	},
	cuboid::Cuboid,
};
use bevy::math::Vec3;
use std::fmt::Debug;

pub struct Cobblestone;

impl BlockTrait for Cobblestone {
	const BLOCK_ID: BlockId = BlockId(4);

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

// SAFETY: Cobblestone is a Unit Type
unsafe impl BlockWithoutData for Cobblestone {}

impl Debug for Cobblestone {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, stringify!(Cobblestone))
	}
}

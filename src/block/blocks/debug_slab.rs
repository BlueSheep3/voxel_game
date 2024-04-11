use crate::{
	block::{
		block_trait::{BlockTrait, BlockWithoutData}, BlockData, BlockId
	},
	cuboid::Cuboid,
};
use bevy::math::Vec3;
use std::fmt::Debug;

pub struct DebugSlab;

impl BlockTrait for DebugSlab {
	const BLOCK_ID: BlockId = BlockId(100);

	unsafe fn from_data(_data: BlockData) -> Self {
		Self
	}

	fn is_replacable(&self) -> bool {
		false
	}

	fn get_collision(&self) -> Vec<Cuboid> {
		vec![Cuboid {
			min: Vec3::ZERO,
			max: Vec3::new(1.0, 0.5, 1.0),
		}]
	}
}

// SAFETY: DebugSlab is a Unit Type
unsafe impl BlockWithoutData for DebugSlab {}

impl Debug for DebugSlab {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, stringify!(DebugSlab))
	}
}

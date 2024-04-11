use crate::{
	block::{
		block_trait::{BlockTrait, BlockWithoutData},
		BlockData, BlockId,
	},
	cuboid::Cuboid,
};
use std::fmt::Debug;

pub struct Air;

impl BlockTrait for Air {
	const BLOCK_ID: BlockId = BlockId(0);

	unsafe fn from_data(_data: BlockData) -> Self {
		Self
	}

	fn is_replacable(&self) -> bool {
		true
	}

	fn get_collision(&self) -> Vec<Cuboid> {
		Vec::new()
	}
}

// SAFETY: Air is a Unit Type
unsafe impl BlockWithoutData for Air {}

impl Debug for Air {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, stringify!(Air))
	}
}

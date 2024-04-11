use super::{Block, BlockData, BlockId};
use crate::cuboid::Cuboid;
use std::fmt::Debug;

/// The Trait for a Block that may contain BlockData
pub trait BlockTrait: Debug + Sized {
	const BLOCK_ID: BlockId;

	/// Creates an instance of the Block from the given data
	///
	/// # Safety
	///
	/// This function is unsafe, because the data might be invalid for
	/// the given block, but calling this function assumes that it is valid.
	unsafe fn from_data(data: BlockData) -> Self;

	/// whether you can place a block inside of this one
	fn is_replacable(&self) -> bool;

	/// gets the Volume where you can collide with the block
	fn get_collision(&self) -> Vec<Cuboid>;

	/// gets the Volume where the block can be highighted by looking at it
	fn get_outline(&self) -> Vec<Cuboid> {
		// TODO in the future more blocks will have other outlines
		self.get_collision()
	}
}

/// The Trait for a Block that is guarenteed to never contain BlockData
///
/// # Safety
///
/// This trait is unsafe, because it assumes that the
/// implemented type is a unit type.
pub unsafe trait BlockWithoutData: BlockTrait {
	const BLOCK: Block = Block {
		id: Self::BLOCK_ID,
		// data: BlockData(0),
	};

	fn new() -> Self {
		// SAFETY: BlockWithoutData already assumes that this contains no data
		unsafe { Self::from_data(BlockData(0)) }
	}
}

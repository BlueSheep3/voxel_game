// FIXME block system is currently kind of weird because of the
// `BlockTrait` and the `match_block_id` macro.
// especially with the `BlockWithoutData` trait, since it is unsafe.

mod block_trait;
mod blocks;
mod macros;
pub mod prelude;

use self::{macros::match_block_id, prelude::*};
use crate::cuboid::Cuboid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Block {
	pub id: BlockId,
	// TODO block data
	// pub data: BlockData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockData(u8);

impl Block {
	/// whether you can place a block inside of this one
	pub fn is_replacable(self) -> bool {
		match_block_id!(self, (block: _Type) => block.is_replacable())
	}

	/// gets the Volume where you can collide with the block
	pub fn get_collision(&self) -> Vec<Cuboid> {
		match_block_id!(self, (block: _Type) => block.get_collision())
	}

	/// gets the Volume where the block can be highighted by looking at it
	pub fn get_outline(&self) -> Vec<Cuboid> {
		match_block_id!(self, (block: _Type) => block.get_outline())
	}
}

impl BlockId {
	pub fn from_debug_name(s: &str) -> Option<Self> {
		match s {
			"Air" => Some(Air::BLOCK_ID),
			"Stone" => Some(Stone::BLOCK_ID),
			"Dirt" => Some(Dirt::BLOCK_ID),
			"GrassBlock" => Some(GrassBlock::BLOCK_ID),
			"Cobblestone" => Some(Cobblestone::BLOCK_ID),
			"DebugBlock" => Some(DebugBlock::BLOCK_ID),
			"DebugSlab" => Some(DebugSlab::BLOCK_ID),
			"Log" => Some(Log::BLOCK_ID),
			"Planks" => Some(Planks::BLOCK_ID),
			"Leaves" => Some(Leaves::BLOCK_ID),
			_ => None,
		}
	}
}

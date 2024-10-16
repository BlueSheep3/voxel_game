/// Turns a [`BlockId`](super::BlockId) into the struct implementing
/// [`BlockTrait`](super::block_trait::BlockTrait), that has
/// that [`BlockTrait:BLOCK_ID`](super::block_trait::BlockTrait::BLOCK_ID),
/// and lets you use it in some expression.
///
/// # Examples
///
/// ```
/// # use crate::block::BlockId;
/// let id = BlockId::from_debug_name("Stone").unwrap();
/// let is_replacable = match_block_id!(id, (block: Type) => {
///     format!("{} | {}", Type::BlockId.0, block.is_replacable())
/// });
/// assert_eq!(is_replacable, "1 | false");
/// ```
macro_rules! match_block_id {
	($block:expr, ($block_var:ident: $block_type:ident) => $expr:expr) => {{
		use $crate::block::blocks::*;
		$crate::block::macros::big_match! {
			$block, $block_var, $block_type, $expr;
			air::Air,
			stone::Stone,
			dirt::Dirt,
			cobblestone::Cobblestone,
			grass_block::GrassBlock,
			log::Log,
			planks::Planks,
			leaves::Leaves,
			debug_block::DebugBlock,
			debug_slab::DebugSlab,
		}
	}};
}
pub(crate) use match_block_id;

macro_rules! big_match {
	($block:expr, $block_var:ident, $block_type:ident, $expr:expr; $($t:ty),* $(,)?) => {
		use $crate::block::block_trait::{BlockTrait, BlockWithoutData};
		match $block {
			$(
				Block { id: <$t>::BLOCK_ID } => {
					type $block_type = $t;
					let $block_var = <$t>::new();
					$expr
				}
			)*
			block => panic!("No implementation for block: {:?}", block),
		}
	}
}
pub(crate) use big_match;

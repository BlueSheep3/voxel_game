use crate::{block::prelude::*, pos::BlockInChunkPos};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

pub const CHUNK_LENGTH: usize = 32; // must be < 256
use CHUNK_LENGTH as LEN;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockArray([[[Block; LEN]; LEN]; LEN]);

impl BlockArray {
	pub const ALL_AIR: Self = Self([[[Air::BLOCK; LEN]; LEN]; LEN]);

	pub fn iter_xyz(&self) -> impl Iterator<Item = (BlockInChunkPos, Block)> + '_ {
		(0..LEN)
			.flat_map(|x| (0..LEN).flat_map(move |y| (0..LEN).map(move |z| [x, y, z])))
			.map(|[x, y, z]| {
				(
					BlockInChunkPos::new(x as u8, y as u8, z as u8),
					self.0[x][y][z],
				)
			})
	}
}

impl Index<BlockInChunkPos> for BlockArray {
	type Output = Block;

	fn index(&self, pos: BlockInChunkPos) -> &Self::Output {
		let BlockInChunkPos { x, y, z } = pos;
		&self.0[x as usize][y as usize][z as usize]
	}
}

impl IndexMut<BlockInChunkPos> for BlockArray {
	fn index_mut(&mut self, pos: BlockInChunkPos) -> &mut Self::Output {
		let BlockInChunkPos { x, y, z } = pos;
		&mut self.0[x as usize][y as usize][z as usize]
	}
}

use crate::{block::prelude::*, pos::BlockInChunkPos};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

// must be < 256 and a power of 2
pub const CHUNK_LENGTH: usize = 32;
use CHUNK_LENGTH as LEN;

// it would be nice to change this to a single array with length `LEN.pow(3)`,
// but then the `Serialize` and `Deserialize` derives wont work anymore.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockArray([[[Block; LEN]; LEN]; LEN]);

impl BlockArray {
	pub const ALL_AIR: Self = Self([[[Air::BLOCK; LEN]; LEN]; LEN]);

	pub fn iter_xyz(&self) -> impl Iterator<Item = (BlockInChunkPos, Block)> + '_ {
		(0..LEN.pow(3)).map(|index| {
			// SAFETY: `index < CHUNK_LENGTH.pow(3)`,
			// because its its an item of the iterator `0..LEN.pow(3)`
			let pos = unsafe { BlockInChunkPos::from_index_unchecked(index) };
			(pos, self[pos])
		})
	}
}

impl Index<BlockInChunkPos> for BlockArray {
	type Output = Block;

	fn index(&self, pos: BlockInChunkPos) -> &Self::Output {
		// since `index` must be `< CHUNK_LENGTH.pow(3)`, and
		// CHUNK_LENGTH must be `< 256`, this must fit inside an `isize`.
		let index = pos.index() as isize;

		// we treat the reference to `self` as the pointer to the first
		// element of a sequence of `Block`s.
		let self_ptr = std::ptr::from_ref(self);
		let self_ptr = self_ptr.cast::<Block>();

		// SAFETY: since `index` is an in bounds index of a `BlockArray`,
		// this will point to something inside `self`, which guarantees
		// that the offset did not go over the size of an `isize`.
		// finally, `self_ptr` is derived from `self`.
		let block_ptr = unsafe { self_ptr.offset(index) };

		// SAFETY: we have just proven that `block_ptr` points inside `self`,
		// meaning it cant be null or dangling.
		// and `offset` will never unalign the pointer, meaning its aligned here.
		// the lifetime of the return value of this function is bound by `self`.
		unsafe { &*block_ptr }
	}
}

impl IndexMut<BlockInChunkPos> for BlockArray {
	fn index_mut(&mut self, pos: BlockInChunkPos) -> &mut Self::Output {
		// since `index` must be `< CHUNK_LENGTH.pow(3)`, and
		// CHUNK_LENGTH must be `< 256`, this must fit inside an `isize`.
		let index = pos.index() as isize;

		// we treat the reference to `self` as the pointer to the first
		// element of a sequence of `Block`s.
		let self_ptr = std::ptr::from_mut(self);
		let self_ptr = self_ptr.cast::<Block>();

		// SAFETY: since `index` is an in bounds index of a `BlockArray`,
		// this will point to something inside `self`, which guarantees
		// that the offset did not go over the size of an `isize`.
		// finally, `self_ptr` is derived from `self`.
		let block_ptr = unsafe { self_ptr.offset(index) };

		// SAFETY: we have just proven that `block_ptr` points inside `self`,
		// meaning it cant be null or dangling.
		// and `offset` will never unalign the pointer, meaning its aligned here.
		// the lifetime of the return value of this function is bound by `self`.
		unsafe { &mut *block_ptr }
	}
}

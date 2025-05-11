use crate::{axis::Axis, game_world::chunk::CHUNK_LENGTH};
use bevy::math::{IVec3, Vec3};

// CHUNK_LENGTH is guaranteed to be a power of 2
const X_MASK: usize = Y_MASK << CHUNK_LENGTH.trailing_zeros();
const Y_MASK: usize = Z_MASK << CHUNK_LENGTH.trailing_zeros();
const Z_MASK: usize = CHUNK_LENGTH - 1;

mod internal {
	/// A position relative to some chunk.
	/// This is guaranteed to be in bounds of the chunk.
	#[derive(Default, Clone, Copy, PartialEq, Eq)]
	pub struct BlockInChunkPos {
		/// The index of the block at this position in a `BlockArray`.\
		/// You must guarantee `index < CHUNK_LENGTH.pow(3)`.
		index: usize,
	}

	impl BlockInChunkPos {
		/// # Safety
		///
		/// You must guarantee that `index < CHUNK_LENGTH.pow(3)`
		pub const unsafe fn from_index_unchecked(index: usize) -> Self {
			Self { index }
		}

		/// The index of the block at this position in a `BlockArray`.\
		/// Guaranteed to be `< CHUNK_LENGTH.pow(3)`.
		pub const fn index(self) -> usize {
			self.index
		}
	}
}

pub use self::internal::BlockInChunkPos;

impl BlockInChunkPos {
	pub fn try_from_index(index: usize) -> Option<Self> {
		(index < CHUNK_LENGTH.pow(3)).then(|| {
			// SAFETY: we just checked the only required safety guarante
			unsafe { Self::from_index_unchecked(index) }
		})
	}

	pub fn try_new(x: usize, y: usize, z: usize) -> Option<Self> {
		(x < CHUNK_LENGTH && y < CHUNK_LENGTH && z < CHUNK_LENGTH).then(|| {
			let index = x * CHUNK_LENGTH.pow(2) + y * CHUNK_LENGTH + z;
			// SAFETY: the largest possible value for index is if all
			// coordinates are 1 less than CHUNK_LENGTH, in which case
			// index will be barely less than CHUNK_LENGTH.pow(3)
			unsafe { Self::from_index_unchecked(index) }
		})
	}

	pub const fn x(self) -> usize {
		(self.index() / CHUNK_LENGTH.pow(2)) % CHUNK_LENGTH
	}

	pub const fn y(self) -> usize {
		(self.index() / CHUNK_LENGTH) % CHUNK_LENGTH
	}

	pub const fn z(self) -> usize {
		self.index() % CHUNK_LENGTH
	}

	pub const fn get(self, axis: Axis) -> usize {
		match axis {
			Axis::X => self.x(),
			Axis::Y => self.y(),
			Axis::Z => self.z(),
		}
	}

	pub fn with_x(self, x: usize) -> Option<Self> {
		(x < CHUNK_LENGTH).then(|| {
			let new_index = (self.index() & !X_MASK) + x * CHUNK_LENGTH.pow(2);
			// SAFETY: the inserted value for x is `< CHUNK_LENGTH` and
			// all previous values are as well, meaning the complete
			// index must be `< CHUNK_LENGTH.pow(3)`
			unsafe { Self::from_index_unchecked(new_index) }
		})
	}

	pub fn with_y(self, y: usize) -> Option<Self> {
		(y < CHUNK_LENGTH).then(|| {
			let new_index = (self.index() & !Y_MASK) + y * CHUNK_LENGTH;
			// SAFETY: the inserted value for y is `< CHUNK_LENGTH` and
			// all previous values are as well, meaning the complete
			// index must be `< CHUNK_LENGTH.pow(3)`
			unsafe { Self::from_index_unchecked(new_index) }
		})
	}

	pub fn with_z(self, z: usize) -> Option<Self> {
		(z < CHUNK_LENGTH).then(|| {
			let new_index = (self.index() & !Z_MASK) + z;
			// SAFETY: the inserted value for z is `< CHUNK_LENGTH` and
			// all previous values are as well, meaning the complete
			// index must be `< CHUNK_LENGTH.pow(3)`
			unsafe { Self::from_index_unchecked(new_index) }
		})
	}
}

impl From<BlockInChunkPos> for IVec3 {
	fn from(value: BlockInChunkPos) -> Self {
		Self {
			x: value.x() as i32,
			y: value.y() as i32,
			z: value.z() as i32,
		}
	}
}

impl From<BlockInChunkPos> for Vec3 {
	fn from(value: BlockInChunkPos) -> Self {
		Self {
			x: value.x() as f32,
			y: value.y() as f32,
			z: value.z() as f32,
		}
	}
}

impl TryFrom<IVec3> for BlockInChunkPos {
	type Error = IVec3;

	/// returns `Ok` if all coordinates are in the range `0..CHUNK_LENGTH`
	fn try_from(value: IVec3) -> Result<Self, Self::Error> {
		let IVec3 { x, y, z } = value;
		if (0..CHUNK_LENGTH as i32).contains(&x)
			&& (0..CHUNK_LENGTH as i32).contains(&y)
			&& (0..CHUNK_LENGTH as i32).contains(&z)
		{
			let [x, y, z] = [x as usize, y as usize, z as usize];
			let index = x * CHUNK_LENGTH.pow(2) + y * CHUNK_LENGTH + z;
			// SAFETY: the largest possible value for index is if all
			// coordinates are 1 less than CHUNK_LENGTH, in which case
			// index will be barely less than CHUNK_LENGTH.pow(3)
			Ok(unsafe { Self::from_index_unchecked(index) })
		} else {
			Err(value)
		}
	}
}

impl TryFrom<[usize; 3]> for BlockInChunkPos {
	type Error = [usize; 3];

	fn try_from(value @ [x, y, z]: [usize; 3]) -> Result<Self, Self::Error> {
		Self::try_new(x, y, z).ok_or(value)
	}
}

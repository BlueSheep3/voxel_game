use crate::{
	face::Face,
	pos::{BlockPos, ChunkPos},
};
use std::fmt::{self, Debug, Display};

impl Display for ChunkPos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
	}
}

impl Debug for ChunkPos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple(stringify!(ChunkPos))
			.field(&self.x)
			.field(&self.y)
			.field(&self.z)
			.finish()
	}
}

impl Display for BlockPos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
	}
}

impl Debug for BlockPos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple(stringify!(BlockPos))
			.field(&self.x)
			.field(&self.y)
			.field(&self.z)
			.finish()
	}
}

impl Debug for Face {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Self::Right => "Right",
			Self::Left => "Left",
			Self::Up => "Up",
			Self::Down => "Down",
			Self::Back => "Back",
			Self::Forward => "Forward",
		};
		write!(f, "Face::{}", s)
	}
}

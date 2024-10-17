use serde::{Deserialize, Serialize};

/// describes how much of the chunk has **already** been generated.<br>
/// for example: has the basic shape of the terrain been generated? or the trees?
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenerationStage {
	/// nothing of the chunk has been generated.
	/// it is literally empty.
	#[default]
	Nothing,
	/// the basic shape of it has been generated.
	/// for example: the dirt and stone blocks, including the cave shapes.
	Terrain,
	/// the trees have been placed in.
	Trees,
}

impl GenerationStage {
	pub const COMPLETE: Self = Self::Trees;
}

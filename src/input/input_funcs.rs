//! functions for the input resources to more easily use them

use super::{RotateInput, WalkInput};
use crate::entity::LookDirection;
use bevy::prelude::*;
use std::f32::consts::TAU;

impl WalkInput {
	/// treats the walk input as a vector in the xz plane and
	/// returns your walking vector when your camera is rotated by `rotation`<br>
	/// ensures that y = 0 and that the max length is 1
	pub fn with_look_dir(self, look_dir: LookDirection) -> Vec3 {
		// z coordinate is negated, because bevy forward is -z
		let vec3 = Vec3::new(self.vec.x, 0., -self.vec.y);
		let rotated = Quat::from_rotation_y(look_dir.yaw).mul_vec3(vec3);
		rotated.clamp_length_max(1.)
	}
}

impl RotateInput {
	/// rotates where you look and ensures that you aren't upside down
	pub fn rotate_look_dir(self, mut look_dir: LookDirection) -> LookDirection {
		look_dir.pitch += self.pitch;
		look_dir.yaw += self.yaw;

		look_dir.pitch = look_dir.pitch.clamp(-TAU / 4.0 + 0.001, TAU / 4.0 - 0.001);
		look_dir.yaw = (look_dir.yaw + TAU) % TAU;

		look_dir
	}
}

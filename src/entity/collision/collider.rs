use crate::cuboid::Cuboid;
use bevy::prelude::*;

/// collision in the shape of a box, with the same width and length<br>
/// (0,0,0) is at the bottom center of the box<br>
/// will not be rotated or scaled with the transform
#[derive(Component, Debug, Clone, Copy)]
pub struct BoxCollider {
	width: f32,
	height: f32,
}

impl BoxCollider {
	pub fn new(width: f32, height: f32) -> Self {
		Self { width, height }
	}

	pub fn into_cuboid(self) -> Cuboid {
		let w = self.width / 2.0;
		let h = self.height;
		Cuboid::from_corners(Vec3::new(-w, 0.0, -w), Vec3::new(w, h, w))
	}
}

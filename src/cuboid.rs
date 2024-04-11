use bevy::prelude::*;
use std::ops::{Add, Sub};

/// a rectangular cuboid that is not rotated<br>
/// behaves like [`bevy::prelude::Rect`], but in 3D
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Cuboid {
	pub min: Vec3,
	pub max: Vec3,
}

// pretty much all of this is copied from bevy's Rect
#[allow(dead_code)]
impl Cuboid {
	pub const ZERO: Self = Self {
		min: Vec3::ZERO,
		max: Vec3::ZERO,
	};

	pub fn from_corners(p0: Vec3, p1: Vec3) -> Self {
		Self {
			min: p0.min(p1),
			max: p0.max(p1),
		}
	}

	pub fn new(x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32) -> Self {
		Self::from_corners(Vec3::new(x0, y0, z0), Vec3::new(x1, y1, z1))
	}

	pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
		assert!(size.cmpge(Vec3::ZERO).all(), "Cuboid size must be positive");
		let half_size = size / 2.;
		Self::from_center_half_size(center, half_size)
	}

	pub fn from_center_half_size(center: Vec3, half_size: Vec3) -> Self {
		assert!(
			half_size.cmpge(Vec3::ZERO).all(),
			"Cuboid half_size must be positive"
		);
		Self {
			min: center - half_size,
			max: center + half_size,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.min.cmpge(self.max).any()
	}

	pub fn width(&self) -> f32 {
		self.max.x - self.min.x
	}

	pub fn height(&self) -> f32 {
		self.max.y - self.min.y
	}

	pub fn length(&self) -> f32 {
		self.max.z - self.min.z
	}

	pub fn size(&self) -> Vec3 {
		self.max - self.min
	}

	pub fn half_size(&self) -> Vec3 {
		self.size() * 0.5
	}

	pub fn center(&self) -> Vec3 {
		(self.min + self.max) * 0.5
	}

	pub fn contains(&self, point: Vec3) -> bool {
		(point.cmpge(self.min) & point.cmple(self.max)).all()
	}

	pub fn intersect(&self, other: Self) -> Self {
		let mut r = Self {
			min: self.min.max(other.min),
			max: self.max.min(other.max),
		};
		// Collapse min over max to enforce invariants and ensure e.g. width() or
		// height() never return a negative value.
		r.min = r.min.min(r.max);
		r
	}
}

impl Add<Vec3> for Cuboid {
	type Output = Self;

	/// translates the Cuboid by `rhs`
	fn add(self, rhs: Vec3) -> Self::Output {
		Self {
			min: self.min + rhs,
			max: self.max + rhs,
		}
	}
}

impl Sub<Vec3> for Cuboid {
	type Output = Self;

	/// translates the Cuboid by `-rhs`
	fn sub(self, rhs: Vec3) -> Self::Output {
		Self {
			min: self.min - rhs,
			max: self.max - rhs,
		}
	}
}

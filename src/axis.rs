#![allow(dead_code)]

use crate::face::Face;
use bitmask::bitmask;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

bitmask! {
	pub mask AxisMask: u8 where flags Axis {
		X = 1,
		Y = 2,
		Z = 4,
	}
}

impl Axis {
	pub fn face_pos(self) -> Face {
		match self {
			Self::X => Face::Right,
			Self::Y => Face::Up,
			Self::Z => Face::Back,
		}
	}

	pub fn face_neg(self) -> Face {
		match self {
			Self::X => Face::Left,
			Self::Y => Face::Down,
			Self::Z => Face::Forward,
		}
	}

	pub fn all() -> AxisIter {
		AxisIter { index: 0 }
	}
}

pub struct AxisIter {
	index: u8,
}

impl Iterator for AxisIter {
	type Item = Axis;

	fn next(&mut self) -> Option<Self::Item> {
		let axis = index_to_axis(self.index);
		// all values above 2 will just be None
		self.index = self.index.saturating_add(1);
		axis
	}
}

/// maps each axis to some value in an efficient manner
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AxisMap<T>([T; 3]);

const fn axis_to_index(axis: Axis) -> usize {
	match axis {
		Axis::X => 0,
		Axis::Y => 1,
		Axis::Z => 2,
	}
}

const fn index_to_axis(index: u8) -> Option<Axis> {
	match index {
		0 => Some(Axis::X),
		1 => Some(Axis::Y),
		2 => Some(Axis::Z),
		_ => None,
	}
}

impl<T> AxisMap<T> {
	pub fn get(&self, axis: Axis) -> &T {
		&self.0[axis_to_index(axis)]
	}

	pub fn get_mut(&mut self, axis: Axis) -> &mut T {
		&mut self.0[axis_to_index(axis)]
	}

	pub fn iter(&self) -> impl Iterator<Item = &T> {
		self.0.iter()
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
		self.0.iter_mut()
	}

	pub fn into_iter(self) -> impl Iterator<Item = T> {
		self.0.into_iter()
	}

	pub fn iter_axis(&self) -> impl Iterator<Item = (&T, Axis)> {
		use Axis as A;
		self.0.iter().zip([A::X, A::Y, A::Z])
	}

	pub fn map<U>(self, f: impl FnMut(T) -> U) -> AxisMap<U> {
		AxisMap(self.0.map(f))
	}

	/// creates a new AxisMap by mapping over every axis
	pub fn from_map(f: impl FnMut(Axis) -> T) -> Self {
		use Axis as A;
		[A::X, A::Y, A::Z].map(f).into()
	}
}

impl<T> AxisMap<Option<T>> {
	pub fn all_some(self) -> Option<AxisMap<T>> {
		self.into_iter()
			.collect::<Option<Vec<_>>>()
			.map(|vec| AxisMap::try_from(vec).unwrap_or_else(|_| unreachable!()))
	}
}

impl<T> From<[T; 3]> for AxisMap<T> {
	fn from(value: [T; 3]) -> Self {
		Self(value)
	}
}

impl<T> TryFrom<Vec<T>> for AxisMap<T> {
	type Error = Vec<T>;

	fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
		value.try_into().map(Self)
	}
}

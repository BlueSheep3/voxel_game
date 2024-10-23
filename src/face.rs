#![allow(dead_code)]

use crate::axis::Axis;
use bevy::math::IVec3;
use bitmask::bitmask;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

bitmask! {
	pub mask FaceMask: u8 where flags Face {
		Right = 1,
		Left = 2,
		Up = 4,
		Down = 8,
		Back = 16,
		Forward = 32,
	}
}

impl Face {
	pub fn normal(self) -> IVec3 {
		match self {
			Self::Right => IVec3::X,
			Self::Left => -IVec3::X,
			Self::Up => IVec3::Y,
			Self::Down => -IVec3::Y,
			Self::Back => IVec3::Z,
			Self::Forward => -IVec3::Z,
		}
	}

	pub fn opposite(self) -> Self {
		match self {
			Self::Right => Self::Left,
			Self::Left => Self::Right,
			Self::Up => Self::Down,
			Self::Down => Self::Up,
			Self::Back => Self::Forward,
			Self::Forward => Self::Back,
		}
	}

	pub fn axis(self) -> Axis {
		match self {
			Self::Right | Self::Left => Axis::X,
			Self::Up | Self::Down => Axis::Y,
			Self::Back | Self::Forward => Axis::Z,
		}
	}

	pub fn all() -> FaceIter {
		FaceIter { index: 0 }
	}
}

pub struct FaceIter {
	index: u8,
}

impl Iterator for FaceIter {
	type Item = Face;

	fn next(&mut self) -> Option<Self::Item> {
		let face = index_to_face(self.index);
		// all values above 5 will just be None
		self.index = self.index.saturating_add(1);
		face
	}
}

/// maps each face to some value in an efficient manner
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaceMap<T>([T; 6]);

const fn face_to_index(face: Face) -> usize {
	match face {
		Face::Right => 0,
		Face::Left => 1,
		Face::Up => 2,
		Face::Down => 3,
		Face::Back => 4,
		Face::Forward => 5,
	}
}

const fn index_to_face(index: u8) -> Option<Face> {
	match index {
		0 => Some(Face::Right),
		1 => Some(Face::Left),
		2 => Some(Face::Up),
		3 => Some(Face::Down),
		4 => Some(Face::Back),
		5 => Some(Face::Forward),
		_ => None,
	}
}

impl<T> FaceMap<T> {
	pub fn get(&self, face: Face) -> &T {
		&self.0[face_to_index(face)]
	}

	pub fn get_mut(&mut self, face: Face) -> &mut T {
		&mut self.0[face_to_index(face)]
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

	pub fn iter_face(&self) -> impl Iterator<Item = (&T, Face)> {
		use Face as F;
		self.0
			.iter()
			.zip([F::Right, F::Left, F::Up, F::Down, F::Back, F::Forward])
	}

	pub fn map<U>(self, f: impl FnMut(T) -> U) -> FaceMap<U> {
		FaceMap(self.0.map(f))
	}

	/// creates a new FaceMap by mapping over every face
	pub fn from_map(f: impl FnMut(Face) -> T) -> Self {
		use Face as F;
		[F::Right, F::Left, F::Up, F::Down, F::Back, F::Forward]
			.map(f)
			.into()
	}
}

impl<T> FaceMap<Option<T>> {
	pub fn all_some(self) -> Option<FaceMap<T>> {
		self.into_iter()
			.collect::<Option<Vec<_>>>()
			.map(|vec| FaceMap::try_from(vec).unwrap_or_else(|_| unreachable!()))
	}
}

impl<T> From<[T; 6]> for FaceMap<T> {
	fn from(value: [T; 6]) -> Self {
		Self(value)
	}
}

impl<T> TryFrom<Vec<T>> for FaceMap<T> {
	type Error = Vec<T>;

	fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
		value.try_into().map(Self)
	}
}

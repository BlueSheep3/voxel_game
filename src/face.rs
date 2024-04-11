use bevy::math::IVec3;
use bitmask::bitmask;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

bitmask! {
	pub mask FacesMask: u8 where flags Face {
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

	pub fn all() -> Vec<Self> {
		vec![
			Self::Right,
			Self::Left,
			Self::Up,
			Self::Down,
			Self::Back,
			Self::Forward,
		]
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

#[allow(dead_code)]
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

#![allow(dead_code)]
// Since Rust 1.84.0 (https://github.com/rust-lang/rust/pull/132577)
// the `unexpected_cfgs` warning is also shown when used inside a macro.
// Until the `bitmask` crate fixes that its macro causes this warning,
// it will be ignored here.
#![allow(unexpected_cfgs)]

use crate::axis::Axis;
use bevy::math::IVec3;
use bitmask::bitmask;
use serde::{Deserialize, Serialize};
use std::{
	fmt::Debug,
	ops::{Index, IndexMut},
};

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
	pub const fn normal(self) -> IVec3 {
		match self {
			Self::Right => IVec3::X,
			Self::Left => IVec3::NEG_X,
			Self::Up => IVec3::Y,
			Self::Down => IVec3::NEG_Y,
			Self::Back => IVec3::Z,
			Self::Forward => IVec3::NEG_Z,
		}
	}

	pub const fn opposite(self) -> Self {
		match self {
			Self::Right => Self::Left,
			Self::Left => Self::Right,
			Self::Up => Self::Down,
			Self::Down => Self::Up,
			Self::Back => Self::Forward,
			Self::Forward => Self::Back,
		}
	}

	/// is this face pointing in the positive direction?
	pub const fn is_pos(self) -> bool {
		match self {
			Self::Right | Self::Up | Self::Back => true,
			Self::Left | Self::Down | Self::Forward => false,
		}
	}

	pub const fn axis(self) -> Axis {
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
	pub fn iter(&self) -> impl Iterator<Item = &T> {
		self.0.iter()
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
		self.0.iter_mut()
	}

	pub fn into_iter(self) -> impl Iterator<Item = T> {
		self.0.into_iter()
	}

	pub fn iter_face(&self) -> impl Iterator<Item = (Face, &T)> {
		use Face as F;
		[F::Right, F::Left, F::Up, F::Down, F::Back, F::Forward]
			.into_iter()
			.zip(&self.0)
	}

	pub fn map<U>(self, f: impl FnMut(T) -> U) -> FaceMap<U> {
		FaceMap(self.0.map(f))
	}

	/// maps over all elements along with which face maps to them
	pub fn face_map<U>(self, mut func: impl FnMut(Face, T) -> U) -> FaceMap<U> {
		let [r, l, u, d, b, f] = self.0;
		let new_map = [
			func(Face::Right, r),
			func(Face::Left, l),
			func(Face::Up, u),
			func(Face::Down, d),
			func(Face::Back, b),
			func(Face::Forward, f),
		];
		FaceMap(new_map)
	}

	/// creates a new FaceMap by mapping over every face
	pub fn from_map(f: impl FnMut(Face) -> T) -> Self {
		use Face as F;
		Self([F::Right, F::Left, F::Up, F::Down, F::Back, F::Forward].map(f))
	}
}

impl<T> Index<Face> for FaceMap<T> {
	type Output = T;

	fn index(&self, index: Face) -> &Self::Output {
		&self.0[face_to_index(index)]
	}
}

impl<T> IndexMut<Face> for FaceMap<T> {
	fn index_mut(&mut self, index: Face) -> &mut Self::Output {
		&mut self.0[face_to_index(index)]
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

use crate::{
	cuboid::Cuboid,
	face::Face,
	game_world::GameWorld,
	match_min,
	pos::{BlockPos, Vec3Utils},
};
use bevy::prelude::*;

/// a Ray with a finite length
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiniteRay {
	pub start: Vec3,
	pub dir: Vec3,
	pub length: f32,
}

impl FiniteRay {
	pub fn new(start: Vec3, dir: Vec3, length: f32) -> Self {
		debug_assert!(dir.is_normalized());
		debug_assert!(length >= 0.0);
		Self { start, dir, length }
	}
}

// NOTE the current impl does not work with blocks that are larger than 1x1x1
/// gets all block positions, that intersect the given finite ray
pub fn get_all_block_pos_in_ray(ray: FiniteRay) -> Vec<BlockPos> {
	// this uses the implementation outlined in:
	// http://www.cse.yorku.ca/~amana/research/grid.pdf

	let FiniteRay { start, dir, length } = ray;

	let dir_a = dir.abs();
	let mut current = start.to_block_pos();
	let delta = 1.0 / dir_a;
	let step = dir.signum().floor_to_ivec3();

	// very messy because of weird behaviour with negatives
	let mut max = Vec3 {
		x: if step.x < 0 {
			start.x.rem_euclid(1.0) / dir_a.x
		} else {
			(1.0 - start.x.rem_euclid(1.0)) / dir_a.x
		},
		y: if step.y < 0 {
			start.y.rem_euclid(1.0) / dir_a.y
		} else {
			(1.0 - start.y.rem_euclid(1.0)) / dir_a.y
		},
		z: if step.z < 0 {
			start.z.rem_euclid(1.0) / dir_a.z
		} else {
			(1.0 - start.z.rem_euclid(1.0)) / dir_a.z
		},
	};

	let mut vec = Vec::new();
	vec.push(current);

	let mut failsafe = 0;

	loop {
		failsafe += 1;
		if failsafe > 200 {
			warn!("get_all_block_pos_in_ray() has activated its failsafe");
			break;
		}

		match_min! {
			max.x => {
				current.0.x += step.x;
				max.x += delta.x;
			}
			max.y => {
				current.0.y += step.y;
				max.y += delta.y;
			}
			max.z => {
				current.0.z += step.z;
				max.z += delta.z;
			}
		}

		vec.push(current);

		let t = max.min_element();
		if t > length {
			break;
		}
	}

	vec
}

/// gets the first world position where the given ray intersects a given Cuboid
fn get_first_ray_intersection(ray: FiniteRay, cuboid: Cuboid) -> Option<(Vec3, Face)> {
	let FiniteRay { start, dir, length } = ray;

	// expand the cuboid slightly to ensure that it contains its boundaries
	let expanded = Cuboid {
		min: cuboid.min - 0.01,
		max: cuboid.max + 0.01,
	};

	// t_min and t_max are refering to the bounds of the Cuboid
	// t_min is not always smaller than t_max
	let t_min = (cuboid.min - start) / dir;
	let t_max = (cuboid.max - start) / dir;

	let min_arr: [f32; 3] = t_min.into();
	let max_arr: [f32; 3] = t_max.into();

	let t_values = [min_arr, max_arr].concat();
	let with_faces = t_values.into_iter().zip([
		Face::Left,
		Face::Down,
		Face::Forward,
		Face::Right,
		Face::Up,
		Face::Back,
	]);

	let mut vec = with_faces.collect::<Vec<_>>();
	vec.sort_by(|(a, _), (b, _)| a.total_cmp(b));

	for &(t, face) in &vec[0..3] {
		if t > length {
			return None;
		}
		let p = start + dir * t;
		if expanded.contains(p) {
			return Some((p, face));
		}
	}
	None
}

/// the information you get when sending a Ray
#[derive(Debug, Clone, Copy)]
pub struct RayHitInfo {
	pub pos: Vec3,
	pub block_pos: BlockPos,
	pub face: Face,
}

pub fn send_out_ray(ray: FiniteRay, game_world: &GameWorld) -> Option<RayHitInfo> {
	let positions = get_all_block_pos_in_ray(ray);

	for block_pos in positions {
		let Some(block) = game_world.get_block_at(block_pos) else {
			continue;
		};
		for block_outline in block.get_outline() {
			let block_outline = block_outline + block_pos.to_world_pos();
			if let Some((pos, face)) = get_first_ray_intersection(ray, block_outline) {
				return Some(RayHitInfo {
					pos,
					block_pos,
					face,
				});
			}
		}
	}

	None
}

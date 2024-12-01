use super::collider::BoxCollider;
use crate::{
	cuboid::Cuboid,
	entity::movement::{MovementSet, OnGround, Velocity},
	face::Face,
	game_world::GameWorld,
	pos::{BlockPos, IVec3Utils, Vec3Utils},
	GlobalState,
};
use bevy::prelude::*;
use float_next_after::NextAfter;

pub struct MoveAndSlidePlugin;

impl Plugin for MoveAndSlidePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			move_and_slide
				.in_set(MovementSet::Translate)
				.run_if(in_state(GlobalState::InWorld)),
		);
	}
}

// FIXME can still sometimes walk through walls

/// moves the hitbox using its velocity and
/// slides along any blocks in the way
fn move_and_slide(
	mut query: Query<(
		&mut Transform,
		&mut Velocity,
		&BoxCollider,
		Option<&mut OnGround>,
	)>,
	time: Res<Time>,
	game_world: Res<GameWorld>,
) {
	let dt = time.delta_secs();
	for (mut trans, mut vel, col, mut on_ground) in &mut query {
		let local_hitbox = col.into_cuboid();

		if let Some(ref mut on_ground) = on_ground {
			on_ground.0 = false;
		}

		// v += a/2;  s += v;  v += a/2;
		// this is the velocity after the first step
		// since `vel.vel` is the velocity after the last step
		let mut v = vel.vel - vel.delta() / 2.;

		let mut t = dt;
		while t > 0. {
			let hitbox = local_hitbox + trans.translation;

			// get the relevant cuboids
			let positions = get_all_block_pos_for_cuboid_cast(hitbox, v * t);
			let block_collisions = get_block_collisions(positions, &game_world);

			let block_hit = earliest_block_hit(&block_collisions, hitbox, v, dt);
			let Some((hit_t, face)) = block_hit else {
				// since there are no more collisions in a straight line,
				// we can just move linearly for the rest of t
				trans.translation += v * t;
				break;
			};

			if let Some(ref mut on_ground) = on_ground {
				if face == Face::Up {
					on_ground.0 = true;
				}
			}

			// move by hit_t
			t -= hit_t;
			trans.translation += v * hit_t;

			// FIXME moving slightly makes this not 100% precise :(
			trans.translation = move_slighty_in_direction(trans.translation, face);

			// set velocity to 0 in direction of face
			let vel_mask = 1. - face.normal().to_vec3().abs();
			v *= vel_mask;
			vel.vel *= vel_mask;
		}
	}
}

/// Moves a position in the direction of the normal of a face.
/// Note that this is not necessarily the smallest possible value,
/// but it will never get rounded away.
fn move_slighty_in_direction(pos: Vec3, face: Face) -> Vec3 {
	let normal = face.normal().to_vec3();
	let after_pos = Vec3::new(
		if normal.x.abs() < f32::EPSILON {
			pos.x
		} else {
			normal.x.signum() * f32::INFINITY
		},
		if normal.y.abs() < f32::EPSILON {
			pos.y
		} else {
			normal.y.signum() * f32::INFINITY
		},
		if normal.z.abs() < f32::EPSILON {
			pos.z
		} else {
			normal.z.signum() * f32::INFINITY
		},
	);
	Vec3::new(
		pos.x.next_after(after_pos.x),
		pos.y.next_after(after_pos.y),
		pos.z.next_after(after_pos.z),
	) + normal * f32::EPSILON
}

/// Figures out how far a hitbox can be moved with a given velocity until it
/// hits any block in any direction. Will return `None` if no block is
/// hit, and `Some((t, f))` if it hits, where `t` is how long the hitbox
/// has to move to hit a block, and `f` is the face of the block.
fn earliest_block_hit(
	block_collisions: &[Cuboid],
	hitbox: Cuboid,
	vel: Vec3,
	max_t: f32,
) -> Option<(f32, Face)> {
	block_collisions
		.iter()
		.flat_map(|block_collision| {
			Face::all()
				.flat_map(|face| {
					find_plane_intersect(hitbox, *block_collision, face, vel)
						.map(|t| (t, face))
						.filter(|&(t, _)| t <= max_t)
				})
				.min_by(|(a, _), (b, _)| a.total_cmp(b))
		})
		.min_by(|(a, _), (b, _)| a.total_cmp(b))
}

/// similar to [`find_min_movement`], except only checks
/// for a single given block and a single given direction
fn find_plane_intersect(
	hitbox: Cuboid,
	block_col: Cuboid,
	block_face: Face,
	vel: Vec3,
) -> Option<f32> {
	let hitbox_face = block_face.opposite();

	macro_rules! faces_intersect {
		($($face:ident => $axis:ident, $hitbox_side:ident, $block_side:ident,
		$other_plane:ident, $v_cmp:tt, $b_cmp:tt);* $(;)?) => {
			match hitbox_face { $(
				Face::$face => {
					if vel.$axis $v_cmp 0. {
						return None;
					}

					let hitbox_max = hitbox.$hitbox_side.$axis;
					let block_min = block_col.$block_side.$axis;
					if hitbox_max $b_cmp block_min {
						return None;
					}

					// time to hit the block
					let t = (block_min - hitbox_max) / vel.$axis;
					if t < 0. {
						return None;
					}
					let delta = vel.$other_plane() * t;

					// check whether the planes intersect
					let corner_min = hitbox.min.$other_plane() + delta;
					let corner_max = hitbox.max.$other_plane() + delta;
					let hitbox_plane = Rect::from_corners(corner_min, corner_max);

					let corner_min = block_col.min.$other_plane();
					let corner_max = block_col.max.$other_plane();
					let block_plane = Rect::from_corners(corner_min, corner_max);

					rects_intersect(hitbox_plane, block_plane).then_some(t)
				}
			)* }
		};
	}

	faces_intersect!(
		Right   => x, max, min, yz, <=, >;
		Left    => x, min, max, yz, >=, <;
		Up      => y, max, min, xz, <=, >;
		Down    => y, min, max, xz, >=, <;
		Back    => z, max, min, xy, <=, >;
		Forward => z, min, max, xy, >=, <;
	)
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
	// if allow_thin {
	a.min.cmple(b.max).all() && b.min.cmple(a.max).all()
	// } else {
	// a.min.cmplt(b.max).all() && b.min.cmplt(a.max).all()
	// }
}

// NOTE the current impl does not work with blocks that are larger than 1x1x1
/// gets a Vec of all block collision Cuboids from a Vec of block positions
fn get_block_collisions(positions: Vec<BlockPos>, game_world: &GameWorld) -> Vec<Cuboid> {
	positions
		.into_iter()
		.flat_map(|pos| {
			game_world
				.get_block_at(pos)
				.into_iter()
				.flat_map(|block| block.get_collision())
				.map(move |col| col + pos.to_world_pos())
		})
		.collect()
}

// NOTE the current impl does not work with blocks that are larger than 1x1x1
/// gets all positions of blocks, that are relevant
/// to the given hitbox moving by some delta
fn get_all_block_pos_for_cuboid_cast(cuboid: Cuboid, delta: Vec3) -> Vec<BlockPos> {
	// NOTE this implementation returns more blocks than needed
	let Cuboid { min, max } = cuboid;
	// expand bounds in direction of delta
	let min = min.min(min + delta);
	let max = max.max(max + delta);
	// expand bounds slightly to avoid weird things at boundaries
	let min = (min - Vec3::ONE * 0.2).floor_to_ivec3();
	let max = (max + Vec3::ONE * 0.2).floor_to_ivec3();
	// get block positions
	(min.x..=max.x)
		.flat_map(move |x| (min.y..=max.y).map(move |y| [x, y]))
		.flat_map(|[x, y]| (min.z..=max.z).map(move |z| [x, y, z]))
		.map(|[x, y, z]| BlockPos::new(x, y, z))
		.collect()
}

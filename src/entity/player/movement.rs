use super::{cam::PlayerCamMode, Player};
use crate::{
	entity::{
		movement::{MovementSet, OnGround, Velocity},
		LookDirection,
	},
	input::{InputSet, JumpInput, WalkInput},
	GlobalState,
};
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<MovementValues>()
			.register_type::<MovementValues>()
			.add_systems(
				Update,
				(walk, jump)
					.run_if(not(in_state(PlayerCamMode::FreeCam)))
					.run_if(in_state(GlobalState::InWorld))
					.in_set(MovementSet::Accel)
					.in_set(InputSet::Use),
			);
	}
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct MovementValues {
	move_ground_accel: f32,
	move_friction: f32,
	move_air_accel: f32,
	move_drag: f32,
	jump_strength: f32,
}

impl Default for MovementValues {
	fn default() -> Self {
		Self {
			move_ground_accel: 12.0,
			move_friction: 0.3,
			move_air_accel: 8.0,
			move_drag: 0.2,
			jump_strength: 10.0,
		}
	}
}

// FIXME currently VERY slippery
// const MOVE_GROUND_ACCEL: f32 = 12.0;
// const MOVE_FRICTION: f32 = 0.3;
// const MOVE_AIR_ACCEL: f32 = 8.0;
// const MOVE_DRAG: f32 = 0.2;
// const JUMP_STRENGTH: f32 = 10.0;

fn walk(
	walk_input: Res<WalkInput>,
	mut player: Query<(&mut Velocity, &LookDirection, &OnGround), With<Player>>,
	time: Res<Time>,
	values: Res<MovementValues>,
) {
	let dt = time.delta_seconds();
	let (mut player_vel, look_dir, on_ground) = player.single_mut();
	let vec = walk_input.with_look_dir(*look_dir);

	let prev_y = player_vel.vel.y;
	if on_ground.0 {
		player_vel.vel += vec * values.move_ground_accel * dt;
		player_vel.vel *= 1.0 - values.move_friction * dt;
	} else {
		player_vel.vel += vec * values.move_air_accel * dt;
		player_vel.vel *= 1.0 - values.move_drag * dt;
	}
	player_vel.vel.y = prev_y;
}

fn jump(
	input: Res<JumpInput>,
	mut player: Query<(&mut Velocity, &OnGround), With<Player>>,
	values: Res<MovementValues>,
) {
	if input.started {
		let (mut player_vel, on_ground) = player.single_mut();
		if on_ground.0 {
			player_vel.vel.y = values.jump_strength;
		}
	}
}

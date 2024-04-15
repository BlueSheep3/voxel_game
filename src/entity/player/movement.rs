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
				(
					(jump, walk).run_if(not(in_state(PlayerCamMode::FreeCam))),
					friction,
				)
					// chain ensures that walk and friction are always done in the same order
					.chain()
					.run_if(in_state(GlobalState::InWorld))
					.in_set(MovementSet::Accel)
					.in_set(InputSet::Use),
			);
	}
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct MovementValues {
	/// the acceleration on the ground in m/s²
	move_ground_accel: f32,
	/// 1 / the amount of seconds it takes to half the velocity due to friction on the ground
	move_friction: f32,
	/// the acceleration in the air in m/s²
	move_air_accel: f32,
	/// 1 / the amount of seconds it takes to half the velocity due to friction in the air
	move_drag: f32,
	/// the upward velocity when jumping
	jump_strength: f32,
}

impl Default for MovementValues {
	fn default() -> Self {
		Self {
			move_ground_accel: 80.0,
			move_friction: 16.0,
			move_air_accel: 40.0,
			move_drag: 8.0,
			jump_strength: 7.0,
		}
	}
}

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
	} else {
		player_vel.vel += vec * values.move_air_accel * dt;
	}
	player_vel.vel.y = prev_y;
}

fn friction(
	mut player: Query<(&mut Velocity, &OnGround), With<Player>>,
	time: Res<Time>,
	values: Res<MovementValues>,
) {
	let dt = time.delta_seconds();
	let (mut player_vel, on_ground) = player.single_mut();

	let prev_y = player_vel.vel.y;
	if on_ground.0 {
		player_vel.vel *= (-values.move_friction * dt).exp2();
	} else {
		player_vel.vel *= (-values.move_drag * dt).exp2();
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

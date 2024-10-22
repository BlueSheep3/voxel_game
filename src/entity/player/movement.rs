use super::{cam::PlayerCamMode, Player};
use crate::{
	entity::{
		movement::{Gravity, MovementSet, OnGround, Velocity},
		LookDirection,
	},
	input::{CrouchInput, InputSet, JumpInput, ScrollInput, WalkInput},
	GlobalState,
};
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<MovementValues>()
			.register_type::<MovementValues>()
			.add_sub_state::<IsFlying>()
			.add_systems(
				Update,
				(
					(
						jump.run_if(in_state(IsFlying(false))),
						fly_vertical.run_if(in_state(IsFlying(true))),
						walk,
						toggle_flying,
						change_speed_mult,
					)
						.run_if(not(in_state(PlayerCamMode::FreeCam))),
					friction,
					update_gravity,
				)
					// chain ensures that walk and friction are always done in the same order
					.chain()
					.run_if(in_state(GlobalState::InWorld))
					.in_set(MovementSet::Accel)
					.in_set(InputSet::Use),
			);
	}
}

pub const GRAVITY: f32 = -20.0;

#[derive(SubStates, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[source(GlobalState = GlobalState::InWorld)]
struct IsFlying(bool);

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
	/// the speed at which you move vertically when flying
	vertical_fly_speed: f32,
	/// the amount of downwards velocity added per frame
	gravity: f32,
	/// multiplies all movement speed (except gravity)
	mult: f32,
}

impl Default for MovementValues {
	fn default() -> Self {
		Self {
			move_ground_accel: 80.0,
			move_friction: 16.0,
			move_air_accel: 40.0,
			move_drag: 8.0,
			jump_strength: 7.0,
			vertical_fly_speed: 200.0,
			gravity: GRAVITY,
			mult: 1.0,
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
		player_vel.vel += vec * values.move_ground_accel * values.mult * dt;
	} else {
		player_vel.vel += vec * values.move_air_accel * values.mult * dt;
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
			player_vel.vel.y = values.jump_strength * values.mult.sqrt();
		}
	}
}

fn toggle_flying(
	flying: Res<State<IsFlying>>,
	mut next_flying: ResMut<NextState<IsFlying>>,
	input: Res<ButtonInput<KeyCode>>,
	mut player_grav: Query<&mut Gravity, With<Player>>,
	values: Res<MovementValues>,
) {
	if input.just_pressed(KeyCode::KeyC) {
		let new = !flying.0;
		next_flying.set(IsFlying(new));

		let mut gravity = player_grav.single_mut();
		*gravity = if new {
			Gravity::ZERO
		} else {
			Gravity::vertical(values.gravity)
		};
	}
}

fn fly_vertical(
	jump_input: Res<JumpInput>,
	crouch_input: Res<CrouchInput>,
	mut player: Query<&mut Velocity, With<Player>>,
	time: Res<Time>,
	values: Res<MovementValues>,
) {
	let dt = time.delta_seconds();
	let mut player_vel = player.single_mut();

	let mut y_vel = 0.;
	if crouch_input.holding {
		y_vel = -values.vertical_fly_speed * values.mult * dt;
	}
	if jump_input.holding {
		y_vel = values.vertical_fly_speed * values.mult * dt;
	}
	player_vel.vel.y = y_vel;
}

fn update_gravity(values: Res<MovementValues>, mut player: Query<&mut Gravity, With<Player>>) {
	if values.is_changed() {
		let mut gravity = player.single_mut();
		*gravity = Gravity::vertical(values.gravity);
	}
}

fn change_speed_mult(
	scroll_input: Res<ScrollInput>,
	mut values: ResMut<MovementValues>,
	mouse_input: Res<ButtonInput<MouseButton>>,
) {
	values.mult *= (scroll_input.delta() / 8.).exp2();
	if mouse_input.just_pressed(MouseButton::Middle) {
		values.mult = 1.;
	}
}

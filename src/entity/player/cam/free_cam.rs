//! a camera to freely fly around and pass through objects<br>
//! will not rotate the player

use super::{CanRotateCam, PlayerCam, PlayerCamMode};
use crate::{
	entity::{
		player::{Player, EYE_HEIGHT},
		LookDirection,
	},
	global_config::Config,
	input::{CrouchInput, InputSet, JumpInput, RotateInput, ScrollInput, WalkInput},
	GlobalState,
};
use bevy::prelude::*;

pub struct FreeCamPlugin;

impl Plugin for FreeCamPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(FreeCamSpeed::default())
			.add_systems(OnEnter(PlayerCamMode::FreeCam), spawn)
			.add_systems(OnExit(PlayerCamMode::FreeCam), despawn)
			.add_systems(
				Update,
				(
					input_translation,
					input_rotation.run_if(in_state(CanRotateCam(true))),
					change_speed,
				)
					.run_if(in_state(GlobalState::InWorld))
					.run_if(in_state(PlayerCamMode::FreeCam))
					.in_set(InputSet::Use),
			);
	}
}

#[derive(Component)]
#[require(PlayerCam, Transform, Camera3d, LookDirection)]
struct FreeCam;

#[derive(Resource)]
struct FreeCamSpeed {
	speed: f32,
}

impl Default for FreeCamSpeed {
	fn default() -> Self {
		Self { speed: 8. }
	}
}

fn spawn(
	mut commands: Commands,
	player: Query<(&Transform, &LookDirection), With<Player>>,
	global_config: Res<Config>,
) {
	let (player_trans, look_dir) = player.single();
	commands.spawn((
		FreeCam,
		*look_dir,
		Transform::from_translation(player_trans.translation + Vec3::Y * EYE_HEIGHT)
			.with_rotation(look_dir.to_quat()),
		Projection::Perspective(PerspectiveProjection {
			fov: global_config.fov,
			..default()
		}),
	));
}

fn despawn(mut commands: Commands, cam: Query<Entity, With<FreeCam>>) {
	let entity = cam.single();
	commands.entity(entity).despawn_recursive();
}

fn input_translation(
	walk_input: Res<WalkInput>,
	jump_input: Res<JumpInput>,
	crouch_input: Res<CrouchInput>,
	mut cam: Query<(&mut Transform, &LookDirection), With<FreeCam>>,
	time: Res<Time>,
	speed: Res<FreeCamSpeed>,
) {
	let dt = time.delta_secs();
	let (mut cam_trans, look_dir) = cam.single_mut();
	let walk = walk_input.with_look_dir(*look_dir);
	cam_trans.translation += walk * speed.speed * dt;

	if crouch_input.holding {
		cam_trans.translation.y -= speed.speed * dt;
	}
	if jump_input.holding {
		cam_trans.translation.y += speed.speed * dt;
	}
}

fn input_rotation(
	rotate_input: Res<RotateInput>,
	mut cam: Query<(&mut Transform, &mut LookDirection), With<FreeCam>>,
) {
	let (mut cam_trans, mut look_dir) = cam.single_mut();

	*look_dir = rotate_input.rotate_look_dir(*look_dir);
	cam_trans.rotation = look_dir.to_quat();
}

fn change_speed(scroll_input: Res<ScrollInput>, mut speed: ResMut<FreeCamSpeed>) {
	speed.speed *= (scroll_input.delta() / 8.).exp2();
}

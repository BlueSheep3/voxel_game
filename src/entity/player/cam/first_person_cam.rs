//! handles the camera while in first person mode<br>
//! this will rotate the actual player's [`LookDirection`] component, not just the camera

use super::{CanRotateCam, PlayerCam, PlayerCamMode};
use crate::{
	entity::{
		movement::MovementSet,
		player::{Player, EYE_HEIGHT},
		LookDirection,
	},
	global_config::Config,
	input::{InputSet, RotateInput},
	GlobalState,
};
use bevy::prelude::*;

pub struct FirstPersonCamPlugin;

impl Plugin for FirstPersonCamPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(PlayerCamMode::FirstPerson), spawn)
			.add_systems(OnExit(PlayerCamMode::FirstPerson), despawn)
			.add_systems(
				Update,
				(
					move_cam.in_set(MovementSet::Camera),
					input_rotation
						.run_if(in_state(CanRotateCam(true)))
						.in_set(InputSet::Use),
				)
					.run_if(in_state(GlobalState::InWorld))
					.run_if(in_state(PlayerCamMode::FirstPerson)),
			);
	}
}

#[derive(Component)]
struct FirstPersonCam;

fn spawn(
	mut commands: Commands,
	player: Query<(&Transform, &LookDirection), With<Player>>,
	global_config: Res<Config>,
) {
	let (player_trans, look_dir) = player.single();
	commands.spawn((
		PlayerCam,
		FirstPersonCam,
		Camera3dBundle {
			transform: Transform::from_translation(player_trans.translation + Vec3::Y * EYE_HEIGHT)
				.with_rotation(look_dir.to_quat()),
			projection: Projection::Perspective(PerspectiveProjection {
				fov: global_config.fov,
				..default()
			}),
			..default()
		},
	));
}

fn despawn(mut commands: Commands, cam: Query<Entity, With<FirstPersonCam>>) {
	let entity = cam.single();
	commands.entity(entity).despawn_recursive();
}

fn move_cam(
	mut cam: Query<&mut Transform, With<FirstPersonCam>>,
	player: Query<&Transform, (With<Player>, Without<FirstPersonCam>)>,
) {
	let mut cam_trans = cam.single_mut();
	let player_trans = player.single();
	cam_trans.translation = player_trans.translation + Vec3::new(0.0, EYE_HEIGHT, 0.0);
}

fn input_rotation(
	rotate_input: Res<RotateInput>,
	mut player: Query<&mut LookDirection, With<Player>>,
	mut cam: Query<&mut Transform, With<FirstPersonCam>>,
) {
	let mut look_dir = player.single_mut();
	let mut cam_trans = cam.single_mut();

	*look_dir = rotate_input.rotate_look_dir(*look_dir);
	cam_trans.rotation = look_dir.to_quat();
}

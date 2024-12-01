use super::{cam::PlayerCamMode, Player};
use crate::{entity::LookDirection, GlobalState};
use bevy::prelude::*;

pub struct PlayerModelPlugin;

impl Plugin for PlayerModelPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(PlayerCamMode::FreeCam), spawn)
			.add_systems(OnExit(PlayerCamMode::FreeCam), despawn)
			.add_systems(
				Update,
				(update_player_pos, draw_facing_arrow)
					.run_if(in_state(GlobalState::InWorld))
					.run_if(in_state(PlayerCamMode::FreeCam)),
			);
	}
}

#[derive(Component)]
#[require(Transform)]
struct PlayerModel;

fn spawn(
	mut commands: Commands,
	player: Query<&Transform, With<Player>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut meshes: ResMut<Assets<Mesh>>,
) {
	let player_trans = player.single();
	commands.spawn((
		PlayerModel,
		Mesh3d(meshes.add(Mesh::from(Cuboid {
			half_size: Vec3::new(super::WIDTH, super::HEIGHT, super::WIDTH) / 2.,
		}))),
		MeshMaterial3d(materials.add(StandardMaterial {
			unlit: true,
			..default()
		})),
		Transform::from_translation(player_trans.translation + Vec3::Y * super::HEIGHT / 2.),
	));
}

fn despawn(mut commands: Commands, model: Query<Entity, With<PlayerModel>>) {
	for e in model.iter() {
		commands.entity(e).despawn();
	}
}

fn update_player_pos(
	player: Query<&Transform, (With<Player>, Without<PlayerModel>)>,
	mut model: Query<&mut Transform, (With<PlayerModel>, Without<Player>)>,
) {
	let mut model = model.single_mut();
	let player = player.single();
	model.translation = player.translation + Vec3::Y * super::HEIGHT / 2.;
}

fn draw_facing_arrow(
	player: Query<(&Transform, &LookDirection), With<Player>>,
	mut gizmos: Gizmos,
) {
	let (trans, look_dir) = player.single();
	let eye_pos = trans.translation + Vec3::Y * super::EYE_HEIGHT;
	let dir = look_dir.dir();
	gizmos.arrow(eye_pos, eye_pos + dir, Color::srgb(1., 0., 0.));
}

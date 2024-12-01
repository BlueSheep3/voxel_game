use crate::{
	entity::{movement::Velocity, player::Player},
	GlobalState,
};
use bevy::prelude::*;
use std::f32::consts::TAU;

pub struct DebugInfoPlugin;

impl Plugin for DebugInfoPlugin {
	fn build(&self, app: &mut App) {
		app.add_sub_state::<DebugInfoEnabled>()
			.add_systems(
				Update,
				try_toggle_debug_info.run_if(in_state(GlobalState::InWorld)),
			)
			.add_systems(OnEnter(DebugInfoEnabled(true)), spawn_debug_info_text)
			.add_systems(OnExit(DebugInfoEnabled(true)), despawn_debug_info_text)
			.add_systems(
				Update,
				update_debug_info_text.run_if(in_state(DebugInfoEnabled(true))),
			);
	}
}

#[derive(Component)]
struct DebugInfoText;

#[derive(SubStates, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[source(GlobalState = GlobalState::InWorld)]
struct DebugInfoEnabled(bool);

fn try_toggle_debug_info(
	state: Res<State<DebugInfoEnabled>>,
	mut next_state: ResMut<NextState<DebugInfoEnabled>>,
	input: Res<ButtonInput<KeyCode>>,
) {
	if input.just_pressed(KeyCode::F3) {
		next_state.set(DebugInfoEnabled(!state.0));
	}
}

fn spawn_debug_info_text(mut commands: Commands) {
	commands.spawn((Text::new(""), TextFont::from_font_size(20.), DebugInfoText));
}

fn despawn_debug_info_text(mut commands: Commands, debug_text: Query<Entity, With<DebugInfoText>>) {
	let debug_text = debug_text.single();
	commands.entity(debug_text).despawn();
}

fn update_debug_info_text(
	mut query: Query<&mut Text, With<DebugInfoText>>,
	time: Res<Time>,
	cam: Query<&Transform, With<Camera3d>>,
	player: Query<(&Transform, &Velocity), With<Player>>,
	entities: Query<Entity>,
) {
	let mut text = String::new();

	{
		let fps = (1.0 / time.delta_secs()).round();

		text.push_str(&format!("FPS: {fps:03}\n"));
	}

	{
		let entity_count = entities.iter().count();

		text.push_str(&format!("Entities: {entity_count}\n"));
	}

	if let Ok(player) = player.get_single() {
		let pos = player.0.translation;
		let vel = player.1.vel;

		text.push_str(&format!("Pos: {pos:.2?}\nVel: {vel:.2?}\n"));
	}

	if let Ok(cam) = cam.get_single() {
		let rot = cam.rotation;
		let (yaw, pitch, _) = cam.rotation.to_euler(EulerRot::YXZ);
		let (yaw, pitch) = (yaw * 360.0 / TAU, pitch * 360.0 / TAU);

		text.push_str(&format!(
			"Rot: {rot:.2?}\nYaw: {yaw:.2}  Pitch: {pitch:.2}\n"
		));
	}

	let mut text_obj = query.single_mut();
	text_obj.0 = text;
}

use crate::{
	entity::{movement::Velocity, player::Player},
	pos::Vec3Utils,
};
use bevy::prelude::*;
use std::{collections::VecDeque, f32::consts::TAU};

pub struct DebugInfoPlugin;

impl Plugin for DebugInfoPlugin {
	fn build(&self, app: &mut App) {
		app.init_state::<DebugInfoEnabled>()
			.add_systems(Update, try_toggle_debug_info)
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

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
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
	mut delta_time_tracker: Local<VecDeque<f32>>,
) {
	let mut text = String::new();

	{
		let delta_time = time.delta_secs();
		// guarantees that delta_time_tracker is non empty
		delta_time_tracker.push_back(delta_time);
		if delta_time_tracker.len() > 60 {
			delta_time_tracker.pop_front();
		}
		let to_fps = |dt: f32| (1.0 / dt).round();

		let min = *delta_time_tracker
			.iter()
			.min_by(|a, b| a.total_cmp(b))
			.unwrap_or(&1.);
		let max = *delta_time_tracker
			.iter()
			.max_by(|a, b| a.total_cmp(b))
			.unwrap_or(&1.);
		let avg = delta_time_tracker.iter().sum::<f32>() / delta_time_tracker.len() as f32;

		// min and max are flipped, because these show fps instead of ms
		text.push_str(&format!(
			"FPS: {:03}  avg: {:03}  min: {:03}  max: {:03}\n",
			to_fps(delta_time),
			to_fps(avg),
			to_fps(max),
			to_fps(min)
		));
	}

	{
		let entity_count = entities.iter().count();

		text.push_str(&format!("Entities: {entity_count}\n"));
	}

	if let Ok(player) = player.get_single() {
		let pos = player.0.translation;
		let chunk_pos = player.0.translation.to_chunk_pos();
		let pos_in_chunk = player.0.translation.to_block_pos().to_block_in_chunk_pos();
		let vel = player.1.vel;

		text.push_str(&format!("Pos: {pos:.2?}\n"));
		text.push_str(&format!("ChunkPos: {chunk_pos:.2?}\n"));
		text.push_str(&format!("PosInChunk: {pos_in_chunk:.2?}\n"));
		text.push_str(&format!("Vel: {vel:.2?}\n"));
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

use super::{Player, EYE_HEIGHT};
use crate::{
	block::prelude::*,
	entity::{
		collision::ray::{send_out_ray, FiniteRay},
		LookDirection,
	},
	game_world::{chunk::ChunkUpdateEvent, GameWorld},
	input::{AttackInput, InputSet, InteractInput},
	pos::BlockPos,
	GlobalState,
};
use bevy::prelude::*;

pub struct InteractBlockPlugin;

impl Plugin for InteractBlockPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(CurrentBlock::default()).add_systems(
			Update,
			(
				(break_block, place_block).in_set(InputSet::Use),
				select_current_block,
			)
				.run_if(in_state(GlobalState::InWorld)),
		);
	}
}

fn break_block(
	input: Res<AttackInput>,
	player: Query<(&Transform, &LookDirection), With<Player>>,
	mut game_world: ResMut<GameWorld>,
	mut chunk_updates: EventWriter<ChunkUpdateEvent>,
	// mut debug_res: ResMut<DebugRes>,
) {
	if !input.started {
		return;
	}
	let (player_trans, player_look_dir) = player.single();
	let player_look_quat = player_look_dir.to_quat();
	let eye_pos = player_trans.translation + Vec3::Y * EYE_HEIGHT;
	let dir = player_look_quat.mul_vec3(Vec3::new(0.0, 0.0, -1.0));
	let ray = FiniteRay::new(eye_pos, dir, 10.0);

	if let Some(hit) = send_out_ray(ray, &game_world) {
		if let Some(block) = game_world.get_block_at_mut(hit.block_pos) {
			*block = Air::BLOCK;
			send_block_update(hit.block_pos, &mut chunk_updates);
		}
	}

	// let positions = get_all_block_pos_in_ray(ray);
	// debug_res.spawn_temp_cubes(&positions, 6.0);
	// debug_res.spawn_temp_ray(ray, 10.0);
}

#[derive(Resource)]
struct CurrentBlock {
	block: Block,
}

impl Default for CurrentBlock {
	fn default() -> Self {
		Self {
			block: Stone::BLOCK,
		}
	}
}

fn place_block(
	input: Res<InteractInput>,
	player: Query<(&Transform, &LookDirection), With<Player>>,
	mut game_world: ResMut<GameWorld>,
	mut chunk_updates: EventWriter<ChunkUpdateEvent>,
	current_block: Res<CurrentBlock>,
) {
	if !input.started {
		return;
	}
	let (player_trans, player_look_dir) = player.single();
	let player_look_quat = player_look_dir.to_quat();
	let eye_pos = player_trans.translation + Vec3::Y * EYE_HEIGHT;
	let dir = player_look_quat.mul_vec3(Vec3::new(0.0, 0.0, -1.0));
	let ray = FiniteRay::new(eye_pos, dir, 10.0);

	if let Some(hit) = send_out_ray(ray, &game_world) {
		let block_pos = BlockPos(hit.block_pos.0 + hit.face.normal());

		if let Some(block) = game_world.get_block_at_mut(block_pos) {
			if !block.is_replacable() {
				return;
			}
			*block = current_block.block;
			send_block_update(hit.block_pos, &mut chunk_updates);
		}
	}
}

fn select_current_block(input: Res<ButtonInput<KeyCode>>, mut current_block: ResMut<CurrentBlock>) {
	if input.just_pressed(KeyCode::Digit1) {
		current_block.block = Stone::BLOCK;
	}
	if input.just_pressed(KeyCode::Digit2) {
		current_block.block = GrassBlock::BLOCK;
	}
	if input.just_pressed(KeyCode::Digit3) {
		current_block.block = Dirt::BLOCK;
	}
	if input.just_pressed(KeyCode::Digit4) {
		current_block.block = Cobblestone::BLOCK;
	}
	if input.just_pressed(KeyCode::Digit5) {
		current_block.block = DebugBlock::BLOCK;
	}
	if input.just_pressed(KeyCode::Digit6) {
		current_block.block = DebugSlab::BLOCK;
	}
}

fn send_block_update(block_pos: BlockPos, chunk_updates: &mut EventWriter<ChunkUpdateEvent>) {
	let chunk_pos = block_pos.to_chunk_pos();
	chunk_updates.send(ChunkUpdateEvent { chunk_pos });

	// update neighbouring chunks
	for neighbour_pos in block_pos.neighbours() {
		let neighbour_chunk_pos = neighbour_pos.to_chunk_pos();
		if neighbour_chunk_pos == chunk_pos {
			continue;
		}
		chunk_updates.send(ChunkUpdateEvent {
			chunk_pos: neighbour_chunk_pos,
		});
	}
}

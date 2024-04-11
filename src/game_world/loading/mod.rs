//! handles chunk / region loading, like determining which
//! chunks to load and what blocks to put in those chunks

pub mod worldgen;

use super::{chunk::IsLoaded, GameWorld};
use crate::{
	entity::player::Player,
	global_config,
	pos::{ChunkPos, Vec3Utils},
	GlobalState,
};
use bevy::prelude::*;
use std::collections::VecDeque;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<ChunkLoadedEvent>()
			.add_event::<ChunkUnloadedEvent>()
			.insert_resource(ChunkLoadingQueue::default())
			.insert_resource(ChunkUnloadingQueue::default())
			.add_systems(
				Update,
				(
					push_chunk_pos_to_load_queue,
					push_chunk_pos_to_unload_queue,
					load_chunks,
					unload_chunks,
				)
					.run_if(in_state(GlobalState::InWorld)),
			);
	}
}

/// event that is sent when a chunk is loaded<br>
/// the chunk is added to the game_world before sending this event
#[derive(Event, Debug)]
pub struct ChunkLoadedEvent {
	pub pos: ChunkPos,
}

/// event that is sent when a chunk is unloaded<br>
/// the chunk is deleted from the game_world before sending this event
#[derive(Event, Debug)]
pub struct ChunkUnloadedEvent {
	pub pos: ChunkPos,
}

#[derive(Resource, Debug, Default)]
struct ChunkLoadingQueue {
	queue: VecDeque<ChunkPos>,
}

#[derive(Resource, Debug, Default)]
struct ChunkUnloadingQueue {
	queue: VecDeque<ChunkPos>,
}

fn load_chunks(
	mut events: EventWriter<ChunkLoadedEvent>,
	mut game_world: ResMut<GameWorld>,
	mut queue: ResMut<ChunkLoadingQueue>,
) {
	// only load a single chunk at a time to not cause any giant lag spikes
	// the first in the queue to load the closest chunks first
	let Some(pos) = queue.queue.pop_front() else {
		return;
	};

	if let Some(chunk) = game_world.chunks.get_mut(&pos) {
		if chunk.loaded.is_player_loaded() {
			return;
		}
		chunk.loaded.set_player_loaded(true);
	} else {
		let chunk = worldgen::generate_chunk(pos, game_world.seed, IsLoaded::PLAYER_LOADED);
		game_world.chunks.insert(pos, chunk);
	}

	events.send(ChunkLoadedEvent { pos });
}

fn unload_chunks(
	mut events: EventWriter<ChunkUnloadedEvent>,
	mut game_world: ResMut<GameWorld>,
	mut queue: ResMut<ChunkUnloadingQueue>,
) {
	// only unload a single chunk at a time to not cause any giant lag spikes
	// gets the last in the queue to unload the furthest chunks first
	let Some(pos) = queue.queue.pop_back() else {
		return;
	};

	// NOTE current impl just leaves the chunk in the game_world but unloads it.
	// not sure if this might cause problems with memory usage

	let Some(chunk) = game_world.chunks.get_mut(&pos) else {
		return;
	};
	chunk.loaded.set_player_loaded(false);

	events.send(ChunkUnloadedEvent { pos });
}

fn push_chunk_pos_to_load_queue(
	mut queue: ResMut<ChunkLoadingQueue>,
	player: Query<&Transform, With<Player>>,
	game_world: Res<GameWorld>,
	global_config: Res<global_config::Config>,
) {
	let render_distance = (
		global_config.horizontal_render_distance,
		global_config.vertical_render_distance,
	);
	let player = player.single();
	let player_pos = player.translation;
	let chunk_pos_to_load = chunk_pos_in_render_distance(player_pos, render_distance)
		.into_iter()
		.filter(|pos| {
			game_world
				.chunks
				.get(pos)
				.map(|c| !c.loaded.is_player_loaded())
				.unwrap_or(true)
		})
		.filter(|pos| !queue.queue.contains(pos))
		.collect::<Vec<_>>();

	queue.queue.extend(chunk_pos_to_load);
}

fn push_chunk_pos_to_unload_queue(
	mut queue: ResMut<ChunkUnloadingQueue>,
	player: Query<&Transform, With<Player>>,
	game_world: Res<GameWorld>,
	global_config: Res<global_config::Config>,
) {
	let render_distance = (
		global_config.horizontal_render_distance,
		global_config.vertical_render_distance,
	);
	let player = player.single();
	let player_pos = player.translation;
	let in_render_distance = chunk_pos_in_render_distance(player_pos, render_distance);
	let chunk_pos_to_unload = game_world
		.chunks
		.iter()
		.filter(|(_, chunk)| chunk.loaded.is_player_loaded())
		.filter(|(pos, _)| !in_render_distance.contains(pos))
		.filter(|(pos, _)| !queue.queue.contains(pos))
		.map(|(pos, _)| *pos)
		.collect::<Vec<_>>();

	queue.queue.extend(chunk_pos_to_unload);
}

/// gets a list of chunk positions that are in the render distance<br>
/// this is ordered from closest to farthest, so close chunks get loaded first
fn chunk_pos_in_render_distance(player_pos: Vec3, render_distance: (u32, u32)) -> Vec<ChunkPos> {
	// current implementation will load in a square
	// TODO load chunks in a sphere

	let mut chunk_pos_to_load = Vec::new();
	let rdh = render_distance.0 as i32;
	let rdv = render_distance.1 as i32;
	let range = |offset, rd| (offset - rd)..=(offset + rd);
	let current_chunk = player_pos.to_chunk_pos();

	for x in range(current_chunk.0.x, rdh) {
		for y in range(current_chunk.0.y, rdv) {
			for z in range(current_chunk.0.z, rdh) {
				chunk_pos_to_load.push(ChunkPos::new(x, y, z));
			}
		}
	}

	chunk_pos_to_load.sort_by_key(|pos| pos.0.distance_squared(current_chunk.0));

	chunk_pos_to_load
}

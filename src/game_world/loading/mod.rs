//! handles chunk / region loading, like determining which
//! chunks to load and what blocks to put in those chunks

pub mod worldgen;

use self::worldgen::{continue_generation_of_chunk, fully_generate_chunk};

use super::{
	chunk::{GenerationStage, IsLoaded},
	GameWorld,
};
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
		app.add_event::<UpdateChunkIsLoadedEvent>()
			.insert_resource(ChunkLoadingQueue::default())
			.insert_resource(ChunkUnloadingQueue::default())
			.add_systems(
				Update,
				(
					push_chunk_pos_to_load_queue,
					push_chunk_pos_to_unload_queue,
					load_chunks,
					unload_chunks,
					update_chunk_visibility,
				)
					.run_if(in_state(GlobalState::InWorld)),
			);
	}
}

/// event that is sent when the loading state of a chunk changes.<br>
/// the chunk may be added or removed from the game_world before sending this event
#[derive(Event, Debug, Clone, Copy)]
pub struct UpdateChunkIsLoadedEvent {
	pub pos: ChunkPos,
	pub old_is_loaded: IsLoaded,
	pub new_is_loaded: IsLoaded,
}

impl UpdateChunkIsLoadedEvent {
	pub fn just_became_visible(&self) -> bool {
		self.new_is_loaded.is_visible() && !self.old_is_loaded.is_visible()
	}

	pub fn just_became_invisible(&self) -> bool {
		!self.new_is_loaded.is_visible() && self.old_is_loaded.is_visible()
	}
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
	mut events: EventWriter<UpdateChunkIsLoadedEvent>,
	mut game_world: ResMut<GameWorld>,
	mut queue: ResMut<ChunkLoadingQueue>,
) {
	// only load a single chunk at a time to not cause any giant lag spikes
	// the first in the queue to load the closest chunks first
	let Some(pos) = queue.queue.pop_front() else {
		return;
	};

	let loaded;
	if let Some(chunk) = game_world.chunks.get(&pos) {
		if chunk.loaded.is_simple_loaded() {
			return;
		}
		if chunk.generation_state != GenerationStage::COMPLETE {
			continue_generation_of_chunk(&mut game_world, pos);
		}
		let chunk = game_world.chunks.get_mut(&pos).unwrap();
		chunk.loaded.set_simple_loaded(true);
		loaded = chunk.loaded;
	} else {
		fully_generate_chunk(&mut game_world, pos, IsLoaded::SIMPLE_LOADED);
		loaded = game_world.chunks.get(&pos).unwrap().loaded;
	}

	events.send(UpdateChunkIsLoadedEvent {
		pos,
		old_is_loaded: IsLoaded::NOT_LOADED,
		new_is_loaded: loaded,
	});
}

fn unload_chunks(
	mut events: EventWriter<UpdateChunkIsLoadedEvent>,
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
	let prev_loaded = chunk.loaded;
	chunk.loaded.set_simple_loaded(false);
	let loaded = chunk.loaded;

	events.send(UpdateChunkIsLoadedEvent {
		pos,
		old_is_loaded: prev_loaded,
		new_is_loaded: loaded,
	});
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
				.map(|c| !c.loaded.is_simple_loaded())
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
		.filter(|(_, chunk)| chunk.loaded.is_simple_loaded())
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
	// these need to incremented by 1 because only chunks
	// surounded by loaded chunks are actually rendered
	let rdh = render_distance.0 as i32 + 1;
	let rdv = render_distance.1 as i32 + 1;
	let range = |offset, rd| (offset - rd)..=(offset + rd);
	let current_chunk = player_pos.to_chunk_pos();

	for x in range(current_chunk.x, rdh) {
		for y in range(current_chunk.y, rdv) {
			for z in range(current_chunk.z, rdh) {
				chunk_pos_to_load.push(ChunkPos::new(x, y, z));
			}
		}
	}

	chunk_pos_to_load.sort_by_key(|pos| pos.distance_squared(current_chunk));

	chunk_pos_to_load
}

fn update_chunk_visibility(
	mut event_writer: EventWriter<UpdateChunkIsLoadedEvent>,
	mut game_world: ResMut<GameWorld>,
) {
	for pos in game_world.chunks.keys().cloned().collect::<Vec<_>>() {
		let is_loaded = game_world.chunks.get(&pos).unwrap().loaded;
		if !is_loaded.is_simple_loaded() {
			// chunks that aren't loaded can't be visible
			continue;
		}
		let should_be_visible = are_surounding_chunks_loaded(pos, &game_world);
		let Some(chunk) = game_world.chunks.get_mut(&pos) else {
			continue;
		};
		let old_loaded = chunk.loaded;
		chunk.loaded.set_visible(should_be_visible);
		if old_loaded == chunk.loaded {
			// nothing changed => no event should be sent
			continue;
		}
		event_writer.send(UpdateChunkIsLoadedEvent {
			pos,
			old_is_loaded: old_loaded,
			new_is_loaded: chunk.loaded,
		});
	}
}

fn are_surounding_chunks_loaded(pos: ChunkPos, game_world: &GameWorld) -> bool {
	for pos in pos.neighbours() {
		let Some(chunk) = game_world.chunks.get(&pos) else {
			return false;
		};
		if !chunk.loaded.is_simple_loaded() {
			return false;
		}
	}
	true
}

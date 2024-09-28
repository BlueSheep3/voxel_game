mod combine_mesh;
mod mesh;

use self::mesh::create_chunk_mesh;
use super::ChunkUpdateEvent;
use crate::{
	block_model::{ChunkMaterial, GlobalTexture, LoadingState},
	face::FaceMap,
	game_world::{loading::UpdateChunkIsLoadedEvent, GameWorld},
	pos::ChunkPos,
	GlobalState,
};
use bevy::{
	pbr::ExtendedMaterial,
	prelude::*,
	tasks::{block_on, AsyncComputeTaskPool, Task},
	utils::HashMap,
};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(QueuedChunkRedraws::default())
			.insert_resource(ChunkMeshEntities::default())
			.insert_resource(MeshTasks::default())
			.add_systems(OnEnter(LoadingState::Done), setup_global_material)
			.add_systems(OnEnter(GlobalState::InWorld), init)
			.add_systems(OnExit(GlobalState::InWorld), cleanup)
			.add_systems(
				Update,
				(
					create_chunk_redraw_tasks,
					spawn_chunk_meshes_from_tasks,
					despawn_chunk_on_unload,
					stop_chunk_redraw_tasks_on_unload,
					queue_loading_chunks,
					queue_updating_chunks,
				)
					.run_if(has_loaded_global_material)
					.run_if(in_state(GlobalState::InWorld)),
			);
	}
}

#[derive(Component)]
struct ChunkMesh;

// exaclty one ChunkMeshParent should exist while GlobalState::InWorld, otherwise zero
#[derive(Component)]
struct ChunkMeshParent;

#[derive(Resource, Default)]
struct QueuedChunkRedraws {
	queue: Vec<ChunkRedrawInfo>,
}

// TODO in the future only redraw a few blocks, instead of the entire chunk
struct ChunkRedrawInfo {
	chunk_pos: ChunkPos,
}

#[derive(Resource, Default)]
struct ChunkMeshEntities {
	entities: HashMap<ChunkPos, Entity>,
}

#[derive(Resource)]
struct GlobalChunkMaterial {
	material: Handle<ExtendedMaterial<StandardMaterial, ChunkMaterial>>,
}

fn has_loaded_global_material(world: &World) -> bool {
	world.contains_resource::<GlobalChunkMaterial>()
}

fn setup_global_material(
	mut commands: Commands,
	global_texture: Res<GlobalTexture>,
	mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ChunkMaterial>>>,
) {
	let global_material_handle = materials.add(ExtendedMaterial {
		base: StandardMaterial {
			unlit: true,
			..default()
		},
		extension: ChunkMaterial {
			texture: global_texture.image.clone(),
		},
	});

	let global_material = GlobalChunkMaterial {
		material: global_material_handle,
	};

	commands.insert_resource(global_material);

	info!("global material inserted");
}

fn init(mut commands: Commands) {
	commands.spawn((
		SpatialBundle::default(),
		ChunkMeshParent,
		Name::new("ChunkMeshParent"),
	));
}

fn cleanup(
	mut commands: Commands,
	chunk_mesh_parent: Query<Entity, With<ChunkMeshParent>>,
	mut mesh_entites: ResMut<ChunkMeshEntities>,
	mut queued_chunk_redraws: ResMut<QueuedChunkRedraws>,
	mut mesh_tasks: ResMut<MeshTasks>,
) {
	if let Ok(chunk_mesh_parent) = chunk_mesh_parent.get_single() {
		commands.entity(chunk_mesh_parent).despawn_recursive();
	}
	*mesh_entites = default();
	*queued_chunk_redraws = default();
	*mesh_tasks = default();
}

/// queues the chunks that are currently being loaded into the render distance
fn queue_loading_chunks(
	mut chunk_loading_event: EventReader<UpdateChunkIsLoadedEvent>,
	mut queued_chunks: ResMut<QueuedChunkRedraws>,
) {
	for event in chunk_loading_event.read() {
		if !event.just_became_visible() {
			continue;
		}
		queued_chunks.queue.push(ChunkRedrawInfo {
			chunk_pos: event.pos,
		});
	}
}

/// queues the chunks that are currently being changed
fn queue_updating_chunks(
	mut chunk_updating_event: EventReader<ChunkUpdateEvent>,
	mut queued_chunks: ResMut<QueuedChunkRedraws>,
	game_world: Res<GameWorld>,
) {
	for event in chunk_updating_event.read() {
		let Some(chunk) = game_world.chunks.get(&event.chunk_pos) else {
			continue;
		};
		if !chunk.loaded.is_simple_loaded() {
			continue;
		}
		queued_chunks.queue.push(ChunkRedrawInfo {
			chunk_pos: event.chunk_pos,
		});
	}
}

fn despawn_chunk_on_unload(
	mut chunk_loading_event: EventReader<UpdateChunkIsLoadedEvent>,
	mut commands: Commands,
	mut mesh_entites: ResMut<ChunkMeshEntities>,
	chunk_mesh_parent: Query<Entity, With<ChunkMeshParent>>,
) {
	let chunk_mesh_parent = chunk_mesh_parent.single();
	for event in chunk_loading_event.read() {
		if !event.just_became_invisible() {
			continue;
		}
		if let Some(entity) = mesh_entites.entities.remove(&event.pos) {
			// currently a child has to manually removed from the parent
			commands
				.entity(chunk_mesh_parent)
				.remove_children(&[entity]);
			commands.entity(entity).despawn();
		}
	}
}

fn stop_chunk_redraw_tasks_on_unload(
	mut chunk_loading_event: EventReader<UpdateChunkIsLoadedEvent>,
	mut mesh_tasks: ResMut<MeshTasks>,
) {
	for event in chunk_loading_event.read() {
		if !event.just_became_invisible() {
			continue;
		}
		if let Some(task) = mesh_tasks.tasks.remove(&event.pos) {
			block_on(task.cancel());
		}
	}
}

#[derive(Resource, Debug, Default)]
struct MeshTasks {
	tasks: HashMap<ChunkPos, Task<Mesh>>,
}

fn create_chunk_redraw_tasks(
	mut queued_chunk_redraws: ResMut<QueuedChunkRedraws>,
	game_world: Res<GameWorld>,
	global_texture: Res<GlobalTexture>,
	mut mesh_tasks: ResMut<MeshTasks>,
) {
	if queued_chunk_redraws.queue.is_empty() {
		return;
	}

	let ChunkRedrawInfo { chunk_pos } = queued_chunk_redraws.queue.remove(0);
	let chunk = game_world
		.chunks
		.get(&chunk_pos)
		.expect("got chunk update event, even though there is no chunk");

	let block_models = global_texture.mappings.clone();
	let cloned_chunk = chunk.clone();

	let neighbour_chunks = FaceMap::from_map(|face| {
		let pos = chunk_pos + face.normal();
		game_world.chunks.get(&pos).cloned()
	})
	.all_some();
	let Some(neighbour_chunks) = neighbour_chunks else {
		// a chunk must have neighbouring chunks to be drawable,
		// because of culling blocks at the edge of the chunk
		return;
	};
	if neighbour_chunks
		.iter()
		.any(|chunk| !chunk.loaded.is_simple_loaded())
	{
		return;
	}

	let pool = AsyncComputeTaskPool::get();
	let task = pool
		.spawn(async move { create_chunk_mesh(&cloned_chunk, &neighbour_chunks, &block_models) });
	mesh_tasks.tasks.insert(chunk_pos, task);
}

fn spawn_chunk_meshes_from_tasks(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	global_material: Res<GlobalChunkMaterial>,
	mut mesh_entites: ResMut<ChunkMeshEntities>,
	mut mesh_tasks: ResMut<MeshTasks>,
	chunk_mesh_parent: Query<Entity, With<ChunkMeshParent>>,
) {
	let chunk_mesh_parent = chunk_mesh_parent.single();
	let keys = mesh_tasks.tasks.keys().cloned().collect::<Vec<_>>();
	for chunk_pos in keys {
		let Some(task) = mesh_tasks.tasks.get(&chunk_pos) else {
			unreachable!()
		};
		if !task.is_finished() {
			continue;
		}
		let Some(task) = mesh_tasks.tasks.remove(&chunk_pos) else {
			unreachable!()
		};

		let mesh = block_on(task);
		let cube_mesh_handle = meshes.add(mesh);

		// PERF it would be more efficient to update the entity instead of creating a new one
		if mesh_entites.entities.contains_key(&chunk_pos) {
			let entity = mesh_entites.entities.remove(&chunk_pos).unwrap();
			commands
				.entity(chunk_mesh_parent)
				.remove_children(&[entity]);
			commands.entity(entity).despawn();
		}

		let entity = commands
			.spawn((
				MaterialMeshBundle {
					mesh: cube_mesh_handle,
					material: global_material.material.clone(),
					transform: Transform::from_translation(chunk_pos.to_world_pos()),
					..default()
				},
				ChunkMesh,
				Name::new(format!("Chunk Mesh at {}", chunk_pos)),
			))
			.id();
		commands.entity(chunk_mesh_parent).add_child(entity);

		mesh_entites.entities.insert(chunk_pos, entity);
	}
}

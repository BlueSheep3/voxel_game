mod combine_mesh;
mod mesh;

use self::mesh::{create_chunk_mesh, FaceTextureIndices};
use super::ChunkUpdateEvent;
use crate::{
	bench::BenchName,
	block_model::{
		chunk_material::{ChunkMaterial, ExtendedChunkMaterial},
		GlobalTexture,
	},
	face::FaceMap,
	game_world::{loading::UpdateChunkIsLoadedEvent, GameWorld},
	pos::ChunkPos,
	GlobalState,
};
use bevy::{
	asset::RenderAssetUsages,
	pbr::ExtendedMaterial,
	prelude::*,
	render::render_resource::{Extent3d, TextureDimension, TextureFormat},
	tasks::{block_on, AsyncComputeTaskPool, Task},
};
use std::{collections::HashMap, time::Instant};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(QueuedChunkRedraws::default())
			.insert_resource(ChunkMeshEntities::default())
			.insert_resource(MeshTasks::default())
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
					.run_if(in_state(GlobalState::InWorld)),
			);
	}
}

#[derive(Component)]
#[require(Mesh3d, Transform)]
struct ChunkMesh;

// exaclty one ChunkMeshParent should exist while GlobalState::InWorld, otherwise zero
#[derive(Component)]
#[require(Transform, Visibility)]
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

fn init(mut commands: Commands) {
	commands.spawn((ChunkMeshParent, Name::new("ChunkMeshParent")));
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
	tasks: HashMap<ChunkPos, Task<(Mesh, FaceTextureIndices)>>,
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

	// benchmarking of thread spawning
	let start_time = Instant::now();

	let neighbour_chunks = FaceMap::from_map(|face| {
		let pos = chunk_pos + face.normal();
		game_world.chunks.get(&pos)
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

	let cloned_chunk = chunk.clone();
	let block_models = global_texture.mappings.clone();
	let chunk_padding = mesh::chunk_padding_from_neighbour_chunks(neighbour_chunks);

	let pool = AsyncComputeTaskPool::get();
	#[rustfmt::skip]
	let task = pool.spawn(
		// using a Box here because this returned Future is about 1000 bytes big
		Box::pin(create_chunk_mesh(cloned_chunk, chunk_padding, block_models))
	);
	mesh_tasks.tasks.insert(chunk_pos, task);

	crate::bench::push_time(BenchName::SpawnThread, start_time.elapsed());
}

#[allow(clippy::too_many_arguments)]
fn spawn_chunk_meshes_from_tasks(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	global_texture: Res<GlobalTexture>,
	mut mesh_entites: ResMut<ChunkMeshEntities>,
	mut mesh_tasks: ResMut<MeshTasks>,
	chunk_mesh_parent: Query<Entity, With<ChunkMeshParent>>,
	mut materials: ResMut<Assets<ExtendedChunkMaterial>>,
	mut images: ResMut<Assets<Image>>,
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

		let (mesh, face_texture_indices) = block_on(task);
		let cube_mesh_handle = meshes.add(mesh);

		// PERF it would be more efficient to update the entity instead of creating a new one
		if mesh_entites.entities.contains_key(&chunk_pos) {
			let entity = mesh_entites.entities.remove(&chunk_pos).unwrap();
			commands
				.entity(chunk_mesh_parent)
				.remove_children(&[entity]);
			commands.entity(entity).despawn();
		}

		// assets are ref counted, so when the chunk is unloaded the asset will be dropped
		let material = create_chunk_material(
			face_texture_indices,
			&global_texture,
			&mut materials,
			&mut images,
		);

		let entity = commands
			.spawn((
				Mesh3d(cube_mesh_handle),
				MeshMaterial3d(material),
				Transform::from_translation(chunk_pos.to_world_pos()),
				ChunkMesh,
				Name::new(format!("Chunk Mesh at {}", chunk_pos)),
			))
			.id();
		commands.entity(chunk_mesh_parent).add_child(entity);

		mesh_entites.entities.insert(chunk_pos, entity);
	}
}

fn create_chunk_material(
	mut face_texture_indices: FaceTextureIndices,
	global_texture: &Res<GlobalTexture>,
	materials: &mut ResMut<Assets<ExtendedChunkMaterial>>,
	images: &mut ResMut<Assets<Image>>,
) -> Handle<ExtendedChunkMaterial> {
	// the width of any image must be non zero.
	// this can only happen if the chunk is empty,
	// so we can just insert some garbage data.
	if face_texture_indices.len() == 0 {
		face_texture_indices.push_index(0);
	}
	let indices = Image::new(
		Extent3d {
			width: face_texture_indices.len() as u32,
			height: 1,
			depth_or_array_layers: 1,
		},
		TextureDimension::D1,
		face_texture_indices.into_bytes(),
		TextureFormat::R32Uint,
		RenderAssetUsages::RENDER_WORLD,
	);
	let indices = images.add(indices);

	materials.add(ExtendedMaterial {
		base: StandardMaterial {
			unlit: true,
			..default()
		},
		extension: ChunkMaterial {
			block_textures: global_texture.image.clone(),
			face_texture_indices: indices,
		},
	})
}

mod combine_mesh;
mod mesh;

use self::mesh::create_chunk_mesh;
use super::{Chunk, ChunkUpdateEvent, CHUNK_LENGTH};
use crate::{
	axis::{Axis, AxisMap},
	bench::BenchName,
	block::BlockId,
	block_model::{BlockModel, ChunkMaterial, GlobalTexture, LoadingState},
	face::FaceMap,
	game_world::{loading::UpdateChunkIsLoadedEvent, GameWorld},
	pos::{BlockInChunkPos, ChunkPos},
	GlobalState,
};
use bevy::{
	pbr::ExtendedMaterial,
	prelude::*,
	tasks::{block_on, AsyncComputeTaskPool, Task},
};
use std::{collections::HashMap, time::Instant};

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

	let blocks_mask = get_blocks_bitmask(chunk, &global_texture.mappings, neighbour_chunks);
	let cloned_chunk = chunk.clone();
	let block_models = global_texture.mappings.clone();

	let pool = AsyncComputeTaskPool::get();
	#[rustfmt::skip]
	let task = pool.spawn(async move {
		create_chunk_mesh(cloned_chunk, blocks_mask, block_models)
	});
	mesh_tasks.tasks.insert(chunk_pos, task);

	crate::bench::push_time(BenchName::SpawnThread, start_time.elapsed());
}

/// Will generate 3 bitmasks for this chunk, one for each axis.
/// The bits just mean whether a cullable block is there or not.
/// The 0th bit is an edge block of the negative facing neighbour chunk,
/// while the 33rd bit is the edge block of the opposite chunk,
/// and all bits in between (1st to 32nd inclusive) are the current chunk.
fn get_blocks_bitmask(
	chunk: &Chunk,
	block_models: &HashMap<BlockId, BlockModel<usize>>,
	neighbour_chunks: FaceMap<&Chunk>,
) -> Box<AxisMap<[[u64; CHUNK_LENGTH]; CHUNK_LENGTH]>> {
	// start out with a completely empty mask
	let mut blocks_mask = AxisMap::<[[u64; CHUNK_LENGTH]; CHUNK_LENGTH]>::default();

	// fill in the current chunk
	for (pos, block) in chunk.blocks.iter_xyz() {
		let BlockInChunkPos { x, y, z } = pos;
		let [x, y, z] = [x as usize, y as usize, z as usize];
		if block_models[&block.id].should_cull {
			blocks_mask[Axis::X][y][z] |= 1 << (x + 1);
			blocks_mask[Axis::Y][x][z] |= 1 << (y + 1);
			blocks_mask[Axis::Z][x][y] |= 1 << (z + 1);
		}
	}

	// fill in the edges of the neighbouring chunks
	macro_rules! neighbours {
		($(($a:ident, $b:ident) in ($axis:expr, $axis_name:ident)
		=> [$x:expr, $y:expr, $z:expr]);* $(;)?) => {
			$(
			for $a in 0..CHUNK_LENGTH {
				for $b in 0..CHUNK_LENGTH {
					let mut pos = BlockInChunkPos::new($x, $y, $z);

					pos.$axis_name = CHUNK_LENGTH as u8 - 1;
					let block = neighbour_chunks[$axis.face_neg()].blocks[pos];
					if block_models[&block.id].should_cull {
						blocks_mask[$axis][$a][$b] |= 1;
					}
					pos.$axis_name = 0;
					let block = neighbour_chunks[$axis.face_pos()].blocks[pos];
					if block_models[&block.id].should_cull {
						blocks_mask[$axis][$a][$b] |= 1 << (CHUNK_LENGTH + 1);
					}
				}
			}
			)*
		};
	}
	neighbours! {
		(y, z) in (Axis::X, x) => [0, y as u8, z as u8];
		(x, z) in (Axis::Y, y) => [x as u8, 0, z as u8];
		(x, y) in (Axis::Z, z) => [x as u8, y as u8, 0];
	}

	// box the blocks_mask, so that its cheap to move around,
	// because its a *lot* of data
	// TODO check if this has better performance if this is put into a box earlier
	Box::new(blocks_mask)
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
				Mesh3d(cube_mesh_handle),
				MeshMaterial3d(global_material.material.clone()),
				Transform::from_translation(chunk_pos.to_world_pos()),
				ChunkMesh,
				Name::new(format!("Chunk Mesh at {}", chunk_pos)),
			))
			.id();
		commands.entity(chunk_mesh_parent).add_child(entity);

		mesh_entites.entities.insert(chunk_pos, entity);
	}
}

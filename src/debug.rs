use crate::{
	entity::{collision::ray::FiniteRay, player::Player},
	game_world::chunk::CHUNK_LENGTH,
	pos::{BlockPos, Vec3Utils},
};
use bevy::{color::palettes::basic::YELLOW, prelude::*};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<DrawChunkBorders>()
			.register_type::<DrawChunkBorders>()
			.init_resource::<DebugRes>()
			.add_systems(
				Update,
				(
					spawn_temp_cubes,
					spawn_temp_lines,
					despawn_temp_cubes,
					toggle_chunk_borders,
					draw_chunk_borders,
				),
			);
	}
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct DrawChunkBorders {
	enabled: bool,
	color: Color,
	/// from the center chunk, how many more chunks the grid goes out
	half_size: UVec3,
}

impl Default for DrawChunkBorders {
	fn default() -> Self {
		Self {
			enabled: false,
			color: YELLOW.into(),
			half_size: UVec3::splat(2),
		}
	}
}

#[derive(Resource, Default)]
pub struct DebugRes {
	queued_temp_cubes: Vec<(BlockPos, f32)>,
	queued_temp_lines: Vec<(Vec3, Vec3, f32, f32)>,
}

#[allow(dead_code)]
impl DebugRes {
	pub fn spawn_temp_cube(&mut self, pos: BlockPos, seconds: f32) {
		self.queued_temp_cubes.push((pos, seconds));
	}

	pub fn spawn_temp_cubes(&mut self, pos: &[BlockPos], seconds: f32) {
		self.queued_temp_cubes
			.extend(pos.iter().map(|p| (*p, seconds)));
	}

	pub fn spawn_temp_line(&mut self, pos: Vec3, dir: Vec3, len: f32, seconds: f32) {
		self.queued_temp_lines.push((pos, dir, len, seconds));
	}

	pub fn spawn_temp_ray(&mut self, ray: FiniteRay, seconds: f32) {
		let FiniteRay { start, dir, length } = ray;
		self.spawn_temp_line(start, dir, length, seconds);
	}
}

#[derive(Component)]
struct TempCube(f32);

fn spawn_temp_cubes(
	mut debug_res: ResMut<DebugRes>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	for (pos, seconds) in debug_res.queued_temp_cubes.drain(..) {
		commands.spawn((
			Mesh3d(meshes.add(Mesh::from(Cuboid {
				half_size: Vec3::splat(0.55),
			}))),
			MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.5, 0.0, 0.5)))),
			Transform::from_translation(pos.to_world_pos() + 0.525),
			TempCube(seconds),
		));
	}
}

fn spawn_temp_lines(
	mut debug_res: ResMut<DebugRes>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
) {
	for (pos, dir, len, seconds) in debug_res.queued_temp_lines.drain(..) {
		let mut trans = Transform::from_translation(pos - Vec3::Z * len / 2.0);
		trans.rotate_around(pos, Quat::from_rotation_arc(-Vec3::Z, dir));

		let cuboid_mesh = Mesh::from(Cuboid {
			half_size: Vec3::new(0.01, 0.01, len / 2.0),
		});

		commands.spawn((Mesh3d(meshes.add(cuboid_mesh)), trans, TempCube(seconds)));
	}
}

fn despawn_temp_cubes(
	mut query: Query<(Entity, &mut TempCube)>,
	time: Res<Time>,
	mut commands: Commands,
) {
	let dt = time.delta_secs();
	for (id, mut cube) in &mut query {
		cube.0 -= dt;
		if cube.0 <= 0.0 {
			commands.entity(id).despawn();
		}
	}
}

fn toggle_chunk_borders(
	input: Res<ButtonInput<KeyCode>>,
	mut draw_chunk_borders: ResMut<DrawChunkBorders>,
) {
	if input.just_pressed(KeyCode::KeyB) {
		draw_chunk_borders.enabled ^= true;
	}
}

fn draw_chunk_borders(
	draw_chunk_borders: Res<DrawChunkBorders>,
	mut gizmos: Gizmos,
	player: Query<&Transform, With<Player>>,
) {
	if !draw_chunk_borders.enabled {
		return;
	}
	let Ok(player) = player.get_single() else {
		return;
	};
	let chunk_pos = player.translation.to_chunk_pos();
	let center = chunk_pos.to_world_pos() + Vec3::splat(CHUNK_LENGTH as f32 / 2.);
	gizmos
		.grid_3d(
			Isometry3d::from_translation(center),
			draw_chunk_borders.half_size * 2 + UVec3::ONE,
			Vec3::splat(CHUNK_LENGTH as f32),
			draw_chunk_borders.color,
		)
		.outer_edges();
}

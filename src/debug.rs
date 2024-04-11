use crate::{entity::collision::ray::FiniteRay, pos::BlockPos};
use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(DebugRes::default()).add_systems(
			Update,
			(spawn_temp_cubes, spawn_temp_lines, despawn_temp_cubes),
		);
	}
}

#[derive(Resource, Default)]
pub struct DebugRes {
	queued_temp_cubes: Vec<(BlockPos, f32)>,
	queued_temp_lines: Vec<(Vec3, Vec3, f32, f32)>,
}

impl DebugRes {
	#[allow(dead_code)]
	pub fn spawn_temp_cube(&mut self, pos: BlockPos, seconds: f32) {
		self.queued_temp_cubes.push((pos, seconds));
	}

	#[allow(dead_code)]
	pub fn spawn_temp_cubes(&mut self, pos: &[BlockPos], seconds: f32) {
		self.queued_temp_cubes
			.extend(pos.iter().map(|p| (*p, seconds)));
	}

	#[allow(dead_code)]
	pub fn spawn_temp_line(&mut self, pos: Vec3, dir: Vec3, len: f32, seconds: f32) {
		self.queued_temp_lines.push((pos, dir, len, seconds));
	}

	#[allow(dead_code)]
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
			PbrBundle {
				mesh: meshes.add(Mesh::from(Cuboid {
					half_size: Vec3::splat(0.55),
				})),
				transform: Transform::from_translation(pos.to_world_pos() + 0.525),
				material: materials.add(StandardMaterial::from(Color::rgb(0.5, 0.0, 0.5))),
				..default()
			},
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
		// FIXME janky code
		commands.spawn((
			PbrBundle {
				mesh: meshes.add(Mesh::from(Cuboid {
					half_size: Vec3::new(0.01, 0.01, len / 2.0),
				})),
				transform: {
					let mut trans = Transform::from_translation(pos - Vec3::Z * len / 2.0);
					trans.rotate_around(pos, Quat::from_rotation_arc(-Vec3::Z, dir));
					trans
				},
				..default()
			},
			TempCube(seconds),
		));
	}
}

fn despawn_temp_cubes(
	mut query: Query<(Entity, &mut TempCube)>,
	time: Res<Time>,
	mut commands: Commands,
) {
	let dt = time.delta_seconds();
	for (id, mut cube) in &mut query {
		cube.0 -= dt;
		if cube.0 <= 0.0 {
			commands.entity(id).despawn();
		}
	}
}

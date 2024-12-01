use super::collision::collider::BoxCollider;
use crate::GlobalState;
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				move_without_collision.in_set(MovementSet::Translate),
				gravity.in_set(MovementSet::Accel),
				update_prev_vel.in_set(MovementSet::CleanUp),
			)
				// NOTE not sure if this should only run InWorld
				.run_if(in_state(GlobalState::InWorld)),
		)
		.configure_sets(
			Update,
			(
				MovementSet::Accel,
				MovementSet::Translate,
				MovementSet::Camera,
				// MovementSet::Get,
				MovementSet::CleanUp,
			)
				.chain(),
		);
	}
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum MovementSet {
	/// changes velocity / accelerates
	Accel,
	/// uses velocity to change position
	Translate,
	/// moves the camera. this needs to be done after
	/// [`MovementSet::Translate`], to make the camera not jittery
	Camera,
	// /// Gets information about movement without changing anything about it.
	// Get,
	/// done at the end of the frame to clean up (set previous velocity, etc.)
	CleanUp,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Velocity {
	pub vel: Vec3,
	prev_vel: Vec3,
}

impl Velocity {
	pub fn delta(self) -> Vec3 {
		self.vel - self.prev_vel
	}
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Gravity(pub Vec3);

impl Gravity {
	pub const ZERO: Self = Self(Vec3::ZERO);

	pub const fn vertical(strength: f32) -> Self {
		Self(Vec3::new(0.0, strength, 0.0))
	}
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct OnGround(pub bool);

fn move_without_collision(
	mut query: Query<(&mut Transform, &Velocity), Without<BoxCollider>>,
	time: Res<Time>,
) {
	let dt = time.delta_secs();
	for (mut trans, vel) in &mut query {
		// compute:  v += a/2;  s += v;  v += a/2;
		// because this is the most accurate approximation of continuos motion
		let v_plus_half = vel.vel - vel.delta() / 2.;
		trans.translation += v_plus_half * dt;
	}
}

fn gravity(mut query: Query<(&mut Velocity, &Gravity)>, time: Res<Time>) {
	let dt = time.delta_secs();
	for (mut vel, grav) in &mut query {
		vel.vel += grav.0 * dt;
	}
}

fn update_prev_vel(mut query: Query<&mut Velocity>) {
	for mut vel in &mut query {
		vel.prev_vel = vel.vel;
	}
}

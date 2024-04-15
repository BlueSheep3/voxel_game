mod cam;
mod interact_block;
mod movement;
mod player_model;

use super::{
	collision::collider::BoxCollider,
	movement::{Gravity, OnGround, Velocity},
	LookDirection,
};
use crate::game_world::{get_height_at_with_seed, GameWorld};
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			cam::CamPlugin,
			interact_block::InteractBlockPlugin,
			movement::MovementPlugin,
			player_model::PlayerModelPlugin,
		))
		.add_systems(Startup, setup);
	}
}

const GRAVITY: f32 = -20.0;
const WIDTH: f32 = 0.8;
const HEIGHT: f32 = 1.85;
const EYE_HEIGHT: f32 = 1.65;

/// The entity representing the player you control.
/// This is not responsible for visuals or other players when playing online.
#[derive(Component, Default, Debug, Clone)]
pub struct Player;

#[derive(Bundle)]
struct PlayerBundle {
	player: Player,
	transform: Transform,
	velocity: Velocity,
	gravity: Gravity,
	look_direction: LookDirection,
	collider: BoxCollider,
	on_ground: OnGround,
	name: Name,
}

impl Default for PlayerBundle {
	fn default() -> Self {
		Self {
			player: Player,
			transform: Transform::default(),
			velocity: Velocity::default(),
			gravity: Gravity::vertical(GRAVITY),
			look_direction: LookDirection::default(),
			collider: BoxCollider::new(WIDTH, HEIGHT),
			on_ground: OnGround::default(),
			name: Name::new("Player"),
		}
	}
}

fn setup(mut commands: Commands, game_world: Res<GameWorld>) {
	let height = get_height_at_with_seed(16, 16, game_world.seed) as f32 + 4.;
	commands.spawn(PlayerBundle {
		transform: Transform::from_xyz(16.5, height, 16.5),
		..default()
	});
}

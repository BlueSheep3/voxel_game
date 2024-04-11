pub mod collider;
mod move_and_slide;
pub mod ray;

use bevy::prelude::*;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(move_and_slide::MoveAndSlidePlugin);
	}
}

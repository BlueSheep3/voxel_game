pub mod collision;
pub mod movement;
pub mod player;

use bevy::prelude::*;

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			player::PlayerPlugin,
			collision::CollisionPlugin,
			movement::MovementPlugin,
		));
	}
}

/// the direction the entity is looking in, as pitch and yaw<br>
/// is independant from the rotation of the entity's transform
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct LookDirection {
	pub pitch: f32,
	pub yaw: f32,
}

impl LookDirection {
	pub fn to_quat(self) -> Quat {
		let pitch = Quat::from_rotation_x(self.pitch);
		let yaw = Quat::from_rotation_y(self.yaw);

		// i dont know how this works (dont switch the order of yaw and pitch)
		yaw * pitch
	}

	// fn from_quat(q: Quat) -> Self {
	// 	Self {
	// 		pitch: q.x,
	// 		yaw: q.y,
	// 	}
	// }

	pub fn dir(self) -> Vec3 {
		self.to_quat() * -Vec3::Z
	}
}

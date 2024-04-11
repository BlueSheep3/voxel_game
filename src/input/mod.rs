//! abstracted input handling for keyboard, mouse, controller

mod gamepad;
mod input_funcs;
mod keyboard;
mod mouse;

use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			keyboard::KeyboardPlugin,
			mouse::MousePlugin,
			gamepad::GamepadPlugin,
		))
		.insert_resource(WalkInput::default())
		.insert_resource(RotateInput::default())
		.insert_resource(JumpInput::default())
		.insert_resource(CrouchInput::default())
		.insert_resource(AttackInput::default())
		.insert_resource(InteractInput::default())
		.add_systems(Update, cleanup_input_resources.in_set(InputSet::CleanUp))
		.configure_sets(
			Update,
			(InputSet::Get, InputSet::Use, InputSet::CleanUp).chain(),
		);
	}
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub enum InputSet {
	/// this system gets input from the user and stores it in some resource.<br>
	/// only used for directly getting input, so not for connecting controllers for example.<br>
	/// this should be done in a way that doesn't interfere with having
	/// multiple input devices at the same time.
	Get,
	/// this system uses the input from an input resource (not directly the user)
	Use,
	/// resets all input values, to be usable on the next frame<br>
	/// this is used instead of simply overriding the values in their own systems
	/// to allow multiple devices to send input at the same time
	CleanUp,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct WalkInput {
	/// The direction the player is trying to move,
	/// where `+x` is right and `+y` is forward.<br>
	/// (is already clamped to have a max length of 1)
	vec: Vec2,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct RotateInput {
	/// how much to rotate the camera along the yz plane<br>
	/// do NOT multiply this by delta time, since it measures how far
	/// you moved since the last frame, and you want to move that entire distance
	pitch: f32,
	/// how much to rotate the camera along the xz plane<br>
	/// do NOT multiply this by delta time, since it measures how far
	/// you moved since the last frame, and you want to move that entire distance
	yaw: f32,
}

macro_rules! single_button {
	($name:ident, $doc_text:expr) => {
		#[derive(Resource, Debug, Clone, Copy, Default)]
		pub struct $name {
			/// whether the player is currently holding
			#[doc = $doc_text]
			pub holding: bool,
			/// whether the player has just started pressing
			#[doc = $doc_text]
			pub started: bool,
		}
	};
}

single_button! { JumpInput, "jump" }
single_button! { CrouchInput, "crouch" }
single_button! { AttackInput, "attack" }
single_button! { InteractInput, "interact" }

fn cleanup_input_resources(
	mut walk_input: ResMut<WalkInput>,
	mut rotate_input: ResMut<RotateInput>,
	mut jump_input: ResMut<JumpInput>,
	mut crouch_input: ResMut<CrouchInput>,
	mut attack_input: ResMut<AttackInput>,
	mut interact_input: ResMut<InteractInput>,
) {
	*walk_input = WalkInput::default();
	*rotate_input = RotateInput::default();
	*jump_input = JumpInput::default();
	*crouch_input = CrouchInput::default();
	*attack_input = AttackInput::default();
	*interact_input = InteractInput::default();
}

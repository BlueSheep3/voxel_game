//! input handling for gamepad (controller / joystick)

use super::{AttackInput, CrouchInput, InputSet, InteractInput, JumpInput, RotateInput, WalkInput};
use crate::savedata;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Controls::load().unwrap_or_default())
			.add_systems(
				Update,
				((
					get_walk_input,
					get_rotate_input,
					get_jump_input,
					get_crouch_input,
					get_attack_input,
					get_interact_input,
				)
					.in_set(InputSet::Get),),
			);
	}
}

fn get_walk_input(mut walk_input: ResMut<WalkInput>, gamepads: Query<&Gamepad>) {
	for gamepad in &gamepads {
		if let (Some(x), Some(y)) = (
			gamepad.get(GamepadAxis::LeftStickX),
			gamepad.get(GamepadAxis::LeftStickY),
		) {
			walk_input.vec += Vec2::new(x, y);
			walk_input.vec = walk_input.vec.clamp_length_max(1.);
		}
	}
}

fn get_rotate_input(
	mut rotate_input: ResMut<RotateInput>,
	controls: Res<Controls>,
	gamepads: Query<&Gamepad>,
) {
	for gamepad in &gamepads {
		if let (Some(x), Some(y)) = (
			gamepad.get(GamepadAxis::RightStickX),
			gamepad.get(GamepadAxis::RightStickX),
		) {
			rotate_input.pitch += y * controls.rotate_sensivity;

			// why does this have to be negated???
			// and is it specific to nintendo switch controllers?
			rotate_input.yaw -= x * controls.rotate_sensivity;
		}
	}
}

macro_rules! single_button {
	($fn_name:ident, $input_res:ident, $control:ident) => {
		fn $fn_name(
			mut input_res: ResMut<$input_res>,
			controls: Res<Controls>,
			gamepads: Query<&Gamepad>,
		) {
			for gamepad in &gamepads {
				// dont directly set these values so that it works with multiple
				// devices at the same time. these values are cleaned up in `InputSet::CleanUp`
				if gamepad.just_pressed(controls.$control) {
					input_res.started = true;
				}
				if gamepad.pressed(controls.$control) {
					input_res.holding = true;
				}
			}
		}
	};
}

single_button! { get_jump_input, JumpInput, jump }
single_button! { get_crouch_input, CrouchInput, crouch }
single_button! { get_attack_input, AttackInput, attack }
single_button! { get_interact_input, InteractInput, interact }

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Controls {
	pub jump: GamepadButton,
	pub crouch: GamepadButton,
	pub attack: GamepadButton,
	pub interact: GamepadButton,
	pub rotate_sensivity: f32,
}

impl Default for Controls {
	fn default() -> Self {
		Self {
			jump: GamepadButton::South,
			crouch: GamepadButton::LeftTrigger,
			attack: GamepadButton::LeftTrigger2,
			interact: GamepadButton::RightTrigger2,
			rotate_sensivity: 0.1,
		}
	}
}

impl Controls {
	pub fn load() -> Result<Self, Box<dyn Error>> {
		let path = savedata::get_savedata_path().join("controls/gamepad.ron");
		let string = fs::read_to_string(path)?;
		let controls = ron::from_str(&string)?;
		Ok(controls)
	}
}

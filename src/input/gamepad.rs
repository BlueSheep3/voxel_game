//! input handling for gamepad (controller / joystick)

use super::{AttackInput, CrouchInput, InputSet, InteractInput, JumpInput, RotateInput, WalkInput};
use crate::savedata;
use bevy::{
	input::gamepad::{GamepadConnection, GamepadEvent},
	prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Controls::load().unwrap_or_default())
			.add_systems(
				Update,
				(
					connect_and_disconnect,
					(
						get_walk_input,
						get_rotate_input,
						get_jump_input,
						get_crouch_input,
						get_attack_input,
						get_interact_input,
					)
						.in_set(InputSet::Get),
				),
			);
	}
}

#[derive(Resource, Debug, Copy, Clone, PartialEq, Eq)]
struct CurrentGamepad(Gamepad);

fn connect_and_disconnect(
	mut commands: Commands,
	current_gamepad: Option<Res<CurrentGamepad>>,
	mut gamepad_events: EventReader<GamepadEvent>,
) {
	for event in gamepad_events.read() {
		if let GamepadEvent::Connection(connection) = event {
			let gp = connection.gamepad;
			match &connection.connection {
				GamepadConnection::Connected(_) => {
					if current_gamepad.is_none() {
						commands.insert_resource(CurrentGamepad(gp));
					}
				}
				GamepadConnection::Disconnected => {
					if let Some(CurrentGamepad(current)) = current_gamepad.as_deref() {
						if current == &gp {
							commands.remove_resource::<CurrentGamepad>();
						}
					}
				}
			}
		}
	}
}

fn get_walk_input(
	input: Res<Axis<GamepadAxis>>,
	mut walk_input: ResMut<WalkInput>,
	current_gamepad: Option<Res<CurrentGamepad>>,
) {
	let Some(gamepad) = current_gamepad else {
		return;
	};

	let axis_lx = GamepadAxis {
		gamepad: gamepad.0,
		axis_type: GamepadAxisType::LeftStickX,
	};
	let axis_ly = GamepadAxis {
		gamepad: gamepad.0,
		axis_type: GamepadAxisType::LeftStickY,
	};

	if let (Some(x), Some(y)) = (input.get(axis_lx), input.get(axis_ly)) {
		walk_input.vec += Vec2::new(x, y);
		walk_input.vec = walk_input.vec.clamp_length_max(1.);
	}
}

fn get_rotate_input(
	input: Res<Axis<GamepadAxis>>,
	mut rotate_input: ResMut<RotateInput>,
	controls: Res<Controls>,
	current_gamepad: Option<Res<CurrentGamepad>>,
) {
	let Some(gamepad) = current_gamepad else {
		return;
	};

	let axis_rx = GamepadAxis {
		gamepad: gamepad.0,
		axis_type: GamepadAxisType::RightStickX,
	};
	let axis_ry = GamepadAxis {
		gamepad: gamepad.0,
		axis_type: GamepadAxisType::RightStickY,
	};

	if let (Some(x), Some(y)) = (input.get(axis_rx), input.get(axis_ry)) {
		rotate_input.pitch += y * controls.rotate_sensivity;

		// why does this have to be negated???
		// and is it specific to nintendo switch controllers?
		rotate_input.yaw -= x * controls.rotate_sensivity;
	}
}

macro_rules! single_button {
	($fn_name:ident, $input_res:ident, $control:ident) => {
		fn $fn_name(
			input: Res<ButtonInput<GamepadButton>>,
			mut input_res: ResMut<$input_res>,
			controls: Res<Controls>,
			current_gamepad: Option<Res<CurrentGamepad>>,
		) {
			let Some(gamepad) = current_gamepad else {
				return;
			};

			let button = GamepadButton {
				gamepad: gamepad.0,
				button_type: controls.$control,
			};

			// dont directly set these values so that it works with multiple
			// devices at the same time. these values are cleaned up in `InputSet::CleanUp`
			if input.just_pressed(button) {
				input_res.started = true;
			}
			if input.pressed(button) {
				input_res.holding = true;
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
	pub jump: GamepadButtonType,
	pub crouch: GamepadButtonType,
	pub attack: GamepadButtonType,
	pub interact: GamepadButtonType,
	pub rotate_sensivity: f32,
}

impl Default for Controls {
	fn default() -> Self {
		Self {
			jump: GamepadButtonType::South,
			crouch: GamepadButtonType::LeftTrigger,
			attack: GamepadButtonType::LeftTrigger2,
			interact: GamepadButtonType::RightTrigger2,
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

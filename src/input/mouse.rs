//! input handling for mouse

use super::{AttackInput, InputSet, InteractInput, RotateInput, ScrollInput};
use crate::savedata;
use bevy::{
	input::mouse::{MouseMotion, MouseWheel},
	prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

pub struct MousePlugin;

impl Plugin for MousePlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Controls::load().unwrap_or_default())
			.add_systems(
				Update,
				(
					get_rotate_input,
					get_scroll_input,
					get_attack_input,
					get_interact_input,
				)
					.in_set(InputSet::Get),
			);
	}
}

fn get_rotate_input(
	mut rotate_input: ResMut<RotateInput>,
	mut mouse_motions: EventReader<MouseMotion>,
	controls: Res<Controls>,
) {
	for motion in mouse_motions.read() {
		// this delta has unit: pixels
		let delta = motion.delta * controls.sensitivity;
		// not sure why but these have to be negated
		rotate_input.pitch -= delta.y;
		rotate_input.yaw -= delta.x;
	}
}

fn get_scroll_input(
	mut scroll_input: ResMut<ScrollInput>,
	mut mouse_scroll_events: EventReader<MouseWheel>,
) {
	// FIXME this currently breaks scrolling in bevy_inspector_egui
	// currently ignores the scroll unit
	for event in mouse_scroll_events.read() {
		scroll_input.delta += event.y;
	}
}

macro_rules! single_button {
	($fn_name:ident, $input_res:ident, $control:ident) => {
		fn $fn_name(
			input: Res<ButtonInput<MouseButton>>,
			mut input_res: ResMut<$input_res>,
			controls: Res<Controls>,
		) {
			// dont directly set these values so that it works with multiple
			// devices at the same time. these values are cleaned up in `InputSet::CleanUp`
			if input.just_pressed(controls.$control) {
				input_res.started = true;
			}
			if input.pressed(controls.$control) {
				input_res.holding = true;
			}
		}
	};
}

single_button! { get_attack_input, AttackInput, attack }
single_button! { get_interact_input, InteractInput, interact }

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Controls {
	/// How many radians to move per pixel that the mouse moves.
	/// This leads to the sensivity usually having a very small value.
	pub sensitivity: f32,
	pub attack: MouseButton,
	pub interact: MouseButton,
}

impl Default for Controls {
	fn default() -> Self {
		Self {
			sensitivity: 0.003,
			attack: MouseButton::Left,
			interact: MouseButton::Right,
		}
	}
}

impl Controls {
	fn load() -> Result<Self, Box<dyn Error>> {
		let path = savedata::get_savedata_path().join("controls/mouse.ron");
		let string = fs::read_to_string(path)?;
		let controls = ron::from_str(&string)?;
		Ok(controls)
	}
}

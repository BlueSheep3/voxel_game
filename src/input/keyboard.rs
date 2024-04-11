//! input handling for keyboard

use super::{CrouchInput, InputSet, JumpInput, WalkInput};
use crate::savedata;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Controls::load().unwrap_or_default())
			.add_systems(
				Update,
				(get_walk_input, get_jump_input, get_crouch_input).in_set(InputSet::Get),
			);
	}
}

fn get_walk_input(
	input: Res<ButtonInput<KeyCode>>,
	mut walk_input: ResMut<WalkInput>,
	controls: Res<Controls>,
) {
	let forward = input.pressed(controls.forward);
	let back = input.pressed(controls.back);
	let right = input.pressed(controls.right);
	let left = input.pressed(controls.left);

	let move_dir = Vec2::new(
		(right as i32 - left as i32) as f32,
		(forward as i32 - back as i32) as f32,
	);
	walk_input.vec += move_dir;
	walk_input.vec = walk_input.vec.clamp_length_max(1.);
}

macro_rules! single_button {
	($fn_name:ident, $input_res:ident, $control:ident) => {
		fn $fn_name(
			input: Res<ButtonInput<KeyCode>>,
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

single_button! { get_jump_input, JumpInput, jump }
single_button! { get_crouch_input, CrouchInput, crouch }
// single_button! { get_attack_input, AttackInput, attack }
// single_button! { get_interact_input, InteractInputc, interact }

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Controls {
	pub forward: KeyCode,
	pub back: KeyCode,
	pub right: KeyCode,
	pub left: KeyCode,
	pub jump: KeyCode,
	pub crouch: KeyCode,
	// pub attack: KeyCode,
	// pub interact: KeyCode,
}

impl Default for Controls {
	fn default() -> Self {
		Self {
			forward: KeyCode::KeyW,
			back: KeyCode::KeyS,
			right: KeyCode::KeyD,
			left: KeyCode::KeyA,
			jump: KeyCode::Space,
			crouch: KeyCode::ShiftLeft,
			// attack: KeyCode::???,
			// interact: KeyCode::???,
		}
	}
}

impl Controls {
	pub fn load() -> Result<Self, Box<dyn Error>> {
		let path = savedata::get_savedata_path().join("controls/keyboard.ron");
		let string = fs::read_to_string(path)?;
		let controls = ron::from_str(&string)?;
		Ok(controls)
	}
}

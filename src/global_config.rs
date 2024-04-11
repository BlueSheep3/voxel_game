use crate::savedata;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};

pub struct GlobalConfigPlugin;

impl Plugin for GlobalConfigPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Config::load().unwrap_or_default());
	}
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	/// the radius of how many chunks to load around the player horizontally
	pub horizontal_render_distance: u32,
	/// the radius of how many chunks to load around the player vertically
	pub vertical_render_distance: u32,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			horizontal_render_distance: 3,
			vertical_render_distance: 2,
		}
	}
}

impl Config {
	pub fn load() -> Result<Self, Box<dyn Error>> {
		let path = savedata::get_savedata_path().join("config.ron");
		let string = fs::read_to_string(path)?;
		let controls = ron::from_str(&string)?;
		Ok(controls)
	}
}

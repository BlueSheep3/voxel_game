use crate::savedata;
use bevy::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use serde::{Deserialize, Serialize};
use std::{error::Error, f32::consts::TAU, fs};

pub struct GlobalConfigPlugin;

impl Plugin for GlobalConfigPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(Config::load().unwrap_or_default())
			.register_type::<Config>()
			.add_plugins(FramepacePlugin)
			.add_systems(Update, update_fps_limit);
	}
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct Config {
	/// the radius of how many chunks to load around the player horizontally
	pub horizontal_render_distance: u32,
	/// the radius of how many chunks to load around the player vertically
	pub vertical_render_distance: u32,
	/// what to limit the fps to
	pub fps_limit: Option<f64>,
	/// the field of view of the player camera
	pub fov: f32,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			horizontal_render_distance: 3,
			vertical_render_distance: 2,
			fps_limit: Some(60.),
			fov: TAU / 8.,
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

fn update_fps_limit(config: Res<Config>, mut framepace: ResMut<FramepaceSettings>) {
	if config.is_changed() {
		if let Some(fps_limit) = config.fps_limit {
			framepace.limiter = Limiter::from_framerate(fps_limit);
		} else {
			framepace.limiter = Limiter::Auto;
		}
	}
}

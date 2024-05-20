//! A Plugin that makes every mesh get drawn as a Wireframe.

use bevy::{
	pbr::wireframe::{WireframeConfig, WireframePlugin},
	prelude::*,
};

/// A Plugin that makes every mesh get drawn as a Wireframe.
pub struct WireframeRenderingPlugin;

impl Plugin for WireframeRenderingPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(WireframePlugin)
			.insert_resource(WireframeConfig {
				// This can be changed via bevy inspector egui to render wireframes.
				global: false,
				default_color: Color::WHITE,
			});
	}
}

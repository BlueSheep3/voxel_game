#![doc = include_str!("../README.md")]
#![allow(clippy::needless_pass_by_value)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::missing_safety_doc)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![warn(clippy::collection_is_never_read)]
#![warn(clippy::use_self)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::suspicious_operation_groupings)]
#![warn(clippy::wildcard_imports)]
#![warn(clippy::enum_glob_use)]
#![warn(clippy::infinite_loop)]
#![warn(clippy::suspicious_to_owned)]

#[cfg(all(not(debug_assertions), feature = "dynamic_linking"))]
compile_error!("can't compile with dynamic linking in release mode");

#[cfg(all(debug_assertions, not(feature = "dynamic_linking")))]
compile_error!(
	"you should enable dynamic linking when compiling in debug mode for faster compile times"
);

mod block;
mod block_model;
mod cuboid;
mod debug;
mod debug_info;
mod display;
mod entity;
mod face;
mod game_world;
mod global_config;
mod input;
mod macros;
mod main_menu;
mod pos;
mod savedata;
mod ui;

use self::game_world::LeaveWorldEvent;
use bevy::{
	app::AppExit, input::common_conditions::input_toggle_active, prelude::*, window::PresentMode,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() -> AppExit {
	App::new()
		.add_plugins((
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: "Voxel Game".to_owned(),
						present_mode: PresentMode::Immediate,
						..default()
					}),
					..default()
				})
				.set(ImagePlugin::default_nearest()),
			WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
			block_model::BlockModelPlugin,
			game_world::GameWorldPlugin,
			input::InputPlugin,
			entity::EntityPlugin,
			ui::UiPlugin,
			debug_info::DebugInfoPlugin,
			debug::DebugPlugin,
			main_menu::MainMenuPlugin,
			global_config::GlobalConfigPlugin,
		))
		.init_state::<GlobalState>()
		.add_systems(
			Update,
			(
				close_on_q,
				leave_world_on_p,
				finish_loading.run_if(in_state(GlobalState::Loading)),
			),
		)
		.run()
}

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalState {
	#[default]
	Loading,
	MainMenu,
	InWorld,
}

fn finish_loading(
	mut global_state: ResMut<NextState<GlobalState>>,
	block_model_state: Res<State<block_model::LoadingState>>,
	// TODO more states
) {
	if *block_model_state == block_model::LoadingState::Done {
		global_state.set(GlobalState::MainMenu);
		info!("finished loading!");
	}
}

fn close_on_q(mut quit_event: EventWriter<AppExit>, keyboard_input: Res<ButtonInput<KeyCode>>) {
	if keyboard_input.just_pressed(KeyCode::KeyQ) {
		quit_event.send(AppExit::Success);
	}
}

fn leave_world_on_p(
	mut leave_event: EventWriter<LeaveWorldEvent>,
	keyboard_input: Res<ButtonInput<KeyCode>>,
) {
	if keyboard_input.just_pressed(KeyCode::KeyP) {
		leave_event.send(LeaveWorldEvent);
	}
}

mod first_person_cam;
mod free_cam;

use crate::GlobalState;
use bevy::{prelude::*, window::CursorGrabMode};

pub struct CamPlugin;

impl Plugin for CamPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			first_person_cam::FirstPersonCamPlugin,
			free_cam::FreeCamPlugin,
		))
		// TODO show cursor when opening inventory and other things
		.add_systems(OnEnter(GlobalState::InWorld), init)
		.add_systems(OnEnter(CanRotateCam(true)), hide_cursor)
		.add_systems(OnExit(CanRotateCam(true)), show_cursor)
		.add_systems(
			Update,
			(toggle_free_cam, toggle_can_rotate).run_if(in_state(GlobalState::InWorld)),
		)
		.add_sub_state::<PlayerCamMode>()
		.add_sub_state::<CanRotateCam>();
	}
}

#[derive(SubStates, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[source(GlobalState = GlobalState::InWorld)]
pub enum PlayerCamMode {
	#[default]
	FirstPerson,
	// TODO add ThirdPerson cam mode
	// ThirdPerson,
	FreeCam,
}

#[derive(SubStates, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[source(GlobalState = GlobalState::InWorld)]
pub struct CanRotateCam(pub bool);

// FIXME for some reason, the player camera moves faster during lag

/// the camera the player currently sees through<br>
/// this does not mean that it has to be close to the player; it can be a free cam<br>
/// exactly one `PlayerCam` should exist while in [`GlobalState::InWorld`]
#[derive(Component, Debug, Default)]
pub(super) struct PlayerCam;

fn init(
	mut cam_mode: ResMut<NextState<PlayerCamMode>>,
	mut can_rotate: ResMut<NextState<CanRotateCam>>,
) {
	cam_mode.set(PlayerCamMode::default());
	can_rotate.set(CanRotateCam(true));
}

fn toggle_free_cam(
	input: Res<ButtonInput<KeyCode>>,
	current_cam_state: Res<State<PlayerCamMode>>,
	mut next_cam_state: ResMut<NextState<PlayerCamMode>>,
) {
	if input.just_pressed(KeyCode::KeyF) {
		next_cam_state.set(match current_cam_state.get() {
			PlayerCamMode::FirstPerson => PlayerCamMode::FreeCam,
			PlayerCamMode::FreeCam => PlayerCamMode::FirstPerson,
		});
	}
}

fn toggle_can_rotate(
	input: Res<ButtonInput<KeyCode>>,
	current_can_rotate_state: Res<State<CanRotateCam>>,
	mut next_can_rotate_state: ResMut<NextState<CanRotateCam>>,
) {
	if input.just_pressed(KeyCode::KeyE) {
		let state = current_can_rotate_state.get().0;
		next_can_rotate_state.set(CanRotateCam(!state));
	}
}

const SHOULD_HIDE_CURSOR: bool = true;

fn hide_cursor(mut windows: Query<&mut Window>) {
	if SHOULD_HIDE_CURSOR {
		for mut window in &mut windows {
			window.cursor_options.visible = false;
			window.cursor_options.grab_mode = CursorGrabMode::Locked;
		}
	}
}

fn show_cursor(mut windows: Query<&mut Window>) {
	if SHOULD_HIDE_CURSOR {
		for mut window in &mut windows {
			window.cursor_options.visible = true;
			window.cursor_options.grab_mode = CursorGrabMode::None;
		}
	}
}

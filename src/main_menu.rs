use crate::{
	game_world::{JoinWorldEvent, NewWorldEvent},
	GlobalState,
};
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GlobalState::MainMenu), spawn)
			.add_systems(OnExit(GlobalState::MainMenu), despawn)
			.add_systems(
				Update,
				(click_new_world_button, click_start_button)
					.run_if(in_state(GlobalState::MainMenu)),
			);
	}
}

#[derive(Component)]
struct MainMenuRoot;

#[derive(Component)]
struct MainMenuCamera;

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct NewWorldButton;

fn spawn(mut commands: Commands) {
	commands.spawn((MainMenuCamera, Camera3d::default()));

	commands
		.spawn((
			MainMenuRoot,
			Node {
				width: Val::Percent(100.),
				height: Val::Percent(100.),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				// TODO add background
				..default()
			},
		))
		.with_children(|parent| {
			parent
				.spawn((
					NewWorldButton,
					Button,
					Node {
						width: Val::VMin(20.),
						height: Val::VMin(10.),
						margin: UiRect::all(Val::VMin(1.)),
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
						..default()
					},
					BackgroundColor::from(Color::srgb(0.15, 0.15, 0.15)),
				))
				.with_children(|parent| {
					parent.spawn((
						Text::new("New World"),
						TextColor::from(Color::WHITE),
						TextFont::from_font_size(20.),
					));
				});
			parent
				.spawn((
					StartButton,
					Button,
					Node {
						width: Val::VMin(20.),
						height: Val::VMin(10.),
						margin: UiRect::all(Val::VMin(1.)),
						justify_content: JustifyContent::Center,
						align_items: AlignItems::Center,
						..default()
					},
					BackgroundColor::from(Color::srgb(0.15, 0.15, 0.15)),
				))
				.with_children(|parent| {
					parent.spawn((
						Text::new("Play"),
						TextColor::from(Color::WHITE),
						TextFont::from_font_size(20.),
					));
				});
		});
}

fn despawn(
	mut commands: Commands,
	query: Query<Entity, With<MainMenuRoot>>,
	cams: Query<Entity, With<MainMenuCamera>>,
) {
	for entity in query.iter() {
		commands.entity(entity).despawn_recursive();
	}
	for cam in cams.iter() {
		commands.entity(cam).despawn();
	}
}

fn click_new_world_button(
	mut join_event: EventWriter<NewWorldEvent>,
	intercation_query: Query<&Interaction, (Changed<Interaction>, With<NewWorldButton>)>,
) {
	for interaction in intercation_query.iter() {
		if interaction == &Interaction::Pressed {
			join_event.send(NewWorldEvent);
		}
	}
}

fn click_start_button(
	mut join_event: EventWriter<JoinWorldEvent>,
	intercation_query: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
) {
	for interaction in intercation_query.iter() {
		if interaction == &Interaction::Pressed {
			join_event.send(JoinWorldEvent);
		}
	}
}

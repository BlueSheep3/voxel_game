use crate::GlobalState;
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GlobalState::MainMenu), spawn)
			.add_systems(OnExit(GlobalState::MainMenu), despawn)
			.add_systems(
				Update,
				click_start_button.run_if(in_state(GlobalState::MainMenu)),
			);
	}
}

#[derive(Component)]
struct MainMenuRoot;

#[derive(Component)]
struct MainMenuCamera;

#[derive(Component)]
struct StartButton;

fn spawn(mut commands: Commands) {
	commands.spawn((MainMenuCamera, Camera3dBundle::default()));

	commands
		.spawn((
			MainMenuRoot,
			NodeBundle {
				style: Style {
					width: Val::Percent(100.),
					height: Val::Percent(100.),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					..default()
				},
				// TODO add background
				..default()
			},
		))
		.with_children(|parent| {
			parent
				.spawn((
					StartButton,
					ButtonBundle {
						style: Style {
							width: Val::VMin(20.),
							height: Val::VMin(10.),
							justify_content: JustifyContent::Center,
							align_items: AlignItems::Center,
							..default()
						},
						background_color: Color::srgb(0.15, 0.15, 0.15).into(),
						..default()
					},
				))
				.with_children(|parent| {
					parent.spawn(TextBundle {
						text: Text::from_section(
							"Play",
							TextStyle {
								color: Color::WHITE,
								font_size: 20.,
								..default()
							},
						),
						..default()
					});
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

fn click_start_button(
	mut global_state: ResMut<NextState<GlobalState>>,
	intercation_query: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
) {
	for interaction in intercation_query.iter() {
		if interaction == &Interaction::Pressed {
			global_state.set(GlobalState::InWorld);
			info!("started game");
		}
	}
}

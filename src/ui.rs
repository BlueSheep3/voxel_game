use crate::GlobalState;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GlobalState::InWorld), spawn)
			.add_systems(OnExit(GlobalState::InWorld), despawn);
	}
}

#[derive(Component)]
struct UiRoot;

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
	let crosshair = asset_server.load("sprite/crosshair.png");

	commands
		.spawn((
			UiRoot,
			NodeBundle {
				style: Style {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					..default()
				},
				..default()
			},
		))
		.with_children(|parent| {
			parent.spawn((
				NodeBundle {
					style: Style {
						width: Val::VMin(5.0),
						height: Val::VMin(5.0),
						..default()
					},
					background_color: Color::WHITE.into(),
					..default()
				},
				UiImage::new(crosshair),
			));
		});
}

fn despawn(mut commands: Commands, root: Query<Entity, With<UiRoot>>) {
	let root = root.single();
	commands.entity(root).despawn_recursive();
}

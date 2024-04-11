use crate::game_world::GameWorld;
use bevy::prelude::info;
use std::{env, error::Error, fs, path::PathBuf};

pub fn save_game_world(world_name: &str, game_world: &GameWorld) -> Result<(), Box<dyn Error>> {
	info!("Saving game world {}...", world_name);
	let binary = bincode::serialize(game_world)?;
	let path = get_savedata_path().join("worlds");
	fs::create_dir_all(&path)?;
	// TODO split up the world into multiple files
	fs::write(path.join(format!("{}.bin", world_name)), binary)?;
	info!("Saved game world {}", world_name);
	Ok(())
}

pub fn load_game_world(world_name: &str) -> Result<GameWorld, Box<dyn Error>> {
	info!("Loading game world {}...", world_name);
	let path = get_savedata_path().join(format!("worlds/{}.bin", world_name));
	let binary = fs::read(path)?;
	let game_world = bincode::deserialize(&binary)?;
	info!("Loaded game world {}", world_name);
	Ok(game_world)
}

pub fn get_savedata_path() -> PathBuf {
	// FIXME %APPDATA% will only work on Windows
	#[cfg(not(target_os = "windows"))]
	compile_error!("get_savedata_path() only works on windows");

	let path = env::var("APPDATA").expect("APPDATA not found");
	let path = PathBuf::from(path);
	let path = path.parent().unwrap();
	path.join("LocalLow/BlueSheep3/Voxel Game")
}

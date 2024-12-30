use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fmt::{self, Display},
	fs,
	path::Path,
	sync::{LazyLock, Mutex},
	time::Duration,
};

pub struct BenchPlugin;

impl Plugin for BenchPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, save_benches_to_file);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BenchName {
	CreateChunkMesh,
	SpawnThread,
}

impl Display for BenchName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::CreateChunkMesh => "create_chunk_mesh",
			Self::SpawnThread => "spawn_thread",
		};
		write!(f, "{}", name)
	}
}

static BENCHES: LazyLock<Mutex<HashMap<BenchName, Vec<Duration>>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn push_time(name: BenchName, duration: Duration) {
	BENCHES
		.lock()
		.unwrap()
		.entry(name)
		.or_default()
		.push(duration);
}

fn save_benches_to_file(input: Res<ButtonInput<KeyCode>>) {
	if input.just_pressed(KeyCode::KeyK) {
		let base_path = crate::savedata::get_savedata_path();

		for (name, times) in &*BENCHES.lock().unwrap() {
			save_stats(times, &name.to_string(), &base_path);
		}
	}
}

fn save_stats(durations: &[Duration], name: &str, base_path: &Path) {
	let path = base_path.join(format!("benches_{}_all.bin", name));
	let content = bincode::serialize(durations).unwrap();
	fs::write(path, content).unwrap();

	let mut config = ron::ser::PrettyConfig::new();
	config.indentor = "\t".to_owned();
	let stats = compute_stats_from_nanos(durations);

	let path = base_path.join(format!("benches_{}_stats.ron", name));
	let content = ron::ser::to_string_pretty(&stats, config).unwrap();
	fs::write(path, content).unwrap();
}

#[derive(Serialize, Deserialize)]
struct Stats {
	avg: f64,
	deviation: f64,
}

fn compute_stats_from_nanos(durations: &[Duration]) -> Stats {
	let avg = durations
		.iter()
		.map(|d| d.as_nanos() as f64 / durations.len() as f64)
		.sum();
	let deviation = durations
		.iter()
		.map(|d| f64::powi(d.as_nanos() as f64 - avg, 2) / durations.len() as f64)
		.sum::<f64>()
		.sqrt();
	Stats { avg, deviation }
}

// mod block_model_asset;

use crate::{block::BlockId, face::FaceMap, GlobalState};
use bevy::{asset::LoadState, prelude::*};
use serde::Deserialize;
use std::{collections::HashMap, fs};

pub struct BlockModelPlugin;

impl Plugin for BlockModelPlugin {
	fn build(&self, app: &mut App) {
		app.init_state::<LoadingState>()
			.insert_resource(BlockModelWithImages::default())
			.add_systems(OnEnter(GlobalState::Loading), load_images)
			.add_systems(
				Update,
				check_finished_loading_images
					.run_if(in_state(GlobalState::Loading))
					.run_if(in_state(LoadingState::LoadingImages)),
			)
			.add_systems(
				OnEnter(LoadingState::SettingGlobalTexture),
				setup_global_texture,
			);
	}
}

/// stores all textures in the game, so they can be used easily in a combined mesh
#[derive(Resource, Debug, Clone)]
pub struct GlobalTexture {
	pub layout: TextureAtlasLayout,
	pub image: Handle<Image>,
	pub mappings: HashMap<BlockId, BlockModel<Rect>>,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct BlockModelAsset<Side: TypePath + Send + Sync> {
	pub should_cull: bool,
	pub cuboids: Vec<BlockModelAssetCuboid<Side>>,
}

#[derive(TypePath, Debug, Clone, Deserialize)]
pub struct BlockModelAssetCuboid<Side> {
	min: Vec3,
	max: Vec3,
	/// the positions for each face, normalized to the global texture
	pub sides: FaceMap<Side>,
}

#[derive(Debug, Clone)]
pub struct BlockModel<Side> {
	pub should_cull: bool,
	pub cuboids: Vec<BlockModelCuboid<Side>>,
}

#[derive(Debug, Clone)]
pub struct BlockModelCuboid<Side> {
	pub min: Vec3,
	pub max: Vec3,
	/// the positions for each face, normalized to the global texture
	pub sides: FaceMap<Side>,
}

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingState {
	#[default]
	LoadingImages,
	SettingGlobalTexture,
	Done,
}

#[derive(Resource, Debug, Default)]
struct BlockModelWithImages {
	images: HashMap<BlockId, BlockModel<Handle<Image>>>,
}

fn load_images(
	asset_server: Res<AssetServer>,
	mut block_images: ResMut<BlockModelWithImages>,
	mut loading_state: ResMut<NextState<LoadingState>>,
) {
	loading_state.set(LoadingState::LoadingImages);

	let block_models = get_block_models();
	for (id, block_model) in block_models {
		// let images = block_model
		// 	.cuboids
		// 	.iter()
		// 	.flat_map(|cuboid| cuboid.sides.iter())
		// 	.map(|side| asset_server.load(format!("sprite/{}.png", side)))
		// 	.collect::<Vec<_>>();

		let model = BlockModel {
			should_cull: block_model.should_cull,
			cuboids: block_model
				.cuboids
				.iter()
				.map(|cuboid| BlockModelCuboid {
					min: cuboid.min,
					max: cuboid.max,
					sides: cuboid
						.sides
						.clone()
						.map(|side| asset_server.load(format!("sprite/{}.png", side))),
				})
				.collect::<Vec<_>>(),
		};
		block_images.images.insert(id, model);
	}
}

fn check_finished_loading_images(
	asset_server: Res<AssetServer>,
	block_images: Res<BlockModelWithImages>,
	mut loading_state: ResMut<NextState<LoadingState>>,
) {
	let image_is_loaded = |i| asset_server.load_state(i) == LoadState::Loaded;
	let all_loaded = block_images.images.iter().all(|(_, model)| {
		model
			.cuboids
			.iter()
			.all(|cuboid| cuboid.sides.iter().all(image_is_loaded))
	});
	if all_loaded {
		loading_state.set(LoadingState::SettingGlobalTexture);
	}
}

fn setup_global_texture(
	mut commands: Commands,
	block_images: Res<BlockModelWithImages>,
	mut images: ResMut<Assets<Image>>,
	mut loading_state: ResMut<NextState<LoadingState>>,
) {
	// make sure there isnt already a global texture
	// because this function is allowed to be called more than once
	// (when reloading texture packs or something)
	commands.remove_resource::<GlobalTexture>();

	let block_models = get_block_models();
	let mut block_model_mappings = HashMap::new();
	let mut texture_atlas_builder = TextureAtlasBuilder::default();
	let mut model_indeces = Vec::new();
	let mut used_paths: HashMap<String, usize> = HashMap::new();

	let mut index = 0;
	for (block_id, block_model) in block_models {
		let mut face_indeces = Vec::new();

		for (i, cuboid) in block_model.cuboids.iter().enumerate() {
			for (side, face) in cuboid.sides.iter_face() {
				if let Some(&i) = used_paths.get(side) {
					face_indeces.push(i);
					continue;
				}

				let model = block_images.images.get(&block_id).unwrap();
				let block_image = model.cuboids[i].sides.get(face);
				let image = images.get(block_image).unwrap();
				texture_atlas_builder.add_texture(Some(block_image.into()), image);
				used_paths.insert(side.clone(), index);

				face_indeces.push(index);
				index += 1;
			}
		}

		let cuboid_maps = face_indeces
			.chunks_exact(6)
			.map(|chunk| FaceMap::try_from(chunk.to_vec()).unwrap())
			.enumerate()
			.map(|(i, face_maps)| BlockModelCuboid {
				min: block_model.cuboids[i].min,
				max: block_model.cuboids[i].max,
				sides: face_maps,
			})
			.collect::<Vec<_>>();
		model_indeces.push((block_id, cuboid_maps, block_model.should_cull));
	}

	let (layout, image) = texture_atlas_builder.finish().unwrap();
	let image = images.add(image);

	for (id, cuboid_maps, should_cull) in model_indeces {
		let model = BlockModel {
			should_cull,
			cuboids: cuboid_maps
				.iter()
				.map(|block_model_cuboid| {
					let min = block_model_cuboid.min;
					let max = block_model_cuboid.max;
					let face_maps = block_model_cuboid.sides;

					let positions = face_maps.map(|i| {
						let Rect { min, max } = layout.textures[i];
						let size = layout.size;
						Rect {
							min: min / size.x,
							max: max / size.y,
						}
					});
					BlockModelCuboid {
						min,
						max,
						sides: positions,
					}
				})
				.collect(),
		};
		block_model_mappings.insert(id, model);
	}

	commands.insert_resource(GlobalTexture {
		layout,
		image,
		mappings: block_model_mappings,
	});

	info!("Finished setting up global texture");

	loading_state.set(LoadingState::Done);
}

// TODO load these with the actual asset server to allow for hot reloading
fn get_block_models() -> Vec<(BlockId, BlockModelAsset<String>)> {
	let mut block_models = Vec::new();

	let in_folder = fs::read_dir("assets/blockmodel").unwrap();
	for file in in_folder {
		let file = file.unwrap();
		if file.file_type().unwrap().is_file() {
			let contents = fs::read_to_string(file.path()).unwrap();
			let block_model = ron::from_str(&contents).unwrap();

			let file_path = file.path();
			let block_name = file_path.file_stem().unwrap().to_str().unwrap();
			let block_id = BlockId::from_debug_name(block_name).unwrap();

			block_models.push((block_id, block_model));
		}
	}

	block_models
}

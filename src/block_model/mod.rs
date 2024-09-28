// mod block_model_asset;
mod chunk_material;
mod wireframe_rendering;

use self::{chunk_material::ChunkMaterialPlugin, wireframe_rendering::WireframeRenderingPlugin};
use crate::{block::BlockId, face::FaceMap, GlobalState};
use bevy::{
	asset::LoadState,
	prelude::*,
	render::{
		render_asset::RenderAssetUsages,
		texture::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
	},
};
use image::{imageops, DynamicImage};
use serde::Deserialize;
use std::{collections::HashMap, ffi::OsString, fs};
use thiserror::Error;

pub use self::chunk_material::{ChunkMaterial, ATTRIBUTE_BASE_VOXEL_INDICES};

pub struct BlockModelPlugin;

impl Plugin for BlockModelPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((ChunkMaterialPlugin, WireframeRenderingPlugin))
			.init_state::<LoadingState>()
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
	pub image: Handle<Image>,
	pub mappings: HashMap<BlockId, BlockModel<usize>>,
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

	let block_models = get_block_models().unwrap();
	for (id, block_model) in block_models {
		let cuboids = block_model
			.cuboids
			.iter()
			.map(|cuboid| {
				let sides = cuboid
					.sides
					.clone()
					.map(|side| asset_server.load(format!("sprite/{}.png", side)));
				BlockModelCuboid {
					min: cuboid.min,
					max: cuboid.max,
					sides,
				}
			})
			.collect::<Vec<_>>();
		let model = BlockModel {
			should_cull: block_model.should_cull,
			cuboids,
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

	let (block_textures, mappings) = get_textures(&block_images, &images);
	let image = images_into_array_texture(block_textures).unwrap();
	let image = images.add(image);

	commands.insert_resource(GlobalTexture { image, mappings });

	info!("Finished setting up global texture");

	loading_state.set(LoadingState::Done);
}

// TODO use result instead of unwrap
fn get_textures(
	block_images: &BlockModelWithImages,
	images: &Assets<Image>,
) -> (Vec<Image>, HashMap<BlockId, BlockModel<usize>>) {
	let block_models = get_block_models().unwrap();
	let mut used_paths: HashMap<String, usize> = HashMap::new();
	let mut block_textures = Vec::new();
	let mut mappings = HashMap::new();

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
				block_textures.push(image.clone());
				used_paths.insert(side.clone(), index);

				face_indeces.push(index);
				index += 1;
			}
		}

		let model = faces_into_model_indices(&face_indeces, &block_model);
		mappings.insert(block_id, model);
	}
	(block_textures, mappings)
}

fn faces_into_model_indices(
	face_indeces: &[usize],
	block_model: &BlockModelAsset<String>,
) -> BlockModel<usize> {
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
	BlockModel {
		should_cull: block_model.should_cull,
		cuboids: cuboid_maps,
	}
}

// TODO load these with the actual asset server to allow for hot reloading
fn get_block_models() -> Result<Vec<(BlockId, BlockModelAsset<String>)>, GlobalTextureError> {
	let mut block_models = Vec::new();

	let in_folder = fs::read_dir("assets/blockmodel")?;
	for file in in_folder {
		let file = file?;
		if file.file_type()?.is_file() {
			let contents = fs::read_to_string(file.path())?;
			let block_model = ron::from_str(&contents)?;

			let file_path = file.path();
			let block_name = file_path
				.file_stem()
				.ok_or(GlobalTextureError::NoFileStem)?;
			let block_name = block_name
				.to_str()
				.ok_or(GlobalTextureError::InvalidUtf8(block_name.to_owned()))?;
			let block_id = BlockId::from_debug_name(block_name)
				.ok_or(GlobalTextureError::UknownBlockName(block_name.to_owned()))?;

			block_models.push((block_id, block_model));
		}
	}

	Ok(block_models)
}

fn images_into_array_texture(images: Vec<Image>) -> Result<Image, GlobalTextureError> {
	let size = images[0].size();
	let count = images.len() as u32;
	let mut dynamic = DynamicImage::new_rgba8(size.x, size.y * count);

	for (i, image) in images.into_iter().enumerate() {
		if image.size() != size {
			return Err(GlobalTextureError::ImageWrongSize(size, image.size()));
		}

		let image = image
			.try_into_dynamic()
			.map_err(|_| GlobalTextureError::IntoDynamicImage)?;
		let y = i as i64 * size.y as i64;
		imageops::overlay(&mut dynamic, &image, 0, y);
	}

	let rau = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
	let mut image = Image::from_dynamic(dynamic, true, rau);
	image.reinterpret_stacked_2d_as_array(count);
	image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
		address_mode_u: ImageAddressMode::Repeat,
		address_mode_v: ImageAddressMode::Repeat,
		..default()
	});

	Ok(image)
}

#[derive(Error, Debug)]
enum GlobalTextureError {
	#[error("{0}")]
	Io(#[from] std::io::Error),
	#[error("{0}")]
	Ron(#[from] ron::error::SpannedError),
	#[error("couldn't convert Image into DynamicImage")]
	IntoDynamicImage,
	#[error("image was wrong size: should be {0}, but got {1}")]
	ImageWrongSize(UVec2, UVec2),
	#[error("uknown block name: {0}")]
	UknownBlockName(String),
	#[error("couldn't get file stem")]
	NoFileStem,
	#[error("invalid utf8: {0:?}")]
	InvalidUtf8(OsString),
}

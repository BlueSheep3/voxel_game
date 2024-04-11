use super::BlockModelAsset;
use bevy::{
	asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
	prelude::*,
	utils::{thiserror, BoxedFuture},
};
use thiserror::Error;

struct BlockModelAssetPlugin;

impl Plugin for BlockModelAssetPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<BlockModelAsset>()
			.init_asset_loader::<BlockModelAssetLoader>();
	}
}

#[derive(Default)]
struct BlockModelAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BlockModelAssetLoaderError {
	/// An [IO](std::io) Error
	#[error("Could not load asset: {0}")]
	Io(#[from] std::io::Error),
	/// A [RON](ron) Error
	#[error("Could not parse RON: {0}")]
	RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for BlockModelAssetLoader {
	type Asset = BlockModelAsset;
	type Settings = ();
	type Error = BlockModelAssetLoaderError;
	fn load<'a>(
		&'a self,
		reader: &'a mut Reader,
		_settings: &'a (),
		_load_context: &'a mut LoadContext,
	) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
		Box::pin(async move {
			let mut bytes = Vec::new();
			reader.read_to_end(&mut bytes).await?;
			let custom_asset = ron::de::from_bytes::<Self::Asset>(&bytes)?;
			Ok(custom_asset)
		})
	}

	fn extensions(&self) -> &[&str] {
		&["block_model", "block_model.ron"]
	}
}

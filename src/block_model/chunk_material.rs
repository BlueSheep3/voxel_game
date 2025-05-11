//! this module is mostly copied from:
//! https://playspacefarer.com/voxel-array-textures/

use bevy::{
	pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline},
	prelude::*,
	render::{
		mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef},
		render_resource::{
			AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
			VertexFormat,
		},
	},
};

pub struct ChunkMaterialPlugin;

impl Plugin for ChunkMaterialPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(MaterialPlugin::<ExtendedChunkMaterial>::default())
			.register_asset_reflect::<ExtendedChunkMaterial>();
	}
}

pub const ATTRIBUTE_START_TEXTURE_INDEX: MeshVertexAttribute =
	MeshVertexAttribute::new("StartTextureIndex", 47834329472, VertexFormat::Uint32);
pub const ATTRIBUTE_RECT_WIDTH: MeshVertexAttribute =
	MeshVertexAttribute::new("RectWidth", 9836487236423, VertexFormat::Uint32);

pub type ExtendedChunkMaterial = ExtendedMaterial<StandardMaterial, ChunkMaterial>;

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect)]
pub struct ChunkMaterial {
	#[texture(100, dimension = "2d_array")]
	#[sampler(101)]
	pub block_textures: Handle<Image>,

	// this is a 1 dimensional dynamically sized array of u32s, that will be used
	// to index into `block_textures` to get the actual texture of the block.
	#[texture(102, dimension = "1d", sample_type = "u_int")]
	pub face_texture_indices: Handle<Image>,
}

impl MaterialExtension for ChunkMaterial {
	fn vertex_shader() -> ShaderRef {
		"shaders/chunk.wgsl".into()
	}

	fn fragment_shader() -> ShaderRef {
		"shaders/chunk.wgsl".into()
	}

	fn specialize(
		_pipeline: &MaterialExtensionPipeline,
		descriptor: &mut RenderPipelineDescriptor,
		layout: &MeshVertexBufferLayoutRef,
		_key: MaterialExtensionKey<Self>,
	) -> Result<(), SpecializedMeshPipelineError> {
		let vertex_layout = layout.0.get_layout(&[
			Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
			Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
			ATTRIBUTE_START_TEXTURE_INDEX.at_shader_location(7),
			ATTRIBUTE_RECT_WIDTH.at_shader_location(8),
		])?;
		descriptor.vertex.buffers = vec![vertex_layout];
		Ok(())
	}
}

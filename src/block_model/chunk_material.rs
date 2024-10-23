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
		app.add_plugins(MaterialPlugin::<
			ExtendedMaterial<StandardMaterial, ChunkMaterial>,
		>::default())
			.register_asset_reflect::<ExtendedMaterial<StandardMaterial, ChunkMaterial>>();
	}
}

pub const ATTRIBUTE_BASE_VOXEL_INDICES: MeshVertexAttribute =
	MeshVertexAttribute::new("BaseVoxelIndices", 47834329472, VertexFormat::Uint32);

#[derive(AsBindGroup, Debug, Clone, Asset, Reflect)]
pub struct ChunkMaterial {
	#[texture(100, dimension = "2d_array")]
	#[sampler(101)]
	pub texture: Handle<Image>,
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
			ATTRIBUTE_BASE_VOXEL_INDICES.at_shader_location(7),
		])?;
		descriptor.vertex.buffers = vec![vertex_layout];
		Ok(())
	}
}

//! this module is mostly copied from:
//! https://playspacefarer.com/voxel-array-textures/

use bevy::{
	pbr::{MaterialPipeline, MaterialPipelineKey},
	prelude::*,
	render::{
		mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
		render_resource::{
			AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
			VertexFormat,
		},
	},
};

pub struct ChunkMaterialPlugin;

impl Plugin for ChunkMaterialPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(MaterialPlugin::<ChunkMaterial>::default());
	}
}

pub const ATTRIBUTE_BASE_VOXEL_INDICES: MeshVertexAttribute =
	MeshVertexAttribute::new("BaseVoxelIndices", 47834329472, VertexFormat::Uint32);
// pub const ATTRIBUTE_OVERLAY_VOXEL_INDICES: MeshVertexAttribute =
// 	MeshVertexAttribute::new("OverlayVoxelIndices", 3249572836, VertexFormat::Uint32);

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct ChunkMaterial {
	#[texture(0, dimension = "2d_array")]
	#[sampler(1)]
	pub texture: Handle<Image>,
	// #[texture(2, dimension = "2d_array")]
	// #[sampler(3)]
	// pub pbr_texture: Handle<Image>,
}

impl Material for ChunkMaterial {
	fn vertex_shader() -> ShaderRef {
		"shaders/chunk.wgsl".into()
	}

	fn fragment_shader() -> ShaderRef {
		"shaders/chunk.wgsl".into()
	}

	fn alpha_mode(&self) -> AlphaMode {
		AlphaMode::Blend
	}

	fn specialize(
		_pipeline: &MaterialPipeline<Self>,
		descriptor: &mut RenderPipelineDescriptor,
		layout: &MeshVertexBufferLayout,
		_key: MaterialPipelineKey<Self>,
	) -> Result<(), SpecializedMeshPipelineError> {
		let vertex_layout = layout.get_layout(&[
			Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
			Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
			Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
			// Mesh::ATTRIBUTE_TANGENT.at_shader_location(3),
			// Mesh::ATTRIBUTE_COLOR.at_shader_location(4),
			// Mesh::ATTRIBUTE_JOINT_INDEX.at_shader_location(5),
			// Mesh::ATTRIBUTE_JOINT_WEIGHT.at_shader_location(6),
			ATTRIBUTE_BASE_VOXEL_INDICES.at_shader_location(7),
			// ATTRIBUTE_OVERLAY_VOXEL_INDICES.at_shader_location(8),
		])?;
		descriptor.vertex.buffers = vec![vertex_layout];
		Ok(())
	}
}

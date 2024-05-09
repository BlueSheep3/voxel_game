use bevy::{
	asset::LoadState,
	prelude::*,
	render::{
		mesh::{Indices, PrimitiveTopology},
		render_asset::RenderAssetUsages,
	},
};

use super::chunk_material::{ChunkMaterial, ATTRIBUTE_BASE_VOXEL_INDICES};

pub struct ArrayTextureTestPlugin;

impl Plugin for ArrayTextureTestPlugin {
	fn build(&self, app: &mut App) {
		app.init_state::<LoadingState>()
			.add_systems(Startup, start_loading_texture)
			.add_systems(Update, set_global_texture)
			.add_systems(OnEnter(LoadingState::Finished), spawn_mesh);
	}
}

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum LoadingState {
	#[default]
	Loading,
	Finished,
}

#[derive(Resource)]
pub struct GlobalArrayTexture {
	pub image: Handle<Image>,
}

#[derive(Resource)]
struct LoadingTexture {
	image: Handle<Image>,
}

#[derive(Component)]
struct TestMesh;

fn start_loading_texture(mut commands: Commands, asset_server: Res<AssetServer>) {
	let image = asset_server.load("sprite/StackedTest.png");
	commands.insert_resource(LoadingTexture { image });
}

fn set_global_texture(
	loading_texture: Res<LoadingTexture>,
	asset_server: Res<AssetServer>,
	mut images: ResMut<Assets<Image>>,
	loading_state: Res<State<LoadingState>>,
	mut next_loading_state: ResMut<NextState<LoadingState>>,
	mut commands: Commands,
) {
	if loading_state.get() == &LoadingState::Finished
		|| asset_server.load_state(loading_texture.image.clone()) != LoadState::Loaded
	{
		return;
	}
	let mut image = images.get(loading_texture.image.clone()).unwrap().clone();
	image.reinterpret_stacked_2d_as_array(3);
	let image = images.add(image);

	commands.insert_resource(GlobalArrayTexture { image });
	next_loading_state.set(LoadingState::Finished);
}

fn spawn_mesh(
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ChunkMaterial>>,
	mut commands: Commands,
	global_array_texture: Res<GlobalArrayTexture>,
	// asset_server: Res<AssetServer>,
) {
	let mesh = meshes.add(create_plane_mesh());
	let material = materials.add(ChunkMaterial {
		texture: global_array_texture.image.clone(),
		// pbr_texture: global_array_texture.image.clone(),
	});
	// let material = materials.add(StandardMaterial {
	// 	unlit: true,
	// 	base_color_texture: Some(asset_server.load("sprite/Dirt.png")),
	// 	..default()
	// });
	let transform = Transform::from_translation(Vec3::new(0., 10., 0.));

	commands.spawn((
		MaterialMeshBundle {
			mesh,
			material,
			transform,
			..default()
		},
		TestMesh,
		Name::new("TestMesh"),
	));
}

/// creates the mesh for a plane
fn create_plane_mesh() -> Mesh {
	let mut mesh = Mesh::new(
		PrimitiveTopology::TriangleList,
		RenderAssetUsages::default(),
	);

	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, get_mesh_positions());
	mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, get_mesh_uvs());
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, get_mesh_normals());
	mesh.insert_indices(Indices::U32(get_mesh_tris()));
	mesh.insert_attribute(ATTRIBUTE_BASE_VOXEL_INDICES, get_mesh_voxel_indices());

	mesh
}

fn get_mesh_positions() -> Vec<[f32; 3]> {
	vec![[0., 0., 0.], [5., 0., 0.], [0., 5., 0.], [5., 5., 0.]]
}

fn get_mesh_uvs() -> Vec<[f32; 2]> {
	vec![[0., 0.], [1., 0.], [0., 1.], [1., 1.]]
}

fn get_mesh_normals() -> Vec<[f32; 3]> {
	vec![[0., 0., 1.], [0., 0., 1.], [0., 0., 1.], [0., 0., 1.]]
}

fn get_mesh_tris() -> Vec<u32> {
	vec![0, 1, 2, 2, 3, 0]
}

fn get_mesh_voxel_indices() -> Vec<u32> {
	vec![0, 1, 2, 0]
}

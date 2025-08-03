use legion::Resources;
use std::sync::Arc;

const ASSETPATH: &str = "assets/";

pub struct AssetManager {
    loaders: Resources,

    // mesh_manager: Arc<MeshManager>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl AssetManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) /*->  Self */ {
        // let texture_manager = Arc::new(TextureManager::new(device.clone(), queue.clone()));
        // let shader_manager = Arc::new(ShaderManager::new(device.clone()));
        // let mut loaders = Resources::default();

        // let material_manager = Arc::new(MaterialManager::new(
        //     device.clone(),
        //     queue.clone(),
        //     texture_manager.clone(),

        // ));
        // let mesh_manager = Arc::new(MeshManager::new(device, material_manager));

        // loaders.insert(material_manager);
        // Self {
        //     loaders,
        //     texture_manager,
        //     shader_manager,
        //     mesh_manager,
        //     device,
        //     queue,

        // }
    }

}

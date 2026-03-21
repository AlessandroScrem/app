use super::*;

#[derive(Default)]
pub struct AssetManager {
    pub textures: TextureAssets,
    pub materials: MaterialAssets,
    pub meshes: MeshAssets,
    pub skybox: SkyboxHandle,
}

#[derive(Default)]
pub struct SkyboxHandle {
    cubemap: TextureId,
    intensity: f32,
}

impl SkyboxHandle {
    pub fn new(id: TextureId) -> Self {
        Self {
            cubemap: id,
            intensity: 1.0,
        }
    }

    #[allow(unused)]
    pub fn get_intensity(&self) -> f32 {
        self.intensity
    }
    #[allow(unused)]
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }
    pub fn get_id(&self) -> TextureId {
        self.cubemap
    }
    pub fn set_id(&mut self, id: TextureId) {
        self.cubemap = id;
    }
}

#[derive(Default, Debug, Clone)]
pub struct ResourceStats {
    pub count: usize,
    pub shared: usize,
    pub estimated_bytes: usize,
}

impl ResourceStats {
    pub fn add(&mut self, size: usize) {
        self.estimated_bytes += size;
        self.count += 1;
    }
    pub fn remove(&mut self, size: usize) {
        if self.count > 0 {
            let result = self.estimated_bytes.checked_sub(size).unwrap_or(0);
            self.estimated_bytes = result;
            self.count -= 1;
        }
    }

    pub fn add_shared(&mut self) {
        self.shared += 1;
    }
    pub fn remove_sahred(&mut self) {
        if self.shared > 0 {
            self.shared -= 1;
        }
    }
}

pub trait HasStats {
    fn get_stats(&self) -> ResourceStats;
}

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

    #[allow(dead_code)]
    pub fn get_intensity(&self) -> f32 {
        self.intensity
    }
    #[allow(dead_code)]
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity =  intensity;
    }
    pub fn get_id(&self) -> TextureId {
        self.cubemap
    }
    pub fn set_id(&mut self, id: TextureId) {
        self.cubemap =  id;
    }
}

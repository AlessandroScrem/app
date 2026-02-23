use super::*;

#[derive(Default)]
pub(crate) struct AssetManager {
    pub(crate) textures: TextureAssets,
    pub(crate) materials: MaterialAssets,
    pub(crate) meshes: MeshAssets,
    pub(crate) skybox: SkyboxHandle,
}

#[derive(Default)]
pub(crate) struct SkyboxHandle {
    cubemap: TextureId,
    intensity: f32,
}

impl SkyboxHandle {
    pub(crate) fn new(id: TextureId) -> Self {
        Self {
            cubemap: id,
            intensity: 1.0,
        }
    }

    pub(crate) fn get_intensity(&self) -> f32 {
        self.intensity
    }
    pub(crate) fn set_intensity(&mut self, intensity: f32) {
        self.intensity =  intensity;
    }
    pub(crate) fn get_id(&self) -> TextureId {
        self.cubemap
    }
    pub(crate) fn set_id(&mut self, id: TextureId) {
        self.cubemap =  id;
    }
}

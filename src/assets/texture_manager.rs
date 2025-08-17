use crate::assets::texture::Texture;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct TextureManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    textures: HashMap<PathBuf, Arc<Texture>>,
}

impl TextureManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let mut textures = HashMap::new();
        let lightbulb: PathBuf = "assets/core/lightbulb-icon32.png".into();
        let white: PathBuf = "assets/core/white.png".into();

        textures.insert(
            lightbulb.clone(),
            Arc::new(Texture::new(&device, &queue, Self::read(&lightbulb), false)),
        );

        textures.insert(
            white.clone(),
            Arc::new(Texture::new(&device, &queue, Self::read(&white), false)),
        );

        Self {
            device,
            queue,
            textures,
        }
    }

    pub fn load_texture(&mut self, filepath: PathBuf, is_normal: bool) -> Arc<Texture> {
        let filepath = {
            let candidate = filepath;
            candidate
                .is_file()
                .then(|| candidate)
                .unwrap_or_else(|| PathBuf::from("assets/core/white.png"))
        };

        if self.textures.contains_key(&filepath) {
            println!("Found: {:?}", filepath);
            return self.textures.get(&filepath).unwrap().clone();
        } else {
            println!("Add to texture: {:?}", filepath);
            let texture = Arc::new(Texture::new(
                &self.device,
                &self.queue,
                Self::read(&filepath),
                is_normal,
            ));
            self.textures.insert(filepath, texture.clone());

            return texture;
        }
    }

    fn read(filepath: &Path) -> Vec<u8> {
        std::fs::read(filepath).expect(&format!("Impossibile leggere il file {:?}", filepath))
    }
}

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
    white_texture: Arc<Texture>,
}

impl TextureManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let buffer = include_bytes!("../../assets/core/white.png");
        let white_texture = Texture::new(&device, &queue, buffer.to_vec(), false);

        Self {
            device,
            queue,
            textures: HashMap::new(),
            white_texture: Arc::new(white_texture),
        }
    }

    pub fn get_or_create(&mut self, filepath: &Path, is_normal: bool) -> Arc<Texture> {
        match self.textures.get(&filepath.to_path_buf()) {
            Some(texture) => texture.clone(),
            None => self.create_texture(filepath, is_normal),
        }
    }

    fn create_texture(&mut self, filepath: &Path, is_normal: bool) -> Arc<Texture> {
        match Self::read(filepath) {
            Some(buffer) => {
                let texture = Arc::new(Texture::new(&self.device, &self.queue, buffer, is_normal));
                println!("filepath {}", filepath.display());
                self.textures.insert(filepath.to_path_buf(), texture.clone());
                texture.clone()
            }
            None => self.white_texture.clone(),
        }
    }

    fn read(filepath: &Path) -> Option<Vec<u8>> {
        match std::fs::read(filepath) {
            Ok(buffer) => Some(buffer),
            Err(err) => {
                println!(
                    "{}, Impossibile leggere il file {}",
                    err,
                    filepath.display()
                );
                None
            }
        }
    }
}

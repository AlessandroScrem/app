use crate::assets::texture::Texture;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct TextureManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pub textures: HashMap<PathBuf, Arc<Texture>>,
    white_texture: Arc<Texture>,
}

impl TextureManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let buffer = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/core/white.png"
        ));
        let white_texture = Arc::new(Texture::new(&device, &queue, buffer, false));

        Self {
            device,
            queue,
            textures: HashMap::new(),
            white_texture,
        }
    }

    pub fn get_or_create(&mut self, filepath: &Path, is_normal: bool) -> Arc<Texture> {
        if self.textures.contains_key(filepath) {
            return self.textures.get(filepath).unwrap().clone();
        };

        self.create_texture(filepath, is_normal)
    }

    fn create_texture(&mut self, filepath: &Path, is_normal: bool) -> Arc<Texture> {
        match Self::read_bytes(filepath) {
            Some(buffer) => {
                let texture = Arc::new(Texture::new(&self.device, &self.queue, &buffer, is_normal));
                self.textures
                    .insert(filepath.to_path_buf(), texture.clone());

                texture
            }
            None => self.white_texture.clone(),
        }
    }

    fn read_bytes(filepath: &Path) -> Option<Vec<u8>> {
        match std::fs::read(filepath) {
            Ok(buffer) => {
                println!("read filepath {}", filepath.display());
                Some(buffer)
            }
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

use wgpu::TextureFormat;

use crate::assets::texture::{CubeTexture, Texture};
use crate::prelude::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct TextureManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    white_texture: Arc<Texture>,
    pub textures: HashMap<PathBuf, Arc<Texture>>,
}

impl TextureManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let buffer = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/core/white.png"
        ));
        let white_texture = Arc::new(Texture::new(
            &device,
            &queue,
            buffer,
            TextureFormat::Rgba8UnormSrgb,
        ));

        Self {
            device,
            queue,
            textures: HashMap::new(),
            white_texture,
        }
    }


    pub fn create_cubemap(
        &mut self,
        f0: &Path,
        f1: &Path,
        f2: &Path,
        f3: &Path,
        f4: &Path,
        f5: &Path,
        format: TextureFormat,
    ) -> Arc<CubeTexture> {
        let buffer0 = Self::read_bytes(f0).unwrap();
        let buffer1 = Self::read_bytes(f1).unwrap();
        let buffer2 = Self::read_bytes(f2).unwrap();
        let buffer3 = Self::read_bytes(f3).unwrap();
        let buffer4 = Self::read_bytes(f4).unwrap();
        let buffer5 = Self::read_bytes(f5).unwrap();

        // Slice di slice
        let buffers: [&[u8]; 6] = [&buffer0, &buffer1, &buffer2, &buffer3, &buffer4, &buffer5];
        let cubemap = CubeTexture::new(&self.device, &self.queue, &buffers, format);

        Arc::new(cubemap)
    }

    pub fn get_or_create(&mut self, filepath: &Path, format: TextureFormat) -> Arc<Texture> {
        let texture = match self.textures.get(filepath) {
            Some(texture) => texture.clone(),
            None => self.create_texture(filepath, format),
        };
        texture
    }

    // Aggiunge una texture
    fn create_texture(&mut self, filepath: &Path, format: TextureFormat) -> Arc<Texture> {
        match Self::read_bytes(filepath) {
            Some(buffer) => {
                let texture = Arc::new(Texture::new(&self.device, &self.queue, &buffer, format));
                self.textures
                    .insert(filepath.to_path_buf(), texture.clone());

                texture
            }
            None => self.white_texture.clone(),
        }
    }

    // Rimuove una texture e notifica
    // TODO: add texture removal
    #[allow(dead_code)]
    fn remove_texture(&mut self) {
    }

    fn read_bytes(filepath: &Path) -> Option<Vec<u8>> {
        match std::fs::read(filepath) {
            Ok(buffer) => {
                // println!("read filepath {} ", filepath.display());
                Some(buffer)
            }
            Err(err) => {
                info!(
                    "{}, Impossibile leggere il file {}",
                    err,
                    filepath.display()
                );
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils;

    fn create_manager() -> TextureManager {
        let (device, queue) = test_utils::get_device_and_queue();
        TextureManager::new(device.clone(), queue.clone())
    }

    #[test]
    fn should_create_texture_manager() {
        let manager = create_manager();

        assert!(manager.textures.is_empty());
    }

    #[test]
    fn should_load_cube_texture() {
        let mut manager = create_manager();

        #[rustfmt::skip] let f0 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/right.png"));
        #[rustfmt::skip] let f1 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/left.png"));
        #[rustfmt::skip] let f2 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/top.png"));
        #[rustfmt::skip] let f3 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/bottom.png"));
        #[rustfmt::skip] let f4 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/front.png"));
        #[rustfmt::skip] let f5 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/back.png"));

        let cube = manager.create_cubemap(f0, f1, f2, f3, f4, f5, TextureFormat::Rgba8UnormSrgb);

        assert_eq!(cube.extent.depth_or_array_layers, 6);
    }

    #[test]
    fn should_load_hdr_texture_rgba32float() {
        let mut manager = create_manager();

        #[rustfmt::skip] let f0 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/clarens_night_02_256.hdr"));

        let hdr = manager.get_or_create(f0, TextureFormat::Rgba32Float);

        assert_eq!(hdr.inner.format(), TextureFormat::Rgba32Float);
    }

    /// Hdr
    #[test]
    fn should_load_hdr_texture_rgba16float() {
        let mut manager = create_manager();

        #[rustfmt::skip] let f0 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/test/clarens_night_02_256.hdr"));

        let hdr = manager.get_or_create(f0, TextureFormat::Rgba16Float);

        assert_eq!(hdr.inner.format(), TextureFormat::Rgba16Float);
        assert!(hdr.inner.width() > 0);
        assert!(hdr.inner.height() > 0);
        assert_eq!(hdr.inner.mip_level_count(), 1); // <- no mipmaps
        assert_eq!(hdr.inner.depth_or_array_layers(), 1); // <- 2D texture
        assert_eq!(hdr.inner.dimension(), wgpu::TextureDimension::D2);

        #[cfg(feature = "save_tests")]
        {
            let (device, queue) = test_utils::get_device_and_queue();
            test_utils::save_texture(&device, &queue, "hdr.png", &hdr.inner, 0).unwrap();
        }
    }
}

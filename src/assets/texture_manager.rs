use wgpu::TextureFormat;

use crate::assets::texture::{CubeTexture, Texture};
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

    fn read_bytes(filepath: &Path) -> Option<Vec<u8>> {
        match std::fs::read(filepath) {
            Ok(buffer) => {
                println!("read filepath {} ", filepath.display());
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn should_load_cube_texture() {
        let (_adapter, device, queue) = pollster::block_on(async {
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::None,
                    compatible_surface: None,
                    ..Default::default()
                })
                .await
                .unwrap();

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor::default())
                .await
                .unwrap();

            let arc_device = Arc::new(device);
            let arc_queue = Arc::new(queue);

            (adapter, arc_device, arc_queue)
        });

        let mut texture_manager = TextureManager::new(device.clone(), queue.clone());

        #[rustfmt::skip] let f0 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/right.png"));
        #[rustfmt::skip] let f1 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/left.png"));
        #[rustfmt::skip] let f2 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/top.png"));
        #[rustfmt::skip] let f3 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/bottom.png"));
        #[rustfmt::skip] let f4 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/front.png"));
        #[rustfmt::skip] let f5 = Path::new(concat!(env!("CARGO_MANIFEST_DIR"),"/assets/skybox/back.png"));

        let cube =
            texture_manager.create_cubemap(f0, f1, f2, f3, f4, f5, TextureFormat::Rgba8UnormSrgb);

        assert_eq!(cube.extent.depth_or_array_layers, 6);
    }
}

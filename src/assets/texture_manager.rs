/* use wgpu::TextureFormat;

use crate::assets::texture::{CubeTexture, Texture};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
 */

// pub struct TextureManager {
//     device: Arc<wgpu::Device>,
//     queue: Arc<wgpu::Queue>,
//     white_texture: Arc<Texture>,
//     pub textures: HashMap<PathBuf, Arc<Texture>>,
// }

/* impl TextureManager {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
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
            device: Arc::new(device),
            queue: Arc::new(queue),
            textures: HashMap::new(),
            white_texture,
        }
    }

    pub fn create_cubemap<P: AsRef<Path>>(
        &mut self,
        path: [P; 6],
        format: TextureFormat,
    ) -> Arc<CubeTexture> {
        let buffer0 = Self::read_bytes(path[0].as_ref()).unwrap();
        let buffer1 = Self::read_bytes(path[1].as_ref()).unwrap();
        let buffer2 = Self::read_bytes(path[2].as_ref()).unwrap();
        let buffer3 = Self::read_bytes(path[3].as_ref()).unwrap();
        let buffer4 = Self::read_bytes(path[4].as_ref()).unwrap();
        let buffer5 = Self::read_bytes(path[5].as_ref()).unwrap();

        // Slice di slice
        let buffers: [&[u8]; 6] = [&buffer0, &buffer1, &buffer2, &buffer3, &buffer4, &buffer5];
        let cubemap = CubeTexture::new(&self.device, &self.queue, &buffers, format);

        Arc::new(cubemap)
    }

    pub fn get_texture<P: AsRef<Path>>(&self, filepath: P) -> Arc<Texture> {
        match self.textures.get(filepath.as_ref()) {
            Some(texture) => texture.clone(),
            None => self.white_texture.clone(),
        }
    }

    pub fn create_texture<P: AsRef<Path>>(
        &mut self,
        filepath: P,
        format: TextureFormat,
    ) -> Arc<Texture> {
        let texture = match self.textures.get(filepath.as_ref()) {
            Some(texture) => texture.clone(),
            None => self.create(filepath.as_ref(), format),
        };
        texture
    }

    // Aggiunge una texture
    fn create<P: AsRef<Path>>(&mut self, filepath: P, format: TextureFormat) -> Arc<Texture> {
        match Self::read_bytes(filepath.as_ref()) {
            Some(buffer) => {
                let texture = Arc::new(Texture::new(&self.device, &self.queue, &buffer, format));
                self.textures
                    .insert(filepath.as_ref().to_path_buf(), texture.clone());

                texture
            }
            None => self.white_texture.clone(),
        }
    }

    fn read_bytes<P: AsRef<Path>>(filepath: P) -> Option<Vec<u8>> {
        match std::fs::read(filepath) {
            Ok(buffer) => {
                // println!("read filepath {} ", filepath.display());
                Some(buffer)
            }
            Err(_err) => {
                // info!(
                //     "{}, Impossibile leggere il file {}",
                //     err,
                //     filepath.display()
                // );
                None
            }
        }
    }
} */
/* 
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;
    const HDR_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");

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

        let images = [
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/right.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/left.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/top.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/bottom.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/front.png"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test/back.png"),
        ];

        let cube = manager.create_cubemap(images, TextureFormat::Rgba8UnormSrgb);

        assert_eq!(cube.extent.depth_or_array_layers, 6);

        #[cfg(feature = "save_tests")]
        {
            let (device, queue) = test_utils::get_device_and_queue();
            test_utils::save_cubemap_cross(&device, &queue, "Skybox_result.png", &cube.inner)
                .unwrap();
        }
    }

    #[test]
    fn should_load_hdr_texture_rgba32float() {
        let mut manager = create_manager();

        let hdr = manager.create_texture(HDR_PATH, TextureFormat::Rgba32Float);

        assert_eq!(hdr.inner.format(), TextureFormat::Rgba32Float);
    }

    /// Hdr
    #[test]
    fn should_load_hdr_texture_rgba16float() {
        let mut manager = create_manager();

        let hdr = manager.create_texture(HDR_PATH, TextureFormat::Rgba16Float);

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
 */
// step for creating IBL environment
// CreateBRDFintegrationMap();
// CubemapFromHDR(skybox);
// CreateIrradiance(skybox);
// CreatePrefilterMap(skybox);

use super::ibl_impl::*;
use super::*;

pub struct Ibl {
    hdr_id: crate::assets::TextureId,
    _cube_map: wgpu::Texture,
    _cube_map_view: wgpu::TextureView,
    _irradiance_map: wgpu::Texture,
    _prefilter_map: wgpu::Texture,
    sampler: wgpu::Sampler,
    irradiance_view: wgpu::TextureView,
    prefilter_view: wgpu::TextureView,
    brdf_lut_view: wgpu::TextureView,
}

impl Ibl {
    pub fn get_bindgroup_entry<'a>(&'a self) -> Vec<wgpu::BindGroupEntry<'a>> {
        let Ibl {
            sampler,
            irradiance_view,
            prefilter_view,
            brdf_lut_view,
            ..
        } = self;

        vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(irradiance_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(prefilter_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(brdf_lut_view),
            },
        ]
    }

    pub fn get_skybox_bindgroup_entry<'a>(&'a self) -> Vec<wgpu::BindGroupEntry<'a>> {
        let Ibl {
            sampler,
            _cube_map_view,
            ..
        } = self;

        vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&_cube_map_view),
            },
        ]
    }

    pub fn get_skybox_bindgroup_entry_blur<'a>(&'a self) -> Vec<wgpu::BindGroupEntry<'a>> {
        let Ibl {
            sampler,
            irradiance_view,
            ..
        } = self;

        vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&irradiance_view),
            },
        ]
    }
}

pub struct IblManager {
    _brdf_lut: wgpu::Texture,
    _brdf_lut_view: wgpu::TextureView,
    ibl: Ibl,
}

impl IblManager {
    pub fn new(
        hdr_id: crate::assets::TextureId,
        hdr: &GpuTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        // Create BRDF LUT texture for PBR
        let brdf_lut = BRDFLUTBuilder::build(device, queue);
        let brdf_lut_view = brdf_lut.create_view(&wgpu::TextureViewDescriptor::default());
        let ibl = Self::create_ibl(hdr_id, hdr, brdf_lut_view.clone(), device, queue);

        Self {
            _brdf_lut: brdf_lut,
            _brdf_lut_view: brdf_lut_view,
            ibl,
        }
    }

    pub fn update_ibl(
        &mut self,
        hdr_id: crate::assets::TextureId,
        hdr: &GpuTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if self.ibl.hdr_id == hdr_id {
            return;
        }

        self.ibl = Self::create_ibl(hdr_id, hdr, self._brdf_lut_view.clone(), device, queue);
    }

    pub fn get_ibl(&self) -> &Ibl {
        &self.ibl
    }

    pub fn get_hdr_id(&self) -> crate::assets::TextureId {
        self.ibl.hdr_id
    }

    fn create_ibl(
        hdr_id: crate::assets::TextureId,
        hdr: &GpuTexture,
        brdf_lut_view: wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Ibl {
        let cube_map = EquirectangularToCubemap::build(&hdr, device, queue, 512);
        let _irradiance_map = IrrarianceMap::build(&cube_map, device, queue);
        let _prefilter_map = PrefilterMap::build(device, queue, &cube_map);
        let cube_map_view = cube_map.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let irradiance_view = _irradiance_map.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let prefilter_view = _prefilter_map.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        Ibl {
            hdr_id,
            _cube_map: cube_map,
            _cube_map_view: cube_map_view,
            _irradiance_map,
            irradiance_view,
            _prefilter_map,
            prefilter_view,
            sampler,
            brdf_lut_view,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assets::asset_manager::AssetManager, test_utils};

    const HDR_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/newport_loft.hdr");

    #[test]
    fn ibl_manager_is_initialized() {
        let (device, queue) = test_utils::get_device_and_queue();
        let mut asset_mgr = AssetManager::default();
        let hdr_id = asset_mgr
            .textures
            .from_file(HDR_PATH, crate::assets::TextureUsage::HDR16);
        asset_mgr.textures.load_cpu_textures();

        let mut texture_cache = GpuTextureCache::new(device, queue);
        texture_cache.upload_textures(&mut asset_mgr.textures, device, queue);
        let hdr = texture_cache.get_or_fallback_white(hdr_id);

        let _manager = IblManager::new(hdr_id, hdr, &device, &queue);
    }
}

use crate::{
    assets::{
        MATERIAL_TEXTURE_COUNT, MaterialAssets, MaterialId, MaterialTextureSlot, TextureAssets,
    },
    renderer::{GpuResourceStats, HasGpuStats},
    uniform::MaterialUniform,
};

use super::*;

use slotmap::SecondaryMap;
use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct GpuMaterialCache {
    map: SecondaryMap<MaterialId, GpuMaterial>,
    stats: GpuResourceStats,
}

impl HasGpuStats for GpuMaterialCache {
    fn get_stats(&self) -> GpuResourceStats {
        self.stats.clone()
    }
}

#[derive(Default)]
pub struct GpuMaterial {
    pub bind_group: Option<wgpu::BindGroup>,
    pub uniform_buffer: Option<wgpu::Buffer>,
}
impl GpuMaterial {
    const MATERIAL_SIZE: usize = size_of::<MaterialUniform>();
    fn estimated_size() -> usize {
        Self::MATERIAL_SIZE
    }
}

impl GpuMaterialCache {
    pub fn ensure(
        &mut self,
        id: MaterialId,
        gpu_texture_cache: &mut GpuTextureCache,
        assets: &AssetManager,
        gpu_mgr: &GpuManager,
        device: &wgpu::Device,
    ) {
        if !self.map.contains_key(id) {
            let value = Self::create_gpu_material(id, gpu_texture_cache, assets, gpu_mgr, device);
            self.map.insert(id, value);
            self.stats.add(GpuMaterial::estimated_size());
        }
    }

    pub fn retain(&mut self, assets: &MaterialAssets) {
        // Sync cleanup
        self.map.retain(|id, _| {
            if assets.contains_key(id) {
                true //mantain
            } else {
                // update stats
                self.stats.remove(GpuMaterial::estimated_size());
                trace!("removed gpu material {:?}", id);
                false // remove
            }
        });
    }

    pub fn update(&self, id: &MaterialId, queue: &wgpu::Queue, uniform: &MaterialUniform) {
        if let Some(material) = self.map.get(*id) {
            if let Some(buffer) = &material.uniform_buffer {
                queue.write_buffer(buffer, 0, bytemuck::bytes_of(uniform));
            }
        }
    }

    pub fn get(&self, id: &MaterialId) -> Option<&GpuMaterial> {
        self.map.get(*id)
    }

    fn create_gpu_material(
        material_id: crate::assets::MaterialId,
        texture_cache: &mut GpuTextureCache,
        asset_manager: &AssetManager,
        gpu_manager: &GpuManager,
        device: &wgpu::Device,
    ) -> GpuMaterial {
        let material_desc = asset_manager.materials.get_desc(material_id).unwrap();
        let uniform_buffer = create_uniform_from_desc(device, material_desc);

        let bindgroup = create_bindgroup_from_desc(
            device,
            asset_manager,
            texture_cache,
            material_desc,
            &uniform_buffer,
            gpu_manager,
        );

        GpuMaterial {
            bind_group: Some(bindgroup),
            uniform_buffer: Some(uniform_buffer),
        }
    }
}

fn create_uniform_from_desc(device: &wgpu::Device, material_desc: &MaterialDesc) -> wgpu::Buffer {
    let uniform = MaterialUniform::from(material_desc);

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: bytemuck::bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    uniform_buffer
}

use std::ops::Index;
pub struct TextureViews<'a>(pub [&'a wgpu::TextureView; MATERIAL_TEXTURE_COUNT]);

impl<'a> Index<MaterialTextureSlot> for TextureViews<'a> {
    type Output = wgpu::TextureView;

    fn index(&self, slot: MaterialTextureSlot) -> &Self::Output {
        &self.0[slot as usize]
    }
}

fn resolve_texture_views<'a>(
    texture_cache: &'a GpuTextureCache,
    desc: &MaterialDesc,
    texture_assets: &TextureAssets,
) -> TextureViews<'a> {
    use MaterialTextureSlot::*;
    let fallback = texture_assets.white();

    TextureViews([
        texture_cache.view(desc.texture(BaseColor).unwrap_or_else(|| fallback)),
        texture_cache.view(desc.texture(Normal).unwrap_or_else(|| fallback)),
        texture_cache.view(desc.texture(MetallicRoughness).unwrap_or_else(|| fallback)),
        texture_cache.view(desc.texture(Emissive).unwrap_or_else(|| fallback)),
        texture_cache.view(desc.texture(Occlusion).unwrap_or_else(|| fallback)),
        texture_cache.view_or(desc.texture(Transmission), CacheTextureSlot::Black),
        texture_cache.view_or(desc.texture(Volume), CacheTextureSlot::White),
    ])
}

fn create_bindgroup_from_desc(
    device: &wgpu::Device,
    asset_manager: &AssetManager,
    texture_cache: &mut GpuTextureCache,
    material_desc: &MaterialDesc,
    uniform_buffer: &wgpu::Buffer,
    gpu_manager: &GpuManager,
) -> wgpu::BindGroup {
    // Default sampler for all material textures (can be overridden by texture asset)
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });
    use MaterialTextureSlot::*;

    let views = resolve_texture_views(texture_cache, material_desc, &asset_manager.textures);

    let texture_bind_group_layout = gpu_manager.get_bindgroup_layout(BindgroupLayoutKind::Material);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        label: Some("Material  bind_group"),
        entries: &[
            // uniform buffer
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            // sampler
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            // main texture
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&views[BaseColor]),
            },
            // normal texture
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&views[Normal]),
            },
            // metallic_roughness texture
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&views[MetallicRoughness]),
            },
            // material emissive
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&views[Emissive]),
            },
            // material occlusion
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(&views[Occlusion]),
            },
            // material transmission
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::TextureView(&views[Transmission]),
            },
            // material volume
            wgpu::BindGroupEntry {
                binding: 8,
                resource: wgpu::BindingResource::TextureView(&views[Volume]),
            },
        ],
    });
    bind_group
}

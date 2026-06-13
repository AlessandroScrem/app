use crate::{
    assets::{MATERIAL_TEXTURE_COUNT, MaterialId, MaterialTextureSlot},
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
    fn retain<F>(&mut self, contains: F)
    where
        F: Fn(&MaterialId) -> bool,
    {
        // Sync cleanup
        self.map.retain(|id, _| {
            let keep = contains(&id);
            if !keep {
                // remove id
                // update stats
                self.stats.remove(GpuMaterial::estimated_size());
                trace!("removed gpu material {:?}", id);
            }
            keep
        });
    }

    pub fn sync(
        &mut self,
        texture_cache: &mut GpuTextureCache,
        device: &wgpu::Device,
        gpu_manager: &GpuManager,
        inputs: &[SyncInput<MaterialId, MaterialDesc>],
    ) {
        let desired: HashSet<_> = inputs.iter().map(|i| i.id).collect();

        self.retain(|id| desired.contains(id));

        for input in inputs {
            if !self.map.contains_key(input.id) {
                let value =
                    Self::create_gpu_material(texture_cache, input.data, gpu_manager, device);
                self.map.insert(input.id, value);
                self.stats.add(GpuMaterial::estimated_size());
            }
        }
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

    pub fn create_gpu_material(
        texture_cache: &mut GpuTextureCache,
        material_desc: &MaterialDesc,
        gpu_manager: &GpuManager,
        device: &wgpu::Device,
    ) -> GpuMaterial {
        let uniform_buffer = create_material_uniform_from_desc(device, material_desc);

        let bindgroup = create_bindgroup_from_desc(
            device,
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

pub fn create_material_uniform_from_desc(device: &wgpu::Device, material_desc: &MaterialDesc) -> wgpu::Buffer {
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
) -> TextureViews<'a> {
    use MaterialTextureSlot::*;

    TextureViews([
        texture_cache.view_or(desc.texture(BaseColor), CacheTextureSlot::White),
        texture_cache.view_or(desc.texture(Normal), CacheTextureSlot::White),
        texture_cache.view_or(desc.texture(MetallicRoughness), CacheTextureSlot::White),
        texture_cache.view_or(desc.texture(Emissive), CacheTextureSlot::White),
        texture_cache.view_or(desc.texture(Occlusion), CacheTextureSlot::White),
        texture_cache.view_or(desc.texture(Transmission), CacheTextureSlot::Black),
        texture_cache.view_or(desc.texture(Volume), CacheTextureSlot::White),
    ])
}

fn create_bindgroup_from_desc(
    device: &wgpu::Device,
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

    let views = resolve_texture_views(texture_cache, material_desc);

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

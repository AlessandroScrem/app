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
    const MATERIAL_SIZE:usize = size_of::<MaterialUniform>();
    fn estimated_size()->usize {
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
        queue: &wgpu::Queue,
    ) {
        if !self.map.contains_key(id) {
            let value =
                Self::create_gpu_material(id, gpu_texture_cache, assets, gpu_mgr, device, queue);
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
        queue: &wgpu::Queue,
    ) -> GpuMaterial {
        let material_desc = asset_manager.materials.get_desc(material_id).unwrap();
        let uniform_buffer = create_uniform_from_desc(device, material_desc);

        let bindgroup = create_bindgroup_from_desc(
            device,
            queue,
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

fn resolve_texture_id(
    slot: MaterialTextureSlot,
    desc: &MaterialDesc,
    texture_assets: &TextureAssets,
) -> TextureId {
    desc.key.textures[slot as usize].unwrap_or_else(|| texture_assets.white())
}

fn create_bindgroup_from_desc(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    asset_manager: &AssetManager,
    texture_cache: &mut GpuTextureCache,
    material_desc: &MaterialDesc,
    uniform_buffer: &wgpu::Buffer,
    gpu_manager: &GpuManager,
) -> wgpu::BindGroup {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    use crate::assets::material_asset::MaterialTextureSlot::*;

    let base_id = resolve_texture_id(BaseColor, material_desc, &asset_manager.textures);
    let normal_id = resolve_texture_id(Normal, material_desc, &asset_manager.textures);
    let met_rough_id =
        resolve_texture_id(MetallicRoughness, material_desc, &asset_manager.textures);
    let emissive_id = resolve_texture_id(Emissive, material_desc, &asset_manager.textures);
    let occlusion_id = resolve_texture_id(Occlusion, material_desc, &asset_manager.textures);

    texture_cache.ensure(base_id, device, queue);
    texture_cache.ensure(normal_id, device, queue);
    texture_cache.ensure(met_rough_id, device, queue);
    texture_cache.ensure(emissive_id, device, queue);
    texture_cache.ensure(occlusion_id, device, queue);

    let base_view = texture_cache.view(base_id);
    let normal_view = texture_cache.view(normal_id);
    let met_rough_view = texture_cache.view(met_rough_id);
    let emissive_view = texture_cache.view(emissive_id);
    let occlusion_view = texture_cache.view(occlusion_id);

    let texture_bind_group_layout = gpu_manager.get_layout(LayoutKind::Material);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        label: Some("Material  bind_group"),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            // main texture
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(base_view),
            },
            // normal texture
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(normal_view),
            },
            // metallic_roughness texture
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(met_rough_view),
            },
            // material emissive
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(emissive_view),
            },
            // material occlusion
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(occlusion_view),
            },
            // uniform buffer
            wgpu::BindGroupEntry {
                binding: 6,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    });
    bind_group
}

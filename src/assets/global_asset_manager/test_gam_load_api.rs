#![cfg_attr(test, allow(warnings))]

use std::path::PathBuf;

use super::*;
use log::trace;
use wgpu::BindGroupLayout;
use wgpu::util::DeviceExt;

use crate::TextureError;
use crate::assets::file;
use crate::assets::image_decoder::{
    decode_image_rgbaf32, decode_stb_image_rgaba8, decode_stb_image_rgbaf16,
};
use crate::assets::material_desc::MaterialDesc;
use crate::assets::texture_asset::{
    ColorSpace, SamplerDesc, TextureDesc, TextureState, TextureUsage,
};
use crate::gpu::texture_upload::{TextureData, UploadPayload};
use crate::renderer::GpuTexture;
use crate::renderer::GpuTextureBuilder;
use crate::assets::ResourceStats;

///////////////////////////////
// TEXTURE
///////////////////////////////
#[derive(Clone)]
pub struct TextureAsset {
    pub state: TextureState,
    pub desc: TextureDesc,
}

impl Asset for TextureAsset {
    type Key = TextureDesc;

    fn key(&self) -> &Self::Key {
        &self.desc
    }
}
///////////////////////////////
// MATERIAL
///////////////////////////////
#[derive(Clone)]
pub struct MaterialAsset {
    pub stats: ResourceStats,
    pub desc: MaterialDesc,
}

impl Asset for MaterialAsset {
    type Key = String;

    fn key(&self) -> &Self::Key {
        &self.desc.name
    }
}


///////////////////////////////
// MESH
///////////////////////////////
use crate::renderer::MeshVertexData;
use crate::BoundingBox;

#[derive(Default)]
pub struct MeshDesc {
    pub vertices: Vec<MeshVertexData>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<SubMesh>,
    pub bounds: BoundingBox,
}

pub struct SubMesh {
    pub index_range: std::ops::Range<u32>,
    pub material: GlobalAssetId,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum MeshSource {
    File {
        path: PathBuf,
        submesh_index: usize, // submesh index nel file gltf
    },
}

pub struct MeshAsset {
    pub desc: MeshDesc,
    pub mesh_source: MeshSource,
    pub stats: ResourceStats,
}

impl Asset for MeshAsset {
    type Key = MeshSource;

    fn key(&self) -> &Self::Key {
        &self.mesh_source
    }
}

// fn load_and_decode(desc: Option<&TextureDesc>) -> Result<UploadPayload, TextureError> {
//     let desc = match desc {
//         Some(d) => d,
//         None => {
//             return Ok(UploadPayload::Fallback);
//         }
//     };

//     let (path, color_space) = match desc {
//         TextureDesc::File { path, usage, .. } => (path, usage.color_space()),

//         TextureDesc::White => {
//             return Ok(UploadPayload::Fallback);
//         }
//     };

//     println!("read texture {:?}", path.as_path());

//     let buffer = file::read_bytes(path)?;

//     let (pixels, width, height) = match color_space {
//         ColorSpace::Rgba8 | ColorSpace::Srgba8 => decode_stb_image_rgaba8(&buffer)?,
//         ColorSpace::Rgbaf16 => decode_stb_image_rgbaf16(&buffer)?,
//         ColorSpace::Rgbaf32 => decode_image_rgbaf32(&buffer)?,
//         ColorSpace::Rg32ui => unimplemented!(),
//         ColorSpace::Depth32f => unimplemented!(),
//     };

//     Ok(UploadPayload::Ready(TextureData {
//         format: color_space,
//         width,
//         height,
//         pixels,
//     }))
// }

fn create_gpu_texture_from_cpu(
    payload: UploadPayload,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Option<GpuTexture> {
    match payload {
        UploadPayload::Ready(data) => {
            Some(GpuTextureBuilder::from_cpu(data).build(device, Some(queue)))
        }
        UploadPayload::Fallback => None,
    }
}



fn create_material_bindgroup_from_desc(
    device: &wgpu::Device,
    texture_cache: &GpuTextureCache,
    material_desc: &MaterialDesc,
    uniform_buffer: &wgpu::Buffer,
    bind_group_layout: &BindGroupLayout,
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
    use crate::assets::material_desc::MaterialTextureSlot::*;

    pub struct GpuTextures<'a>(
        pub [&'a GpuTexture; crate::assets::material_desc::MATERIAL_TEXTURE_COUNT],
    );

    use crate::assets::material_desc::MaterialTextureSlot;
    impl<'a> std::ops::Index<MaterialTextureSlot> for GpuTextures<'a> {
        type Output = GpuTexture;

        fn index(&self, slot: MaterialTextureSlot) -> &Self::Output {
            self.0[slot as usize]
        }
    }

    fn resolve_textures<'a>(
        texture_cache: &'a GpuTextureCache,
        desc: &MaterialDesc,
    ) -> GpuTextures<'a> {
        use crate::assets::material_desc::MaterialTextureSlot::*;

        GpuTextures([
            texture_cache.get_or(desc.texture(BaseColor), CacheTextureSlot::White),
            texture_cache.get_or(desc.texture(Normal), CacheTextureSlot::White),
            texture_cache.get_or(desc.texture(MetallicRoughness), CacheTextureSlot::White),
            texture_cache.get_or(desc.texture(Emissive), CacheTextureSlot::White),
            texture_cache.get_or(desc.texture(Occlusion), CacheTextureSlot::White),
            texture_cache.get_or(desc.texture(Transmission), CacheTextureSlot::Black),
            texture_cache.get_or(desc.texture(Volume), CacheTextureSlot::White),
        ])
    }

    let textures = resolve_textures(texture_cache, material_desc);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
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
                resource: wgpu::BindingResource::TextureView(&textures[BaseColor].view),
            },
            // normal texture
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&textures[Normal].view),
            },
            // metallic_roughness texture
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&textures[MetallicRoughness].view),
            },
            // material emissive
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&textures[Emissive].view),
            },
            // material occlusion
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(&textures[Occlusion].view),
            },
            // material transmission
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::TextureView(&textures[Transmission].view),
            },
            // material volume
            wgpu::BindGroupEntry {
                binding: 8,
                resource: wgpu::BindingResource::TextureView(&textures[Volume].view),
            },
        ],
    });
    bind_group
}


fn create_material_uniform_from_desc(
    device: &wgpu::Device,
    material_desc: &MaterialDesc,
) -> wgpu::Buffer {
    let uniform = crate::uniform::MaterialUniform::from(material_desc);

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: bytemuck::bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    uniform_buffer
}
//////////////////////////////
/// GPU TEXTURE
//////////////////////////////
use crate::renderer::caches::texture::{CacheTextureSlot, GpuBuiltinTextures};
struct GpuTextureCache {
    map: HashMap<GlobalAssetId, GpuTexture>,
    builtin: GpuBuiltinTextures,
}

impl GpuTextureCache {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let builtin = GpuBuiltinTextures::new(device, queue);

        Self {
            map: HashMap::new(),
            builtin,
        }
    }

    pub fn insert(&mut self, id: GlobalAssetId, texture: GpuTexture) {
        self.insert(id, texture);
    }

    pub fn contain(&self, id: GlobalAssetId) -> bool {
        self.map.contains_key(&id)
    }

    pub fn get(&self, id: GlobalAssetId) -> &GpuTexture {
        &self.get_or(Some(id), CacheTextureSlot::White)
    }

    pub fn get_or(&self, id: Option<GlobalAssetId>, slot: CacheTextureSlot) -> &GpuTexture {
        id.and_then(|id| self.map.get(&id))
            .unwrap_or_else(|| self.builtin.get(slot))
    }
}

//////////////////////////////
/// GPU MATERIAL
//////////////////////////////
#[derive(Default)]
pub struct GpuMaterial {
    pub bind_group: Option<wgpu::BindGroup>,
    pub uniform_buffer: Option<wgpu::Buffer>,
}

//////////////////////////////
/// GPU MESH
//////////////////////////////
#[derive(Default)]
pub struct GpuMesh {
    pub vertexbuffer: Option<wgpu::Buffer>,
    pub indexbuffer: Option<wgpu::Buffer>,
    _indexcount: u32,
    estimated_size: usize,
}

fn create_gpu_mesh(
    device: &wgpu::Device,
    vertices: &Vec<MeshVertexData>,
    indices: &Vec<u32>,
) -> GpuMesh {
    let vertexbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Mesh Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let indexbuffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Mesh Index Buffer"),
        contents: &bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let indexcount = indices.len() as u32;
    let estimated_size = vertices.len() + indices.len();

    GpuMesh {
        vertexbuffer: Some(vertexbuffer),
        indexbuffer: Some(indexbuffer),
        _indexcount: indexcount,
        estimated_size,
    }
}

#[test]
fn same_texture_same_id() {
    let mut mgr = GlobalAssetManager::new();

    let desc = TextureDesc::File {
        path: "albedo.png".into(),
        usage: TextureUsage::Albedo,
        sampler: SamplerDesc::default(),
        mipmaps: true,
    };

    let texture = TextureAsset {
        desc: desc,
        state: TextureState::MetaOnly,
    };

    let a = mgr.add(texture.clone());
    let b = mgr.add(texture);

    assert_eq!(a, b);
}

#[test]
fn texture_created_event() {
    use crate::test_utils;
    use crate::assets::texture_upload::load_and_decode;

    const TEXTURE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/white.png");

    let (device, queue) = test_utils::get_device_and_queue();

    let mut mgr = GlobalAssetManager::new();

    let desc = TextureDesc::File {
        path: TEXTURE_PATH.into(),
        usage: TextureUsage::Albedo,
        sampler: SamplerDesc::default(),
        mipmaps: true,
    };

    let texture = TextureAsset {
        desc: desc,
        state: TextureState::MetaOnly,
    };

    let id = mgr.add(texture);

    assert_eq!(mgr.events.len(), 1);

    let grouped = mgr.drain_grouped_events();

    let mut texture_cache = GpuTextureCache::new(device, queue);

    if let Some(tex_created) = grouped.get(&(TypeId::of::<TextureAsset>(), AssetEventKind::Created))
    {
        for ev in tex_created {
            let asset = mgr.get::<TextureAsset>(ev.id).unwrap();
            let data = load_and_decode(asset.desc.clone()).unwrap();
            let texture = GpuTextureBuilder::from_cpu(data).build(device, Some(queue));

            texture_cache.insert(ev.id, texture);
        }
    }

    assert!(mgr.events.is_empty());
    assert!(texture_cache.contain(id))
}

#[test]
fn material_created_event() {
    use crate::renderer::BindgroupLayoutKind;
    use crate::renderer::gpu::BindgroupLayoutCache;
    use crate::test_utils;

    let (device, queue) = test_utils::get_device_and_queue();
    let layout_cache = BindgroupLayoutCache::new(device);
    let bind_group_layout = layout_cache.get(BindgroupLayoutKind::Material);

    let texture_cache = GpuTextureCache::new(device, queue);

    let mut mgr = GlobalAssetManager::new();

    let material = MaterialAsset {
        stats: ResourceStats::default(),
        desc: MaterialDesc::default(),
    };

    let id = mgr.add(material);

    assert_eq!(mgr.events.len(), 1);

    let grouped = mgr.drain_grouped_events();

    let mut gpu_materials: HashMap<GlobalAssetId, GpuMaterial> = Default::default();

    if let Some(tex_created) =
        grouped.get(&(TypeId::of::<MaterialAsset>(), AssetEventKind::Created))
    {
        for ev in tex_created {
            let asset = mgr.get::<MaterialAsset>(ev.id).unwrap();

            let uniform_buffer = create_material_uniform_from_desc(device, &asset.desc);
            let bind_group = create_material_bindgroup_from_desc(
                device,
                &texture_cache,
                &asset.desc,
                &uniform_buffer,
                bind_group_layout,
            );

            let gpu_material = GpuMaterial {
                uniform_buffer: Some(uniform_buffer),
                bind_group: Some(bind_group),
            };

            gpu_materials.insert(ev.id, gpu_material);
        }
    }

    assert!(mgr.events.is_empty());

    assert_eq!(gpu_materials.len(), 1);
    assert!(gpu_materials.get(&id).is_some())
}

#[test]
fn mesh_created_event() {
    use crate::renderer::BindgroupLayoutKind;
    use crate::renderer::gpu::BindgroupLayoutCache;
    use crate::test_utils;

    let (device, queue) = test_utils::get_device_and_queue();

    let mut mgr = GlobalAssetManager::new();

    const MESH_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/core/cube/cube.gltf");
    let mesh_source = MeshSource::File { path: MESH_PATH.into(), submesh_index: 0 };

    let mesh = MeshAsset {
        stats: ResourceStats::default(),
        mesh_source,
        desc: MeshDesc::default(),
    };

    let id = mgr.add(mesh);

    assert_eq!(mgr.events.len(), 1);

    let grouped = mgr.drain_grouped_events();

    let mut gpu_meshes: HashMap<GlobalAssetId, GpuMesh> = Default::default();

    if let Some(tex_created) =
        grouped.get(&(TypeId::of::<MeshAsset>(), AssetEventKind::Created))
    {
        for ev in tex_created {
            let asset = mgr.get::<MeshAsset>(ev.id).unwrap();
            let gpu_mesh = create_gpu_mesh(&device, &asset.desc.vertices, &asset.desc.indices);
            gpu_meshes.insert(ev.id, gpu_mesh);
        }
    }

    assert!(mgr.events.is_empty());

    assert_eq!(gpu_meshes.len(), 1);
    assert!(gpu_meshes.get(&id).is_some())
}

use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, RenderPipeline};

use crate::{
    assets::{
        self,
        texture_asset::{ColorSpace, SamplerDesc},
    },
    gpu::{
        GpuResourceStats, GpuTexture, GpuTextureBuilder, GpuTextureUsage, HasGpuStats,
        pipeline_manager::PipelineExt,
    },
};

pub struct GpuShadow {
    texture: GpuTexture,
    bind_group: BindGroup,
}

impl GpuShadow {
    #[allow(unused)]
    fn estimated_size(&self) -> usize {
        self.texture.estimated_size
    }
}

impl GpuShadow {
    pub fn get_view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }
    #[allow(unused)]
    pub fn get_texture(&self) -> &wgpu::Texture {
        &self.texture.inner
    }
    pub fn get_gpu_texture(&self) -> &GpuTexture {
        &self.texture
    }
    pub fn get_bg(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

type LightId = usize;

const MAX_SHADOWS: usize = 64;
const SHADOWS_SIZE: u32 = 1024;

impl HasGpuStats for ShadowManager {
    fn get_stats(&self) -> GpuResourceStats {
        self.stats.clone()
    }
}

pub struct ShadowManager {
    shadowmap_create_bindgroup: BindGroup,
    texture_rgba: GpuShadow,
    pipeline: RenderPipeline,
    shadow_maps: Vec<GpuShadow>,
    sampler: wgpu::Sampler,
    stats: GpuResourceStats,
}

impl ShadowManager {
    pub fn new(device: &wgpu::Device, layout: &BindGroupLayout, light_buffer: &Buffer) -> Self {
        let shadow_maps: Vec<GpuShadow> = (0..MAX_SHADOWS)
            .map(|_| Self::create_gpu_shadow(device, layout))
            .collect();

        let stats = shadow_maps
            .iter()
            .fold(GpuResourceStats::default(), |mut s, shadow| {
                s.add(shadow.estimated_size());
                s
            });

        let texture_rgba = Self::crate_texture_rgba(device);
        let layout = &Self::create_shadowmap_create_layout(device);
        let shadowmap_create_bindgroup =
            Self::crate_shadowmap_create_bg(device, &light_buffer, layout);
            let pipeline = Self::create_pipeline(device, layout);
            
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                compare: Some(wgpu::CompareFunction::LessEqual),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                ..Default::default()
            });

        Self {
            pipeline,
            shadowmap_create_bindgroup,
            shadow_maps,
            sampler,
            texture_rgba,
            stats,
        }
    }
}

impl ShadowManager {
    pub fn get_shadowmap(&self, id: LightId) -> Option<&GpuShadow> {
        self.shadow_maps.get(id)
    }

    pub fn get_sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn get_views(&self) -> Vec<&wgpu::TextureView> {
        self.shadow_maps.iter().map(|s| s.get_view()).collect()
    }

    pub fn get_rgba(&self) -> &GpuShadow {
        &self.texture_rgba
    }

    pub fn get_create_bg(&self) -> &BindGroup {
        &self.shadowmap_create_bindgroup
    }
    pub fn get_pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }
}

impl ShadowManager {
    fn create_gpu_shadow(device: &Device, layout: &BindGroupLayout) -> GpuShadow {
        let texture = GpuTextureBuilder::from_empty(SHADOWS_SIZE, SHADOWS_SIZE)
            .format(ColorSpace::Depth32f)
            .usage(GpuTextureUsage::SampledTexture)
            .sampler(SamplerDesc::NearestClamp)
            .label("shadow_texture depth")
            .build(device, None);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadowmap_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
            ],
        });
        GpuShadow {
            texture,
            bind_group,
        }
    }

    fn crate_texture_rgba(device: &Device) -> GpuShadow {
        let texture = GpuTextureBuilder::from_empty(SHADOWS_SIZE, SHADOWS_SIZE)
            .format(ColorSpace::Rgba8)
            .usage(GpuTextureUsage::SampledTexture)
            .sampler(SamplerDesc::LinearRepeat)
            .label("shadow_texture rgba")
            .build(device, None);

        let layout = &Self::create_rgba_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ShadowMapRgba_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
            ],
        });

        GpuShadow {
            texture,
            bind_group,
        }
    }

    fn create_rgba_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TextureRgba_bind_group_layout"),
            entries: &[
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_shadowmap_create_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ShadowMapCreate_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                // Ligth
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }
    fn crate_shadowmap_create_bg(
        device: &Device,
        uniform_buffer: &Buffer,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                // Light
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("ShadowMapCreate_bind_group"),
        })
    }

    fn create_pipeline(device: &Device, layout: &BindGroupLayout) -> RenderPipeline {
        let layouts = [
            Some(layout), //0
            None,         //1
            None,         //2
            None,         //3
        ];
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ShadowMapCreate Pipeline Layout"),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            });
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shadow_map.wgsl"));
        let buffer_desc = &[
            assets::MeshVertexData::get_layout(),
            assets::VertexInstance::get_layout(),
        ];

        let pipeline_desc = PipelineExt::default();

        let target = &[];

        pipeline_desc.build_pipeline(
            "ShadowMap Pipeline",
            device,
            Some(&render_pipeline_layout),
            target,
            shader,
            buffer_desc,
        )
    }
}

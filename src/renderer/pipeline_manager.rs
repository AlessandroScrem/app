use crate::renderer::gpu_manager::{GPUResourceManager, LayoutKind};
use wgpu::DepthStencilState;

/// A description of a render pipeline.
/// Note: You can call `default()` to get a base implementation.
#[derive(Debug, Hash, Clone)]
pub struct PipelineDesc {
    pub primitive: wgpu::PrimitiveState,
    pub multisample: wgpu::MultisampleState,
    pub depth_stencil: Option<DepthStencilState>,
    pub blend: Option<wgpu::BlendState>,
}
impl Default for PipelineDesc {
    fn default() -> Self {
        Self {
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float, // o quello che hai usato per creare la texture
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            blend: Some(wgpu::BlendState::REPLACE),
        }
    }
}
impl PipelineDesc {
    pub fn build_pipeline(
        self,
        device: &wgpu::Device,
        layout: wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
        shader: wgpu::ShaderModule,
        buffers: &[wgpu::VertexBufferLayout<'static>],
    ) -> wgpu::RenderPipeline {
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Render Pipeline")),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: self.blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: self.primitive,
            multisample: self.multisample,
            depth_stencil: self.depth_stencil,
            multiview: None,
            cache: None,
        });

        render_pipeline
    }
}

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter )]
pub enum PipelineKind {
    Default,
    Light,
    Skybox,
}

pub struct PipelineManager {
    pipelines: Vec<wgpu::RenderPipeline>,
}

impl PipelineManager {
    pub fn new(device: &wgpu::Device, gpu_resource_manager: &GPUResourceManager, format: wgpu::TextureFormat) -> Self {

        let pipelines:Vec<wgpu::RenderPipeline> = PipelineKind::iter()
            .map(|kind| create_pipeline(device, gpu_resource_manager, kind, format))
            .collect();


        Self {
            pipelines,
        }
    }

    pub fn get_render_pipeline(&self, kind: PipelineKind) -> &wgpu::RenderPipeline {
        &self.pipelines[kind as usize]
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    gpu_resource_manager: &GPUResourceManager,
    kind: PipelineKind,
    format: wgpu::TextureFormat
) -> wgpu::RenderPipeline {
    match kind {
        PipelineKind::Default => {
            let layouts: Vec<&wgpu::BindGroupLayout> = vec![
                gpu_resource_manager.get_layout(LayoutKind::Camera), //0
                gpu_resource_manager.get_layout(LayoutKind::Texture), //1
                gpu_resource_manager.get_layout(LayoutKind::Model),  //2
                gpu_resource_manager.get_layout(LayoutKind::Light),  //3
            ];
            let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &layouts,
                push_constant_ranges: &[],
            });
            let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));
            let buffer_desc = &[crate::assets::mesh::MeshVertexData::get_layout()];
            
            let pipeline_desc = PipelineDesc::default();
            
            pipeline_desc.build_pipeline(
                device,
                render_pipeline_layout,
                format,
                shader,
                buffer_desc,
            )
        }
        PipelineKind::Light => {
            let layouts: Vec<&wgpu::BindGroupLayout> = vec![
                gpu_resource_manager.get_layout(LayoutKind::Camera), //0
                gpu_resource_manager.get_layout(LayoutKind::Light),  //1
                gpu_resource_manager.get_layout(LayoutKind::LightTexture), //2
            ];

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                });

            let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/light.wgsl"));

            let buffer_desc = &[];
            let pipeline_desc = PipelineDesc::default();


            pipeline_desc.build_pipeline(
                device,
                render_pipeline_layout,
                format,
                shader,
                buffer_desc,
            )
        }
        PipelineKind::Skybox => {
            let layouts: Vec<&wgpu::BindGroupLayout> = vec![
                gpu_resource_manager.get_layout(LayoutKind::Camera), //0
                gpu_resource_manager.get_layout(LayoutKind::Skybox), //1
            ];

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                });

            let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/skybox.wgsl"));

            let buffer_desc = &[];

            let pipeline_desc = PipelineDesc {
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                ..Default::default()
            };

            pipeline_desc.build_pipeline(
                device,
                render_pipeline_layout,
                format,
                shader,
                buffer_desc,
            )
        }
   }
}

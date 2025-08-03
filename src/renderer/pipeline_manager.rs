use std::{collections::HashMap, sync::Arc};

use wgpu::DepthStencilState;

use crate::resources::gpu_manager::GPUResourceManager;

/// A description of a render pipeline.
/// Note: You can call `default()` to get a base implementation.
#[derive(Debug, Hash, Clone)]
pub struct PipelineDesc {
    primitive: wgpu::PrimitiveState,
    multisample: wgpu::MultisampleState,
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
        }
    }
}
impl PipelineDesc {
    pub fn build_pipeline(
        &self,
        device: &wgpu::Device,
        layout: wgpu::PipelineLayout,
        config: &wgpu::SurfaceConfiguration,
        shader: wgpu::ShaderModule,
        buffer_desc: wgpu::VertexBufferLayout<'static>,
    ) -> wgpu::RenderPipeline {
        let format = config.format;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[buffer_desc],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: self.primitive,
            multisample: self.multisample,
            depth_stencil: Some(DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float, // o quello che hai usato per creare la texture
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        render_pipeline
    }
}

/// An actual Render Pipeline that should be stored in the manager.
/// Also contains a description of the pipeline.
pub struct Pipeline {
    pub render_pipeline: wgpu::RenderPipeline,
}

pub enum PipelineType {
    Pipeline(Pipeline),
}

pub struct PipelineManager {
    pipelines: HashMap<String, PipelineType>,
}

impl PipelineManager {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    pub fn get_render_pipeline(&self, name: &str) -> Option<&wgpu::RenderPipeline> {
        match self.pipelines.get(name) {
            Some(PipelineType::Pipeline(p)) => Some(&p.render_pipeline),
            _ => None,
        }
    }

    pub fn add_pipeline(
        &mut self,
        name: &str,
        device: &wgpu::Device,
        render_pipeline_layout: wgpu::PipelineLayout,
        shader: wgpu::ShaderModule,
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        if self.pipelines.contains_key(name) {
            return;
        }

        let desc = PipelineDesc::default();

        let buffer_desc = crate::assets::mesh::MeshVertexData::get_layout();

        let pipeline = desc.build_pipeline(
            device,
            render_pipeline_layout,
            surface_config,
            shader,
            buffer_desc,
        );

        let pipeline = Pipeline {
            render_pipeline: pipeline,
        };

        self.pipelines
            .insert(name.into(), PipelineType::Pipeline(pipeline));
    }
}

pub fn create_default_pipeline(resources: &legion::Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let surface_config = resources.get::<wgpu::SurfaceConfiguration>().unwrap();

    let layout_map = resource_manager.bind_group_layouts.lock().unwrap();
    
    let layouts: Vec<&wgpu::BindGroupLayout> = vec![
        layout_map.get("camera").unwrap(),
        layout_map.get("texture").unwrap(),
    ];

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &layouts,
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));

    pipeline_manager.add_pipeline(
        "default",
        &device,
        render_pipeline_layout,
        shader,
        &surface_config,
    );
}

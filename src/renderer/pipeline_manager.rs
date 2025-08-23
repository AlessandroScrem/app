use std::{collections::HashMap, sync::Arc};

use wgpu::DepthStencilState;

use crate::resources::gpu_manager::{GPUResourceManager, LayoutKind};

/// A description of a render pipeline.
/// Note: You can call `default()` to get a base implementation.
#[derive(Debug, Hash, Clone)]
pub struct PipelineDesc {
    pub primitive: wgpu::PrimitiveState,
    pub multisample: wgpu::MultisampleState,
    pub depth_stencil: Option<DepthStencilState>,
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
        }
    }
}
impl PipelineDesc {
    pub fn build_pipeline(
        self,
        name: &str,
        device: &wgpu::Device,
        layout: wgpu::PipelineLayout,
        format: wgpu::TextureFormat,
        shader: wgpu::ShaderModule,
        buffers: &[wgpu::VertexBufferLayout<'static>],
    ) -> wgpu::RenderPipeline {
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Render Pipeline {}", name)),
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
                    blend: Some(wgpu::BlendState::REPLACE),
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
        buffers: &[wgpu::VertexBufferLayout<'static>],
        shader: wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        pipeline_desc: PipelineDesc,
    ) {
        if self.pipelines.contains_key(name) {
            return;
        }

        let pipeline = pipeline_desc.build_pipeline(
            name,
            device,
            render_pipeline_layout,
            format,
            shader,
            buffers,
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

    let layouts: Vec<&wgpu::BindGroupLayout> = vec![
        resource_manager.get_layout(LayoutKind::Camera),  //0
        resource_manager.get_layout(LayoutKind::Texture), //1
        resource_manager.get_layout(LayoutKind::Model),   //2
        resource_manager.get_layout(LayoutKind::Light),   //3
    ];

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &layouts,
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));
    let buffer_desc = &[crate::assets::mesh::MeshVertexData::get_layout()];

    let pipeline_desc = PipelineDesc::default();

    pipeline_manager.add_pipeline(
        "default",
        &device,
        render_pipeline_layout,
        buffer_desc,
        shader,
        surface_config.format,
        pipeline_desc,
    );
}

pub fn create_light_pipeline(resources: &legion::Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let surface_config = resources.get::<wgpu::SurfaceConfiguration>().unwrap();

    let layouts: Vec<&wgpu::BindGroupLayout> = vec![
        resource_manager.get_layout(LayoutKind::Camera), //0
        resource_manager.get_layout(LayoutKind::Light),  //1
        resource_manager.get_layout(LayoutKind::LightTexture), //2
    ];

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &layouts,
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::include_wgsl!("../light.wgsl"));

    let buffers = &[];
    let pipeline_desc = PipelineDesc::default();

    pipeline_manager.add_pipeline(
        "light",
        &device,
        render_pipeline_layout,
        buffers,
        shader,
        surface_config.format,
        pipeline_desc,
    );
}

pub fn create_skybox_pipeline(resources: &legion::Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let resource_manager = resources.get::<Arc<GPUResourceManager>>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let surface_config = resources.get::<wgpu::SurfaceConfiguration>().unwrap();


    let layouts: Vec<&wgpu::BindGroupLayout> = vec![
        resource_manager.get_layout(LayoutKind::Camera), //0
        resource_manager.get_layout(LayoutKind::Skybox), //1
    ];

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &layouts,
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::include_wgsl!("../skybox.wgsl"));

    let buffers = &[];

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

    pipeline_manager.add_pipeline(
        "skybox",
        &device,
        render_pipeline_layout,
        buffers,
        shader,
        surface_config.format,
        pipeline_desc,
    );
}

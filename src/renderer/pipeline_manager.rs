use std::collections::HashMap;

use crate::resources::gpu_manager::GPUResourceManager;

/// A description of a render pipeline.
/// Note: You can call `default()` to get a base implementation.
#[derive(Debug, Hash, Clone)]
pub struct PipelineDesc {
    primitive: wgpu::PrimitiveState,
    multisample: wgpu::MultisampleState,
    depth_stencil: Option<()>,
    multiview: Option<()>,
    cache: Option<()>,
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
            depth_stencil: None,
            multiview: None,
            cache: None,
        }
    }
}

/// An actual Render Pipeline that should be stored in the manager.
/// Also contains a description of the pipeline.
pub struct Pipeline {
    pub desc: PipelineDesc,
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
        resource_manager: &GPUResourceManager,
        shader: wgpu::ShaderModule,
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        if self.pipelines.contains_key(name) {
            return;
        }

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&resource_manager.camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let buffer_desc = crate::renderer::gpu_renderer::Vertex::desc();

        let pipeline = crate::renderer::pipeline::create_pipeline(device, &render_pipeline_layout, surface_config, shader, buffer_desc);

        let pipeline = Pipeline{
            desc: PipelineDesc::default(),
            render_pipeline: pipeline,
        };

        self.pipelines.insert(name.into() , PipelineType::Pipeline(pipeline));
    }
}

pub fn create_default_pipeline(resources: &legion::Resources) {
    let device = resources.get::<wgpu::Device>().unwrap();
    let resource_manager = resources.get::<GPUResourceManager>().unwrap();
    let mut pipeline_manager = resources.get_mut::<PipelineManager>().unwrap();
    let surface_config = resources.get::<wgpu::SurfaceConfiguration>().unwrap();

    let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));

    pipeline_manager.add_pipeline("default", &device, &resource_manager, shader, &surface_config);
}

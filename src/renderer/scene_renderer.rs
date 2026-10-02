use super::*;

use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{GpuCache, GpuContext, GpuManager, ShadowManager};
use crate::renderer::framebuilder::DrawStats;

use wgpu::Device;

use crate::prelude::{debug, info};
use crate::renderer::renderpass::*;
use crate::assets::{MaterialId, MeshVertexData, VertexInstance};
use crate::math::{perspective, Deg, Mat4, Point3f, Vec3};


pub struct MaterialPreviewRenderer {
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
    entity_target: wgpu::Texture,
    entity_view: wgpu::TextureView,
    depth: wgpu::Texture,
    depth_view: wgpu::TextureView,
    sphere: crate::gpu::GpuMesh,
    index_count: u32,
}

impl MaterialPreviewRenderer {
    const SIZE: u32 = 256;

    pub fn new(device: &wgpu::Device) -> Self {
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Material Preview HDR"),
            size: wgpu::Extent3d { width: Self::SIZE, height: Self::SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let target_view = target.create_view(&Default::default());

        let entity_target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Material Preview Entity"),
            size: wgpu::Extent3d { width: Self::SIZE, height: Self::SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg32Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let entity_view = entity_target.create_view(&Default::default());

        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Material Preview Depth"),
            size: wgpu::Extent3d { width: Self::SIZE, height: Self::SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth.create_view(&Default::default());

        let (vertices, indices) = create_preview_sphere(64, 32);
        let index_count = indices.len() as u32;
        let sphere = crate::gpu::GpuMesh::new(device, &vertices, &indices);

        Self {
            target,
            target_view,
            entity_target,
            entity_view,
            depth,
            depth_view,
            sphere,
            index_count,
        }
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        gpu_context: &GpuContext,
        gpu_manager: &GpuManager,
        gpu_cache: &GpuCache,
        pipeline_manager: &PipelineManager,
        material: Option<MaterialId>,
    ) {
        let Some(material) = material else { return };
        let Some(gpu_material) = gpu_cache.material.get(&material) else { return };
        let Some(material_bg) = gpu_material.bind_group.as_ref() else { return };

        let view = Mat4::look_at_rh(
            Point3f::new(0.0, 0.0, 3.0),
            Point3f::new(0.0, 0.0, 0.0),
            Vec3::unit_y(),
        );
        let proj = perspective(Deg(35.0), 1.0, 0.1, 10.0);
        let camera = crate::renderer::uniform::CameraUniform {
            view_position: [0.0, 0.0, 3.0, 1.0],
            view: view.into(),
            proj: proj.into(),
            screen_size: [Self::SIZE as f32, Self::SIZE as f32],
            ..Default::default()
        };

        let globals = crate::renderer::uniform::GlobalUniform {
            ibl_enable: 1,
            skybox_enable: 0,
            exposure: 0.0,
            ibl_intensity: 1.0,
            tonemap_filter: 0,
            ..Default::default()
        };

        let mut lights = crate::renderer::uniform::LightsUniform::default();
        lights.enabled = 1;
        lights.count = 1;
        lights.lights[0].color = [1.0, 1.0, 1.0];
        lights.lights[0].position = [2.0, 2.0, 3.0];
        lights.lights[0].directional = 0;
        lights.lights[0].cast_shadow = 0;

        let instance = VertexInstance::new(Mat4::identity(), 0);
        gpu_manager.update_buffer(&gpu_context.queue, BufferKind::Camera, std::slice::from_ref(&camera));
        gpu_manager.update_buffer(&gpu_context.queue, BufferKind::Globals, std::slice::from_ref(&globals));
        gpu_manager.update_buffer(&gpu_context.queue, BufferKind::Lights, std::slice::from_ref(&lights));
        gpu_manager.update_buffer(&gpu_context.queue, BufferKind::Instances, std::slice::from_ref(&instance));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Material Preview Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.035, g: 0.035, b: 0.035, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.entity_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        pass.set_pipeline(pipeline_manager.get_render_pipeline(PipelineKind::Pbr));
        pass.set_bind_group(0, gpu_manager.get_bindgroup(BindgroupKind::Perframe), &[]);
        pass.set_bind_group(1, material_bg, &[]);
        pass.set_bind_group(3, gpu_manager.get_bindgroup(BindgroupKind::PbrMap), &[]);
        pass.set_vertex_buffer(0, self.sphere.vertexbuffer.slice(..));
        pass.set_vertex_buffer(1, gpu_manager.get_buffer(BufferKind::Instances).slice(..));
        pass.set_index_buffer(self.sphere.indexbuffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    pub fn texture(&self) -> (&wgpu::Texture, &wgpu::TextureView, wgpu::Extent3d) {
        (
            &self.target,
            &self.target_view,
            wgpu::Extent3d { width: Self::SIZE, height: Self::SIZE, depth_or_array_layers: 1 },
        )
    }
}

fn create_preview_sphere(segments: u32, rings: u32) -> (Vec<MeshVertexData>, Vec<u32>) {
    let mut vertices = Vec::with_capacity(((segments + 1) * (rings + 1)) as usize);
    let mut indices = Vec::with_capacity((segments * rings * 6) as usize);

    for y in 0..=rings {
        let v = y as f32 / rings as f32;
        let theta = v * std::f32::consts::PI;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for x in 0..=segments {
            let u = x as f32 / segments as f32;
            let phi = u * std::f32::consts::TAU;
            let position = [sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin()];
            vertices.push(MeshVertexData {
                position,
                normal: position,
                tangent: [-phi.sin(), 0.0, phi.cos(), 1.0],
                uv: [u, 1.0 - v, 0.0, 0.0],
            });
        }
    }

    let row = segments + 1;
    for y in 0..rings {
        for x in 0..segments {
            let a = y * row + x;
            let b = a + 1;
            let c = a + row;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    (vertices, indices)
}

pub struct SceneRenderContext<'a> {
    pub gpu_context: &'a GpuContext,
    pub gpu_manager: &'a GpuManager,
    pub shadow_manager: &'a ShadowManager,
    pub pipeline_manager: &'a PipelineManager,
    pub gpu_cache: &'a GpuCache,
}

pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub gpu_cache: &'a GpuCache,

    pub gpu_mgr: &'a GpuManager,
    pub shadow_mgr: &'a ShadowManager,
    pub pip_mgr: &'a PipelineManager,
    pub target: &'a wgpu::TextureView,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct FrameStats {
    pub opaque: DrawStats,
    pub transmission: DrawStats,
}

pub struct SceneRenderer {
    default_pass: Vec<RenderPassEnum>,
    stats: FrameStats,
}

impl SceneRenderer {
    pub fn new() -> Self {
        let timer = std::time::Instant::now();
        info!("Initializing renderer...");

        debug!("Renderer initialized in {} ms", timer.elapsed().as_millis());

        let default_pass = vec![
            RenderPassEnum::Shadow(ShadowPass {}),
            RenderPassEnum::Mesh(MeshPass::opaque()),
            RenderPassEnum::Skybox(SkyboxPass {}),
            RenderPassEnum::BuildMipmaps(BuildMipmapsPass {}),
            RenderPassEnum::Transmission(MeshPass::transmission()),
            RenderPassEnum::LightsIcon(LightsIconPass {}),
            RenderPassEnum::Axis(AxisPass {}),
            RenderPassEnum::Lines(LinesPass {}),
            RenderPassEnum::Linearize(LinearizePass {}),
            RenderPassEnum::Outline(OutlinePass {}),
        ];

        Self {
            default_pass,
            stats: FrameStats::default(),
        }
    }

    pub fn get_render_stats(&self) -> FrameStats {
        self.stats
    }

    pub fn render(
        &mut self,
        runtime: &SceneRenderContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: &FrameData,
    ) {
        let SceneRenderContext {
            gpu_context,
            gpu_manager,
            shadow_manager,
            pipeline_manager,
            gpu_cache,
        } = runtime;

        let mut ctx = RenderContext {
            device: &gpu_context.device,
            gpu_cache: &gpu_cache,
            gpu_mgr: &gpu_manager,
            shadow_mgr: &shadow_manager,
            pip_mgr: &pipeline_manager,
            target,
        };

        for pass in &mut self.default_pass {
            pass.execute(encoder, &mut ctx, &frame);
        }

        self.stats = FrameStats {
            opaque: frame.opaque_stats,
            transmission: frame.transmission_stats,
        };
    }
}

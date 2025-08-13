use wgpu::IndexFormat;

use crate::{
    assets::mesh::Mesh,
    renderer::{gpu_renderer::DepthTexture, pipeline_manager::PipelineManager},
    resources::gpu_manager::GPUResourceManager,
    transform::Transform,
};

use legion::{world::SubWorld, *};
use std::sync::Arc;

#[system]
#[read_component(Mesh)]
#[read_component(Transform)]
pub fn mesh(
    world: &mut SubWorld,
    #[resource] frame_view: &wgpu::TextureView,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] depth_texture: &DepthTexture,
    #[resource] queue: &wgpu::Queue,
) {
    let mut mesh_query = <(&Mesh, &Transform)>::query();

    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };

    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: frame_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear_color),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_texture.0,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    let render_pipeline = pipeline_manager
        .get_render_pipeline("default")
        .expect("expected pipeline: 'default'");

    let map = gpu_resource_manager.bind_groups.lock().unwrap();
    let camera_bind_group = map.get("camera").unwrap();
    let model_bind_group = map.get("model").unwrap();

    renderpass.set_pipeline(render_pipeline);
    renderpass.set_bind_group(0, camera_bind_group, &[]);

    for (mesh, transform) in mesh_query.iter(world) {
        update_trnsform(transform, queue, gpu_resource_manager);
        renderpass.set_bind_group(2, model_bind_group, &[]);

        for submesh in mesh.submeshes.iter() {
            let vertex_buffer = submesh.vertex_buffer.as_ref().unwrap();
            let index_buffer = submesh.index_buffer.as_ref().unwrap();
            let index_count = submesh.index_count as u32;

            let texture_bind_group = submesh.material.bind_group.as_ref().unwrap();
            renderpass.set_bind_group(1, texture_bind_group, &[]);

            renderpass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
            renderpass.set_vertex_buffer(0, vertex_buffer.slice(..));
            renderpass.draw_indexed(0..index_count, 0, 0..1);
        }
    }
}

use crate::renderer::uniform::ModelUniform;
pub fn update_trnsform(
    transform: &Transform,
    queue: &wgpu::Queue,
    gpu_resource_manager: &GPUResourceManager,
) {
    let model_matrix = transform.compute_model_matrix();
    let updated_uniforms = ModelUniform::new(model_matrix);
    
    queue.write_buffer(
        &gpu_resource_manager.model_uniform_buffer,
        0,
        bytemuck::bytes_of(&updated_uniforms),
    );
}

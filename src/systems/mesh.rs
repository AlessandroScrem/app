use wgpu::IndexFormat;

use crate::{
    MeshComponent, TransformComponent,
    renderer::{
        gpu_manager::GPUResourceManager,
        gpu_renderer::DepthTexture,
        hdr_frame::HdrFrame,
        pipeline_manager::{PipelineKind, PipelineManager},
    },
};

use legion::{world::SubWorld, *};
use std::sync::Arc;

#[system]
#[read_component(MeshComponent)]
#[read_component(TransformComponent)]
pub fn mesh(
    world: &mut SubWorld,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] depth_texture: &DepthTexture,
    #[resource] hdr_texture: &HdrFrame,
    #[resource] ibl: &crate::renderer::gpu_renderer::Ibl,
) {
    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };

    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &hdr_texture.view,
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

    let render_pipeline = pipeline_manager.get_render_pipeline(PipelineKind::Pbr);

    renderpass.set_pipeline(render_pipeline);
    renderpass.set_bind_group(0, &gpu_resource_manager.globals_bind_group, &[]);
    renderpass.set_bind_group(3, &ibl.ibl_bind_group, &[]);

    let mut mesh_query = <(&MeshComponent, &TransformComponent)>::query();
    for (mesh, _) in mesh_query.iter(world) {
        renderpass.set_bind_group(2, &mesh.data.model_bind_group, &[]);

        for submesh in mesh.data.submeshes.iter() {
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
#[system(for_each)]
#[filter(maybe_changed::<TransformComponent>())]
pub fn update_model_matrix(
    transform: &TransformComponent,
    mesh: &MeshComponent,
    #[resource] queue: &wgpu::Queue,
) {
    println!("Model Matrix maybe_changed");
    let model_matrix = transform.compute_model_matrix();
    let updated_uniforms = ModelUniform::new(model_matrix);

    queue.write_buffer(
        &mesh.data.model_uniform_buffer,
        0,
        bytemuck::bytes_of(&updated_uniforms),
    );
}

#[system(for_each)]
#[filter(maybe_changed::<MeshComponent>())]
pub fn update_material(mesh: &MeshComponent, #[resource] queue: &wgpu::Queue) {
    println!("Material maybe_changed");
    for submesh in mesh.data.submeshes.iter() {
        let material = &submesh.material;
        if let Some(buffer) = &material.material_uniform_buffer {
            let updated_uniforms = crate::MaterialUniform {
                color: material.color.to_owned().into(),
                roughness: material.roughness,
                metallic: material.metallic,
                roughness_use_texture: material.roughness_use_texture as u32,
                metallic_use_texture: material.metallic_use_texture as u32,
                color_use_texture: material.color_use_texture as u32,
                ..Default::default()
            };
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&updated_uniforms));
        }
    }
}

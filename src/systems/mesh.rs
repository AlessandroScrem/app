use wgpu::IndexFormat;

use crate::{
    GlobalModelComponent, MeshComponent, TransformComponent,
    assets::material_manager::MaterialManager,
    entities::EntityRawU64,
    renderer::{
        gpu_manager::GPUResourceManager,
        gpu_renderer::DepthTexture,
        hdr_frame::{HdrFrame, IDTexture},
        pipeline_manager::{PipelineKind, PipelineManager},
        uniform::{MaterialUniform, ModelUniform},
        skybox_manager::SkyboxManager,
    },
};

use legion::{world::SubWorld, *};
use std::sync::Arc;

#[system]
#[read_component(MeshComponent)]
#[read_component(TransformComponent)]
pub fn render_mesh(
    world: &mut SubWorld,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] material_manager: &MaterialManager,
    #[resource] depth_texture: &DepthTexture,
    #[resource] hdr_texture: &HdrFrame,
    #[resource] entity_id_texture: &IDTexture,
    #[resource] skybox_manager: &SkyboxManager,
) {
    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };

    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Mesh Render Pass"),
        color_attachments: &[
            Some(wgpu::RenderPassColorAttachment {
                view: &hdr_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &entity_id_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }),
        ],
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
    renderpass.set_bind_group(0, &gpu_resource_manager.per_frame_bind_group, &[]);
    renderpass.set_bind_group(3, skybox_manager.get_ibl_bindgroup(), &[]);

    let mut mesh_query = <(&MeshComponent, &TransformComponent)>::query();
    for (mesh, _) in mesh_query.iter(world) {
        renderpass.set_bind_group(2, &mesh.data.model_bind_group, &[]);

        for submesh in mesh.data.submeshes.iter() {
            let vertex_buffer = submesh.vertex_buffer.as_ref().unwrap();
            let index_buffer = submesh.index_buffer.as_ref().unwrap();
            let index_count = submesh.index_count as u32;
            let material = material_manager.get(&submesh.material);

            renderpass.set_bind_group(1, &material.bind_group, &[]);

            renderpass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
            renderpass.set_vertex_buffer(0, vertex_buffer.slice(..));
            renderpass.draw_indexed(0..index_count, 0, 0..1);
        }
    }
}

#[system(for_each)]
#[filter(maybe_changed::<MeshComponent>())]
pub fn update_material_system_to_gpu(
    mesh: &MeshComponent,
    #[resource] queue: &wgpu::Queue,
    #[resource] material_manager: &MaterialManager,
) {
    // println!("Material maybe_changed");
    for submesh in mesh.data.submeshes.iter() {
        let material = material_manager.get(&submesh.material);
        let buffer = &material.uniform_buffer;
        let updated_uniforms =  MaterialUniform::from(&material.material_pbr);
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&updated_uniforms));
    }
}

#[system]
#[read_component(GlobalModelComponent)]
#[read_component(MeshComponent)]
pub fn update_model_uniforms_to_gpu(world: &mut SubWorld, #[resource] queue: &wgpu::Queue) {
    let mut uniforms_query = <(Entity, &MeshComponent, &GlobalModelComponent)>::query();
    for (entity, mesh, global_model) in uniforms_query.iter(world) {
        let mut model_uniform = ModelUniform::new(global_model.mat);
        model_uniform.entity_id = entity.as_raw_u64();
        queue.write_buffer(
            &mesh.data.model_uniform_buffer,
            0,
            bytemuck::bytes_of(&model_uniform),
        );
    }
}

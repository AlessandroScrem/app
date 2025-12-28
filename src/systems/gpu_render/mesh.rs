use wgpu::IndexFormat;

use crate::{
    MeshComponent,
    assets::{material_manager::MaterialManager, mesh_manager::MeshManager},
    renderer::{
        gpu_manager::GPUResourceManager,
        pipeline_manager::{PipelineKind, PipelineManager},
        skybox_manager::SkyboxManager,
    },
};

use legion::{world::SubWorld, *};

#[system]
#[read_component(MeshComponent)]
pub fn render_mesh(
    world: &mut SubWorld,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_manager: &GPUResourceManager,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] mesh_manager: &MeshManager,
    #[resource] material_manager: &MaterialManager,
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
                view: &gpu_manager.hdr_frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &gpu_manager.entity_id_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            }),
        ],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &gpu_manager.depth_view,
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
    renderpass.set_bind_group(0, &gpu_manager.per_frame_bind_group, &[]);
    renderpass.set_bind_group(3, skybox_manager.get_ibl_bindgroup(), &[]);

    let mut mesh_query = <&MeshComponent>::query();
    for mesh in mesh_query.iter(world) {
        let uniform_bind_group = mesh_manager.get_model_bindgroup(mesh.handle);
        renderpass.set_bind_group(2, uniform_bind_group, &[]);

        let vertex_buffer = mesh_manager.get_vertexbuffer(mesh.handle);
        let index_buffer = mesh_manager.get_indexbuffer(mesh.handle);
        let index_count = mesh_manager.get_indexcount(mesh.handle);
        let material = material_manager.get(mesh_manager.get_material(mesh.handle));

        renderpass.set_bind_group(1, &material.bind_group, &[]);

        renderpass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        renderpass.set_vertex_buffer(0, vertex_buffer.slice(..));
        renderpass.draw_indexed(0..index_count, 0, 0..1);
    }
}

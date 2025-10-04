use cgmath::num_traits::clamp;
use wgpu::IndexFormat;

use crate::{
    entities::EntityRawU64, renderer::{
        gpu_manager::GPUResourceManager,
        gpu_renderer::{DepthTexture, PickBuffer, PickPoint},
        hdr_frame::{HdrFrame, IDTexture},
        pipeline_manager::{PipelineKind, PipelineManager},
    }, MeshComponent, TransformComponent
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
    #[resource] entity_id_texture: &IDTexture,
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
    model_uniform: &mut ModelUniform,
    mesh: &MeshComponent,
    entity: &Entity,
    #[resource] queue: &wgpu::Queue,
) {
    println!("Model Matrix maybe_changed");
    let model_matrix = transform.compute_model_matrix();
    let mut updated_uniforms = ModelUniform::new(model_matrix);
    updated_uniforms.entity_id = entity.as_raw_u64();
    *model_uniform = updated_uniforms;

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
            let updated_uniforms = crate::renderer::uniform::MaterialUniform {
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

#[system]
pub fn read_entity_id(
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] entity_id_texture: &IDTexture,
    #[resource] readback_pixel_buffer: &mut PickBuffer,
    #[resource] point: &PickPoint,
) {
    
    let aligned_bytes_per_row = 256; // minimo richiesto
    let size = entity_id_texture._texture.size();
    let x = clamp(point.x, 0, size.width - 1);
    let y = clamp(point.y, 0, size.height -1);

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &entity_id_texture._texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback_pixel_buffer.current,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(aligned_bytes_per_row),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
}

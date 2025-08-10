use wgpu::IndexFormat;

use crate::{
    renderer::{gpu_renderer::DepthTexture, pipeline_manager::PipelineManager},
    resources::gpu_manager::GPUResourceManager,
};

pub fn create() -> impl legion::systems::Runnable {
    use crate::assets::mesh;
    use legion::IntoQuery;
    use legion::Read;
    use legion::SystemBuilder;
    use std::sync::Arc;

    SystemBuilder::new("render mesh")
        .read_resource::<wgpu::TextureView>()
        .write_resource::<wgpu::CommandEncoder>()
        .read_resource::<Arc<GPUResourceManager>>()
        .read_resource::<PipelineManager>()
        .read_resource::<DepthTexture>()
        .with_query(<Read<Arc<mesh::Mesh>>>::query())
        .build(
            |_,
             world,
             (frame_view, encoder, gpu_resource_manager, pipeline_manager, depth_texture),
             mesh_query| {

 
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
                    let texture_bind_group = map.get("texture").unwrap();

                    renderpass.set_pipeline(render_pipeline);
                    renderpass.set_bind_group(0, camera_bind_group, &[]);
                    renderpass.set_bind_group(1, texture_bind_group, &[]);

                    for mesh in mesh_query.iter(world) {
                        for submesh in mesh.submeshes.iter() {
                            let vertex_buffer = submesh.vertex_buffer.as_ref().unwrap();
                            let index_buffer = submesh.index_buffer.as_ref().unwrap();
                            let index_count = submesh.index_count as u32;

                            renderpass
                                .set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
                            renderpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                            renderpass.draw_indexed(0..index_count, 0, 0..1);
                        }
                    }
            },
        )
}

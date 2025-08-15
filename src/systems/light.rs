use std::sync::Arc;

use crate::{
    LightComponent,
    renderer::{
        gpu_renderer::DepthTexture, pipeline_manager::PipelineManager, uniform::ModelUniform,
    },
    resources::gpu_manager::GPUResourceManager,
};
use legion::*;


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];
    pub fn get_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[system(for_each)]
pub fn light(
    light: &LightComponent,
    #[resource] frame_view: &wgpu::TextureView,
    #[resource] encoder: &mut wgpu::CommandEncoder,
    #[resource] gpu_resource_manager: &Arc<GPUResourceManager>,
    #[resource] pipeline_manager: &PipelineManager,
    #[resource] depth_texture: &DepthTexture,
    #[resource] device: &wgpu::Device,
) {
    use wgpu::util::DeviceExt;
    use cgmath::{Matrix4, Vector3};

    // Vertex e index buffer statici (potresti spostare fuori dal system)
    let vertices = [
        Vertex { position: [-0.5, -0.5, 0.0] },
        Vertex { position: [ 0.5, -0.5, 0.0] },
        Vertex { position: [ 0.5,  0.5, 0.0] },
        Vertex { position: [-0.5,  0.5, 0.0] },
    ];
    let indices: &[u16] = &[0, 1, 2, 2, 3, 0];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Light Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Light Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    // Model matrix con scala e posizionamento
    let scale = Matrix4::from_scale(0.2);
    let translation = Matrix4::from_translation(Vector3::new(
        light.data.position[0],
        light.data.position[1],
        light.data.position[2],
    ));
    let model_matrix = (translation * scale).into();

    // Uniform buffer per la model matrix
    let model_uniform = ModelUniform::new(model_matrix);
    let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Light Model Buffer"),
        contents: bytemuck::bytes_of(&model_uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let model_layout = gpu_resource_manager.get_layout("model").unwrap();
    let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Light Model BindGroup"),
        layout: &model_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: model_buffer.as_entire_binding(),
        }],
    });

    // Camera bind group
    let camera_bind_group = {
        let map = gpu_resource_manager.bind_groups.lock().unwrap();
        map.get("camera").unwrap().clone()
    };

    // Render pass
    let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Light Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: frame_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_texture.0,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    let pipeline = pipeline_manager.get_render_pipeline("light").unwrap();

    renderpass.set_pipeline(&pipeline);
    renderpass.set_bind_group(0, &camera_bind_group, &[]);
    renderpass.set_bind_group(1, &model_bind_group, &[]);
    renderpass.set_vertex_buffer(0, vertex_buffer.slice(..));
    renderpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    renderpass.draw_indexed(0..6, 0, 0..1);
}


#[system(for_each)]
pub fn update_trnsform(
    light: &LightComponent,
    #[resource] queue: &wgpu::Queue,
    #[resource] gpu_manager: &Arc<GPUResourceManager>,
) {
    queue.write_buffer(
        &gpu_manager.light_uniform_buffer,
        0,
        bytemuck::bytes_of(&light.data),
    );
}

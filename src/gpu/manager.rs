use super::*;

use crate::renderer::uniform::{CameraUniform, GlobalUniform};
use crate::gpu::ibl::GpuIbl;

pub struct GpuManager {
    layout_cache: BindgroupLayoutCache,
    framebuffer_cache: FramebufferCache,
    buffer_cache: BufferCache,
    bindgroup_cache: BindgroupCache,
}

impl GpuManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Self {
        let layout_cache = BindgroupLayoutCache::new(device);
        let buffer_cache = BufferCache::new(device);
        let framebuffer_cache = FramebufferCache::new(device, &layout_cache, width, height);
        let bindgroup_cache = BindgroupCache::new(
            device,
            queue,
            &buffer_cache,
            &framebuffer_cache,
            &layout_cache,
        );

        Self {
            layout_cache,
            framebuffer_cache,
            buffer_cache,
            bindgroup_cache,
        }
    }

    pub fn get_bindgroup_layout(&self, kind: BindgroupLayoutKind) -> &wgpu::BindGroupLayout {
        self.layout_cache.get(kind)
    }

    pub fn get_framebuffer_view(&self, kind: FramebufferKind) -> &wgpu::TextureView {
        self.framebuffer_cache.get_view(kind)
    }

    pub fn get_framebuffer_sampler(&self, kind: FramebufferKind) -> &wgpu::Sampler {
        self.framebuffer_cache.get_sampler(kind)
    }

    pub fn get_framebuffer_texture(&self, kind: FramebufferKind) -> &wgpu::Texture {
        self.framebuffer_cache.get_texture(kind)
    }
    pub fn get_framebuffer_bg(&self, kind: FramebufferKind) -> &wgpu::BindGroup {
        self.framebuffer_cache.get_bg(kind)
    }

    pub fn resize_frame(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.framebuffer_cache
            .resize(device, &self.layout_cache, width, height);
    }

    pub fn get_bindgroup(&self, kind: BindgroupKind) -> &wgpu::BindGroup {
        self.bindgroup_cache.get(kind)
    }
    pub fn get_buffer(&self, kind: BufferKind) -> &wgpu::Buffer {
        self.buffer_cache.get(kind)
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, uniform: &CameraUniform) {
        queue.write_buffer(
            self.get_buffer(BufferKind::Camera),
            0,
            bytemuck::bytes_of(uniform),
        );
    }

    pub fn update_globals(&self, queue: &wgpu::Queue, uniform: &GlobalUniform) {
        queue.write_buffer(
            self.get_buffer(BufferKind::Globals),
            0,
            bytemuck::bytes_of(uniform),
        );
    }

    pub fn get_ibl_skybox_bg(&self, blur_flag: bool) -> &wgpu::BindGroup {
        if blur_flag {
            &self.bindgroup_cache.get(BindgroupKind::SkyboxBlur)
        } else {
            &self.bindgroup_cache.get(BindgroupKind::Skybox)
        }
    }


    pub fn sync_ibl(
        &mut self,
        ibl: &Option<GpuIbl>,
        device: &wgpu::Device,
    ) {
        if let Some(ibl) = ibl {
            println!("update Ibl");

            let entries = ibl.get_bindgroup_entry();
            let bg = create_bindgroup(
                device,
                BindgroupLayoutKind::PbrMaps,
                &self.layout_cache,
                &self.framebuffer_cache,
                &entries,
            );
            self.update_bindgroup(BindgroupKind::PbrMap, bg);

            let entries = ibl.get_skybox_bindgroup_entry();
            let bg = create_bindgroup(
                device,
                BindgroupLayoutKind::Skybox,
                &self.layout_cache,
                &self.framebuffer_cache,
                &entries,
            );
            self.update_bindgroup(BindgroupKind::Skybox, bg);

            let entries = ibl.get_skybox_bindgroup_entry_blur();
            let bg = create_bindgroup(
                device,
                BindgroupLayoutKind::Skybox,
                &self.layout_cache,
                &self.framebuffer_cache,
                &entries,
            );
            self.update_bindgroup(BindgroupKind::SkyboxBlur, bg);
        }
    } 

    //mutables
    fn update_bindgroup(&mut self, kind: BindgroupKind, bg: wgpu::BindGroup) {
        *self.bindgroup_cache.get_mut(kind) = bg;
    }
}

fn create_bindgroup(
    device: &wgpu::Device,
    layout: BindgroupLayoutKind,
    layout_cache: &BindgroupLayoutCache,
    framebuffer_cache: &FramebufferCache,
    entries: &Vec<wgpu::BindGroupEntry>,
) -> wgpu::BindGroup {
    let (label, all_entries) = match layout {
        BindgroupLayoutKind::PbrMaps => {
            let hdr_t_sampler = framebuffer_cache.get_sampler(FramebufferKind::OpaqueWithMips);
            let hdr_t_view = framebuffer_cache.get_view_mips(FramebufferKind::OpaqueWithMips);

            let mut e = entries.clone();
            e.extend([
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(hdr_t_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(hdr_t_view),
                },
            ]);
            (Some("Ibl Bindgroup"), e)
        }
        BindgroupLayoutKind::Skybox => (Some("Skybox bind_group"), entries.clone()),
        _ => unimplemented!("Layout kind unimplemented for bindgroup creation"),
    };

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label,
        layout: layout_cache.get(layout),
        entries: &all_entries,
    })
}


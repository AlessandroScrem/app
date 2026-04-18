use super::*;

use crate::uniform::{CameraUniform, GlobalUniform};

pub struct GpuManager {
    layout_cache: BindgroupLayoutCache,
    framebuffer_cache: FramebufferCache,
    buffer_cache: BufferCache,
    bindgroup_cache: BindgroupCache,
}

impl GpuManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> Self {
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

    pub fn get_layout(&self, kind: BindgroupLayoutKind) -> &wgpu::BindGroupLayout {
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

    //mutables
    pub fn update_bindgroup(&mut self, kind: BindgroupKind, bg: wgpu::BindGroup) {
        *self.bindgroup_cache.get_mut(kind) = bg;
    }

    // CHANGE ME!
    pub fn update_pbrmap_bind_group(
        &mut self,
        device: &wgpu::Device,
        entries: &Vec<wgpu::BindGroupEntry>,
    ) {
        let bg = create_bindgroup(
            device,
            BindgroupLayoutKind::PbrMaps,
            &self.layout_cache,
            &self.framebuffer_cache,
            entries,
        );
        self.update_bindgroup(BindgroupKind::PbrMap, bg);
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
        _ => unimplemented!("Layout kind unimplemented for bindgroup creation"),
    };

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label,
        layout: layout_cache.get(layout),
        entries: &all_entries,
    })
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn should_contain_static_textures() {
//         let (device, queue) = test_utils::get_device_and_queue();
//         let gpu_mgr = GpuManager::new(&device, &queue, 32, 32);

//         let _texture = gpu_mgr.static_textures.lightbulb;

//         // #[cfg(feature = "save_tests")]
//         test_utils::save_texture(device, queue, "texture.png", &_texture, 0).unwrap()
//     }
// }

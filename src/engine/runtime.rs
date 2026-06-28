use std::sync::Arc;

use super::RuntimeEvent;
use crate::app::{Application, HandlesPicking, HasUi, RuntimeApp};
use crate::assets::IblAsset;
use crate::assets::asset_manager::AssetManager;
use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{
    BindgroupLayoutKind, FramebufferKind, GpuCache, GpuContext, GpuInternalCounters, GpuManager, GpuSurface, HasGpuStats, IblManager, ShadowManager,
};
use crate::input::Input;
use crate::picking::PickObject;
use crate::prelude::info;
use crate::renderer::FrameBuilder;
use crate::renderer::ImguiRender;
use crate::renderer::SceneRenderer;
use crate::renderer::scene_renderer::SceneRenderContext;
use crate::ui::{InternalCounter, UiTexture};
use crate::ui::UiLayer;
use crate::ui::UiRuntimeContext;
use winit::{event::Event, window::Window};

impl InternalCounter for GpuCache {
    fn internal_counter(&self) -> GpuInternalCounters {
        GpuInternalCounters {
            textures: self.textures.get_stats(),
            meshes: self.mesh.get_stats(),
            materials: self.material.get_stats(),
        }
    }
}

pub struct RunningApp {
    pub window: Arc<Window>,
    pub gpu_context: GpuContext,
    pub gpu_surface: GpuSurface,
    pub gpu_cache: GpuCache,
    pub gpu_manager: GpuManager,
    pub ibl_manager: IblManager,
    pub pipeline_manager: PipelineManager,
    pub shadow_manager: ShadowManager,

    pub uilayer: UiLayer,
    pub input: Input,

    pub events: Vec<RuntimeEvent>,
    pub scene_renderer: SceneRenderer,
    pub pickobject: PickObject,
    pub imgui_render: ImguiRender,
}

impl RunningApp {
    pub fn handle_winit_event(&mut self, event: &Event<()>) {
        // Handle Imgui platform events
        self.uilayer.handle_event(&self.window, event);

        // Handle Input
        match event {
            Event::WindowEvent { .. } | Event::DeviceEvent { .. } => {
                if !self.uilayer.want_capture_mouse() {
                    self.input.update_events(&event);
                }
            }
            _ => {}
        }
    }

    pub fn tick<A: RuntimeApp>(&mut self, app: &mut A) {
        let events = std::mem::take(&mut self.events);
        for event in events {
            self.handle_runtime_event(app, event);
        }

        self.update_app_hover(app);
        let input = self.input.clone();
        app.update(&input);
        self.sync_gpu_assets(app.asset_mgr_mut());
        self.imgui_render.sync_imgui_framebuffer(&self.gpu_context, self.gpu_manager.get_framebuffers());

        self.update_app_ui(app);

        self.render(app);

        // Clear Input
        self.input.clear();
    }

    pub fn sync_gpu_assets(&mut self, asset_mgr: &mut AssetManager) {
        use crate::assets::TextureId;
        use crate::assets::asset_manager::AssetEventKind;
        use crate::assets::material_asset::MaterialAsset;
        use crate::assets::mesh_asset::MeshAsset;
        use crate::assets::texture_asset::{TextureAsset, TextureDesc};
        use crate::assets::texture_upload::load_cpu_textures_par;
        use crate::gpu::GpuMaterial;
        use crate::gpu::GpuMesh;
        use crate::gpu::texture::GpuTextureBuilder;

        let Self {
            gpu_context,
            ibl_manager,
            gpu_manager,
            ..
        } = self;

        let device = &gpu_context.device;
        let queue = &gpu_context.queue;

        let texture_cache = &mut self.gpu_cache.textures;
        let material_cache = &mut self.gpu_cache.material;
        let mesh_cache = &mut self.gpu_cache.mesh;

        let grouped = asset_mgr.drain_grouped_events();

        grouped.process_type::<TextureAsset, _>(|kind, events| match kind {
            AssetEventKind::Created => {
                info!("loading textures len {}", events.len());

                let jobs: Vec<(TextureId, TextureDesc)> = events
                    .iter()
                    .filter_map(|ev| {
                        asset_mgr
                            .get::<TextureAsset>(ev.id)
                            .map(|asset| (ev.id, asset.desc.clone()))
                    })
                    .collect();

                let cpu_textures = load_cpu_textures_par(jobs);

                for (id, data) in cpu_textures {
                    let texture = GpuTextureBuilder::from_cpu(data).build(device, Some(queue));
                    texture_cache.insert(id, texture);
                }
            }

            AssetEventKind::Updated => {}

            AssetEventKind::Removed => {
                info!("Removed texture len {}", events.len());
                events.iter().for_each(|ev| {
                    texture_cache.remove(ev.id);
                });
            }
            _ => {}
        });

        grouped.process_type::<IblAsset, _>(|kind, events| match kind {
            AssetEventKind::Created | AssetEventKind::Updated => {
                info!("loading/Updating Ibl len {}", events.len());

                events
                    .iter()
                    .filter_map(|ev| asset_mgr.get::<IblAsset>(ev.id).map(|asset| (ev.id, asset)))
                    .for_each(|(_id, asset)| {
                        let hdr = texture_cache.get(asset.hrd_id);
                        ibl_manager.create(hdr, device, queue);
                    });

                gpu_manager.sync_ibl(&ibl_manager.ibl, device);
            }

            AssetEventKind::Removed => {}
            _ => {}
        });

        grouped.process_type::<MaterialAsset, _>(|kind, events| match kind {
            AssetEventKind::Created => {
                info!("loading material len {}", events.len());
                events
                    .iter()
                    .filter_map(|ev| {
                        asset_mgr
                            .get::<MaterialAsset>(ev.id)
                            .map(|asset| (ev.id, asset))
                    })
                    .for_each(|(id, asset)| {
                        let material_layout =
                            gpu_manager.get_bindgroup_layout(BindgroupLayoutKind::Material);
                        let gpu_material =
                            GpuMaterial::new(&texture_cache, &asset.desc, device, material_layout);
                        material_cache.insert(id, gpu_material);
                    });
            }

            AssetEventKind::Updated => {
                info!("updating material len {}", events.len());
                events
                    .iter()
                    .filter_map(|ev| {
                        asset_mgr
                            .get::<MaterialAsset>(ev.id)
                            .map(|asset| (ev.id, &asset.desc))
                    })
                    .for_each(|(id, desc)| {
                        material_cache.update(&id, |gpu_mat| {
                            gpu_mat.update_uniform(queue, desc);
                        });
                    });
            }

            AssetEventKind::Removed => {
                info!("Removed material len {}", events.len());
                events.iter().for_each(|ev| {
                    material_cache.remove(ev.id);
                });
            }
            _ => {}
        });

        grouped.process_type::<MeshAsset, _>(|kind, events| match kind {
            AssetEventKind::Created => {
                info!("loading meshes len {}", events.len());
                events
                    .iter()
                    .filter_map(|ev| {
                        asset_mgr
                            .get::<MeshAsset>(ev.id)
                            .map(|asset| (ev.id, asset))
                    })
                    .for_each(|(id, asset)| {
                        let gpu_mesh =
                            GpuMesh::new(device, &asset.desc.vertices, &asset.desc.indices);
                        mesh_cache.insert(id, gpu_mesh);
                    });
            }

            AssetEventKind::Updated => {}

            AssetEventKind::Removed => {
                info!("Removed meshes len {}", events.len());
                events.iter().for_each(|ev| {
                    mesh_cache.remove(ev.id);
                });
            }
            _ => {}
        });

        // If texture cache is changed -> sync imgui texture
        grouped.process_type::<TextureAsset, _>(|_, _| {
            self.imgui_render
                .sync_imgui_texture(&self.gpu_context, &mut self.gpu_cache.textures);
        });
    }

    fn update_app_hover<A: HandlesPicking>(&mut self, app: &mut A) {
        if self.input.is_cursor_moved() {
            let hovered = self.pickobject.poll_readback(&self.gpu_context.device);
            app.set_hovered(hovered);
        }
    }

    fn update_app_ui<A: HasUi>(&mut self, app: &mut A) {
        let RunningApp {
            window,
            uilayer,
            scene_renderer,
            imgui_render,
            ..
        } = self;

        let debug_texture_id = UiTexture::Framebuffer(FramebufferKind::ShadowMapRgba);

        let context = UiRuntimeContext {
            window: window.as_ref(),
            uilayer,
            texture_resolver: imgui_render,
            gpu_counters: self.gpu_cache.internal_counter(),
            frame_stats: scene_renderer.get_render_stats(),
            debug_texture: Some(debug_texture_id),
        };

        app.update_ui(context);
    }

    fn render<A: Application>(&mut self, app: &A) {
        let mut encoder = self.gpu_context.create_encoder();

        if let Some(frame) = self.gpu_surface.get_frame() {
            let target = frame.texture.create_view(&Default::default());
            let size = (
                self.gpu_surface.get_config().width,
                self.gpu_surface.get_config().height,
            );
            let render_data = app.render_data();
            {
                let RunningApp {
                    scene_renderer,
                    gpu_context,
                    gpu_manager,
                    pipeline_manager,
                    gpu_cache,
                    input,
                    pickobject,
                    ..
                } = self;

                let frame = FrameBuilder::build(
                    render_data.world,
                    render_data.asset_mgr,
                    render_data.selected,
                    pickobject,
                    input,
                    render_data.globals,
                );

                let mut context = SceneRenderContext {
                    gpu_context,
                    gpu_manager,
                    pipeline_manager,
                    gpu_cache,
                    pickobject,
                };

                scene_renderer.render(
                    &mut context,
                    &mut encoder,
                    &target,
                    size,
                    &frame,
                    render_data.camera,
                    render_data.globals,
                    render_data.selected,
                );
            }

            self.imgui_render.render(
                self.uilayer.get_draw_data(),
                &mut encoder,
                &target,
                &self.gpu_context.device,
                &self.gpu_context.queue,
            );

            self.gpu_context.queue.submit([encoder.finish()]);
            frame.present();
        }
    }

    fn handle_runtime_event<A: Application>(&mut self, app: &mut A, event: RuntimeEvent) {
        match event {
            RuntimeEvent::Resize { width, height } => {
                if width == 0 || height == 0 {
                    return;
                }
                self.gpu_manager
                    .resize_frame(&self.gpu_context.device, width, height);

                self.gpu_surface
                    .resize_frame(&self.gpu_context.device, width, height);
                app.on_resize(width, height);
            }
            RuntimeEvent::CloseRequested => {
                app.on_close();
            }
            RuntimeEvent::DroppedFile(path) => {
                app.on_drop(path);
            }
        }
    }
}

use std::sync::Arc;

use super::RuntimeEvent;
use crate::UiLayer;
use crate::app::{Application, HandlesPicking, HasUi, RuntimeApp};
use crate::gpu::caches::internalcounter::HasGpuStats;
use crate::gpu::ibl_asset::IblAsset;
use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{GpuCache, GpuContext, GpuInternalCounters, GpuManager, GpuSurface, IblManager};
use crate::input::Input;
use crate::picking::PickObject;
use crate::prelude::*;
use crate::renderer::FrameBuilder;
use crate::renderer::ImguiRender;
use crate::renderer::gpu_sync::GpuSync;
use crate::renderer::scene_renderer::SceneRenderContext;
use crate::ui::UiRuntimeContext;
use crate::ui::traits::InternalCounter;
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

    pub uilayer: UiLayer,
    pub timer: Timer,
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
        self.update_app_ui(app);

        self.render(app);

        // Clear Input
        self.input.clear();
    }

    pub fn sync_gpu_assets(&mut self, asset_mgr: &mut GlobalAssetManager) {
        use crate::assets::TextureId;
        use crate::assets::global_asset_manager::AssetEventKind;
        use crate::assets::material_asset::MaterialAsset;
        use crate::assets::mesh_asset::MeshAsset;
        use crate::assets::texture_asset::{TextureAsset, TextureDesc};
        use crate::assets::texture_upload::load_cpu_textures_par;
        use crate::gpu::material::GpuMaterial;
        use crate::gpu::mesh::GpuMesh;
        use crate::gpu::texture::GpuTextureBuilder;

        use std::any::TypeId;

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

        // Texture Create events
        if let Some(tex_created) =
            grouped.get(&(TypeId::of::<TextureAsset>(), AssetEventKind::Created))
        {
            println!("loading textures len {}", tex_created.len());
            let jobs: Vec<(TextureId, TextureDesc)> = tex_created
                .iter()
                .filter_map(|ev| {
                    asset_mgr
                        .get::<TextureAsset>(ev.id)
                        .map(|asset| (ev.id, asset.desc.clone()))
                })
                .collect();

            let cpu_textures = load_cpu_textures_par(jobs);

            for (id, data) in cpu_textures.into_iter() {
                let texture = GpuTextureBuilder::from_cpu(data).build(device, Some(queue));
                texture_cache.insert(id, texture);
            }
        }

        // Ibl Create events
        if let Some(ibl_created) = grouped.get(&(TypeId::of::<IblAsset>(), AssetEventKind::Created))
        {
            println!("loading Ibl len {}", ibl_created.len());

            ibl_created
                .iter()
                .filter_map(|ev| asset_mgr.get::<IblAsset>(ev.id).map(|asset| (ev.id, asset)))
                .for_each(|(_id, asset)| {
                    let hdr = texture_cache.get(asset.hrd_id);
                    ibl_manager.create(hdr, device, queue);
                });
            
            gpu_manager.sync_ibl(&ibl_manager.ibl, device);
        }

        // Material Create events
        if let Some(mat_created) =
            grouped.get(&(TypeId::of::<MaterialAsset>(), AssetEventKind::Created))
        {
            println!("loading material len {}", mat_created.len());
            mat_created
                .iter()
                .filter_map(|ev| {
                    asset_mgr
                        .get::<MaterialAsset>(ev.id)
                        .map(|asset| (ev.id, asset))
                })
                .for_each(|(id, asset)| {
                    let material_layout =
                        gpu_manager.get_bindgroup_layout(gpu::BindgroupLayoutKind::Material);
                    let gpu_material =
                        GpuMaterial::new(&texture_cache, &asset.desc, device, material_layout);
                    material_cache.insert(id, gpu_material);
                });
        }

        // Mesh Create events
        if let Some(mesh_created) =
            grouped.get(&(TypeId::of::<MeshAsset>(), AssetEventKind::Created))
        {
            println!("loading material len {}", mesh_created.len());
            mesh_created
                .iter()
                .filter_map(|ev| {
                    asset_mgr
                        .get::<MeshAsset>(ev.id)
                        .map(|asset| (ev.id, asset))
                })
                .for_each(|(id, asset)| {
                    let gpu_mesh = GpuMesh::new(device, &asset.desc.vertices, &asset.desc.indices);
                    mesh_cache.insert(id, gpu_mesh);
                });
        }

        self.timer
            .trigger_every(std::time::Duration::from_secs(1), || {
                self.imgui_render
                    .sync_imgui_texture(&self.gpu_context, &mut self.gpu_cache);
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

        let context = UiRuntimeContext {
            window: window.as_ref(),
            uilayer,
            texture_resolver: imgui_render,
            gpu_counters: self.gpu_cache.internal_counter(),
            frame_stats: scene_renderer.get_render_stats(),
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

                // GpuSync::sync_caches(gpu_cache, gpu_context, gpu_manager, render_data.asset_mgr);

                // GpuSync::update_meshes_materials_to_gpu(
                //     &gpu_context.queue,
                //     &gpu_cache,
                //     render_data.asset_mgr,
                //     &frame,
                // );

                GpuSync::update_lights_to_gpu(&gpu_context.queue, &gpu_manager, &frame);

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
                // self.gpu_manager
                //     .update_ibl_bind_group(&self.gpu_context.device);

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

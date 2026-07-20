use std::sync::Arc;

use super::RuntimeEvent;
use crate::app::domain::events::CameraEvent::{CameraOrbit, CameraPan, CameraZoom};
use crate::app::domain::events::DomainEvent;
use crate::app::domain::events::SelectionEvent::{Hovered, SelectHovered};
use crate::app::{Application, RuntimeApp};
use crate::assets::IblAsset;
use crate::assets::asset_manager::AssetManager;
use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{
    BindgroupLayoutKind, GpuCache, GpuContext, GpuInternalCounters, GpuManager, GpuSurface,
    HasGpuStats, IblManager, ShadowManager,
};
use crate::input::{Input, KeyButton};
use crate::picking::PickObject;
use crate::prelude::info;
use crate::renderer::FrameBuilder;
use crate::renderer::ImguiRender;
use crate::renderer::SceneRenderer;
use crate::renderer::framebuilder::PickingData;
use crate::renderer::scene_renderer::SceneRenderContext;
use crate::ui::InternalCounter;
use crate::ui::UiLayer;
use winit::{event::Event, window::Window};

impl InternalCounter for RunningApp {
    fn internal_counter(&self) -> GpuInternalCounters {
        GpuInternalCounters {
            textures: self.gpu_cache.textures.get_stats(),
            meshes: self.gpu_cache.mesh.get_stats(),
            materials: self.gpu_cache.material.get_stats(),
            shadows: self.shadow_manager.get_stats(),
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

    fn handle_input<A: RuntimeApp>(&mut self, app: &mut A) {
        use crate::input::MouseButton;
        let input = &self.input;

        // handle hovered entity_id
        if self.input.is_cursor_moved() {
            let hovered = self.pickobject.poll_readback(&self.gpu_context.device);
            app.push_event(DomainEvent::Selection(Hovered(hovered)));
        }

        // handle selection: hovered -> selected
        if input.is_mouse_button_pressed(MouseButton::Left) && input.is_key_down(KeyButton::Alt) {
            app.push_event(DomainEvent::Selection(SelectHovered));
        }

        // handle camera -------
        if input.is_mouse_button_down(MouseButton::Left) {
            let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
            app.push_event(DomainEvent::Camera(CameraOrbit(delta.0, delta.1)));
        }

        if input.is_mouse_button_down(MouseButton::Middle) {
            let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
            app.push_event(DomainEvent::Camera(CameraPan(delta.0, delta.1)));
        }

        if let Some(delta) = input.mouse_wheel_movement {
            app.push_event(DomainEvent::Camera(CameraZoom(delta.y)));
        }
        // --------------------
    }

    pub fn tick<A: RuntimeApp>(&mut self, app: &mut A) {
        self.handle_input(app);

        self.handle_runtime_events(app);

        // update app, maybe enqueue new runtime events.
        app.on_update(&mut self.events);

        self.sync_gpu_assets(app.asset_mgr_mut());

        // replace pbrmap & skybox bindgroups
        if self.gpu_manager.bindgroup_diry() {
            self.events.push(RuntimeEvent::UpdateIblMaps);
        }

        self.update_ui(app);

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
                gpu_manager.set_bindgroup_diry();
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

        // If texture cache is changed -> call sync imgui texture
        grouped.process_type::<TextureAsset, _>(|_, _| {
            self.events.push(RuntimeEvent::SyncImguiTextures);
        });
    }

    fn update_ui<A: Application>(&mut self, app: &mut A) {
        // Main operation: update_ui and return domain events
        let frame_stats = self.scene_renderer.get_render_stats();
        let gpu_counters = self.internal_counter();
        let snapshot = app.get_scene_snapshot(&self.imgui_render, frame_stats, gpu_counters);
        let events = self.uilayer.build(&self.window, snapshot);
        for event in events {
            app.push_event(event);
        }
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

            // if self.input.is_cursor_moved() {
            //     self.pickobject.set_picking_coords((
            //         self.input.mouse_position.x as u32,
            //         self.input.mouse_position.y as u32,
            //     ));
            // }

            let picking_data =
                (self.input.is_cursor_moved() && self.pickobject.is_ready()).then(|| PickingData {
                    mouse_pos_x: self.input.mouse_position.x as u32,
                    mouse_pos_y: self.input.mouse_position.y as u32,
                });

            {
                let RunningApp {
                    scene_renderer,
                    gpu_context,
                    gpu_manager,
                    shadow_manager,
                    pipeline_manager,
                    gpu_cache,
                    pickobject,
                    ..
                } = self;


                let frame = FrameBuilder::build(
                    render_data.world,
                    render_data.asset_mgr,
                    render_data.selected,
                    picking_data,
                    render_data.globals,
                );

                let mut context = SceneRenderContext {
                    gpu_context,
                    gpu_manager,
                    shadow_manager,
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

    fn handle_runtime_events<A: Application>(&mut self, app: &mut A) {
        for event in self.events.drain(..) {
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
                RuntimeEvent::SetWindowTitle(title) => {
                    self.window.set_title(&title);
                    info!("Set Window title");
                }
                RuntimeEvent::SyncImguiTextures => {
                    self.imgui_render
                        .sync_imgui_texture(&self.gpu_context, &mut self.gpu_cache.textures);
                }
                RuntimeEvent::UpdateIblMaps => {
                    self.gpu_manager.replace_pbrmap_skybox_bindgroup(
                        self.ibl_manager.get_ibl(),
                        &self.shadow_manager,
                        &self.gpu_context.device,
                    );
                    self.imgui_render
                        .sync_imgui_shadowmap(&self.gpu_context, self.shadow_manager.get_rgba());
                }
            }
        }
    }
}

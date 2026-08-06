use std::collections::HashSet;
use std::sync::Arc;

use super::RuntimeEvent;
use crate::EntityRawU64;
use crate::app::Application;
use crate::app::application::AppRenderData;
use crate::app::domain::events::CameraEvent::{CameraOrbit, CameraPan, CameraZoom};
use crate::app::domain::events::DomainEvent::{Camera, Selection};
use crate::app::domain::events::SelectionEvent::{Hovered, SelectHovered, SelectIbl};
use crate::assets::asset_manager::AssetManager;
use crate::assets::{IblAsset, IblId, TextureId};
use crate::engine::engine::EventBus;
use crate::engine::request_mgr::{QueryResult, RequestManager};
use crate::gpu::gpu_readback::GpuReadback;
use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{
    BindgroupLayoutKind, GpuCache, GpuContext, GpuInternalCounters, GpuManager, GpuMaterialCache,
    GpuMeshCache, GpuSurface, GpuTextureCache, HasGpuStats, IblManager, ShadowManager,
};
use crate::input::{Input, KeyButton};
use crate::prelude::info;
use crate::renderer::FrameBuilder;
use crate::renderer::ImguiRender;
use crate::renderer::SceneRenderer;
use crate::renderer::scene_renderer::SceneRenderContext;
use crate::ui::{EditorInteraction, InternalCounter, UiLayer};
use legion::Entity;
use winit::{event::Event, window::Window};

impl InternalCounter for Runtime {
    fn internal_counter(&self) -> GpuInternalCounters {
        GpuInternalCounters {
            textures: self.gpu_cache.textures.get_stats(),
            meshes: self.gpu_cache.mesh.get_stats(),
            materials: self.gpu_cache.material.get_stats(),
            shadows: self.shadow_manager.get_stats(),
            ibl: self.ibl_manager.get_stats(),
        }
    }
}

pub struct Runtime {
    pub window: Arc<Window>,
    pub gpu_context: GpuContext,
    pub gpu_surface: GpuSurface,
    pub gpu_cache: GpuCache,
    pub gpu_manager: GpuManager,
    pub ibl_manager: IblManager,
    pub pipeline_manager: PipelineManager,
    pub shadow_manager: ShadowManager,
    pub req_mgr: RequestManager,

    pub uilayer: UiLayer,
    pub input: Input,
    pub scene_renderer: SceneRenderer,
    pub editor_interaction: EditorInteraction,
    pub imgui_render: ImguiRender,
    pub hdr_vec: Vec<(TextureId, IblId)>,
    pub wait_for_exit: bool,
}

impl Runtime {
    pub fn new(window: Arc<Window>) -> Self {
        // gpu resources
        let mut imgui_context = imgui::Context::create();
        let gpu_context = GpuContext::default();
        let gpu_surface = GpuSurface::new(
            gpu_context.adapter(),
            gpu_context.instance(),
            window.clone(),
        );
        let imgui_render = ImguiRender::new(
            &gpu_context.device,
            &gpu_context.queue,
            &window,
            &mut imgui_context,
            gpu_surface.get_config().format,
        );

        // gpu resources
        let texture_cache = GpuTextureCache::new(&gpu_context.device, &gpu_context.queue);

        let gpu_cache = GpuCache {
            textures: texture_cache,
            material: GpuMaterialCache::default(),
            mesh: GpuMeshCache::default(),
        };

        let (width, height) = (
            gpu_surface.get_config().width,
            gpu_surface.get_config().height,
        );

        let gpu_manager = GpuManager::new(&gpu_context.device, &gpu_context.queue, width, height);

        let shadow_manager = ShadowManager::new(&gpu_context.device);

        let ibl_manager = IblManager::new(&gpu_context.device, &gpu_context.queue);

        let pipeline_manager = PipelineManager::new(
            &gpu_context.device,
            &gpu_manager,
            gpu_surface.get_config().format,
        );
        //

        let scene_renderer = SceneRenderer::new();
        let uilayer = UiLayer::new(&window, imgui_context, gpu_context.get_adapter_string());

        Self {
            window: window.clone(),
            input: Input::new(),
            scene_renderer,
            editor_interaction: EditorInteraction::None,
            imgui_render,
            uilayer,
            gpu_context,
            gpu_surface,
            gpu_cache,
            gpu_manager,
            ibl_manager,
            pipeline_manager,
            shadow_manager,
            hdr_vec: Vec::new(),
            wait_for_exit: false,
            req_mgr: RequestManager::default(),
        }
    }
}

impl Runtime {
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

    pub fn handle_input(&mut self, bus: &mut EventBus) {
        use crate::input::MouseButton;
        let input = &self.input;

        if let Some(result) = self.req_mgr.poll() {
            match result {
                QueryResult::Pick(id) => {
                    let entity = id.map(Entity::from_raw_u64);
                    bus.send_domain(Selection(Hovered(entity)));
                }

                QueryResult::Selection(ids) => {
                    let selected: HashSet<u64> = ids.into_iter().collect();
                    println!("Selected {:?}", selected);
                }
            }
        }

        // handle hovered entity_id
        if self.input.is_cursor_moved() {
            // TODO: remove this.
            let gpu = GpuReadback::default();

            self.req_mgr.request_pick(
                &gpu,
                &self.gpu_context.device,
                &self.gpu_context.queue,
                &self
                    .gpu_manager
                    .get_framebuffer_texture(crate::gpu::FramebufferKind::EntityId),
                (
                    self.input.mouse_position.x as u32,
                    self.input.mouse_position.y as u32,
                ),
            );
        }

        // handle selection: hovered -> selected
        if input.is_mouse_button_pressed(MouseButton::Left) && input.is_key_down(KeyButton::Alt) {
            bus.send_domain(Selection(SelectHovered));
        }

        match self.editor_interaction {
            // handle start SelectionBox:
            EditorInteraction::None => {
                if input.is_mouse_button_pressed(MouseButton::Left)
                    && input.is_key_down(KeyButton::Control)
                {
                    let current = self.input.mouse_position;
                    let start = current;
                    self.editor_interaction = EditorInteraction::Selecting { start, current };
                }
            }
            EditorInteraction::Selecting { start, current: _ } => {
                // handle drag SelectionBox:
                if input.is_mouse_dragging(MouseButton::Left)
                    && input.is_key_down(KeyButton::Control)
                {
                    let current = self.input.mouse_position;
                    self.editor_interaction = EditorInteraction::Selecting { start, current };
                }
                // handle end SelectionBox:
                if input.is_mouse_button_released(MouseButton::Left) {
                    self.editor_interaction = EditorInteraction::None;
                    let current = self.input.mouse_position;

                    // pos must be origin coordinates from top left.
                    let pos = (start.x.min(current.x) as u32, start.y.min(current.y) as u32);
                    let width = (start.x - current.x).abs() as u32;
                    let height = (start.y - current.y).abs() as u32;

                    // TODO: remove this.
                    let gpu = GpuReadback::default();
                    self.req_mgr.request_selection(
                        &gpu,
                        &self.gpu_context.device,
                        &self.gpu_context.queue,
                        &self
                            .gpu_manager
                            .get_framebuffer_texture(crate::gpu::FramebufferKind::EntityId),
                            pos,
                        (
                            width,
                            height,
                        ),
                    );
                }
            }
        }

        // handle camera -------
        if input.is_mouse_dragging(MouseButton::Left) && input.any_key_down() {
            let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
            bus.send_domain(Camera(CameraOrbit(delta.0, delta.1)));
        }

        if input.is_mouse_dragging(MouseButton::Middle) {
            let delta = (input.mouse_delta.x as f64, input.mouse_delta.y as f64);
            bus.send_domain(Camera(CameraPan(delta.0, delta.1)));
        }

        if let Some(delta) = input.mouse_wheel_movement {
            bus.send_domain(Camera(CameraZoom(delta.y)));
        }
        // --------------------

        // Clear Input
        self.input.clear();
    }

    pub fn handle_runtime_events<A: Application>(&mut self, app: &mut A, bus: &mut EventBus) {
        let events = bus.drain_runtime();
        for event in events {
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
                    self.wait_for_exit = true;
                }
                RuntimeEvent::DroppedFile(path) => {
                    app.on_drop(path, bus);
                }
                RuntimeEvent::SetWindowTitle(title) => {
                    self.window.set_title(&title);
                    info!("Set Window title");
                }
                RuntimeEvent::SyncImguiTextures => {
                    self.imgui_render
                        .sync_imgui_texture(&self.gpu_context, &mut self.gpu_cache.textures);
                }
                RuntimeEvent::UpdateIblMaps(id) => {
                    self.gpu_manager.replace_pbrmap_skybox_bindgroup(
                        self.ibl_manager.get(&id),
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

impl Runtime {
    pub fn sync_gpu_assets(&mut self, asset_mgr: &mut AssetManager, bus: &mut EventBus) {
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
            AssetEventKind::Created => {
                info!("loading/Updating Ibl len {}", events.len());

                events
                    .iter()
                    .filter_map(|ev| asset_mgr.get::<IblAsset>(ev.id).map(|asset| (ev.id, asset)))
                    .for_each(|(ibl_id, asset)| {
                        if let Some(hdr) = texture_cache.get(asset.hrd_id) {
                            let gpu_ibl = ibl_manager.create(hdr, device, queue);
                            ibl_manager.insert(ibl_id, gpu_ibl);
                            self.hdr_vec.push((asset.hrd_id, ibl_id));
                            bus.send_domain(Selection(SelectIbl(ibl_id)));
                            bus.send_runtime(RuntimeEvent::UpdateIblMaps(ibl_id));
                        }
                    });
            }
            AssetEventKind::Updated => {}
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
            bus.send_runtime(RuntimeEvent::SyncImguiTextures);
        });
    }
}
impl Runtime {
    pub fn update_ui<A: Application>(&mut self, app: &mut A, bus: &mut EventBus) {
        let frame_stats = self.scene_renderer.get_render_stats();
        let gpu_counters = self.internal_counter();
        let snapshot =
            app.get_scene_snapshot(&self.imgui_render, frame_stats, gpu_counters, &self.hdr_vec);

        // Main operation: update_ui and push events
        self.uilayer
            .build(&self.window, snapshot, bus, &self.editor_interaction);
    }
}

impl Runtime {
    pub fn render(&mut self, render_data: &AppRenderData) {
        let mut encoder = self.gpu_context.create_encoder();

        if let Some(frame) = self.gpu_surface.get_frame() {
            let target = frame.texture.create_view(&Default::default());
            let size = (
                self.gpu_surface.get_config().width,
                self.gpu_surface.get_config().height,
            );

            {
                let Runtime {
                    scene_renderer,
                    gpu_context,
                    gpu_manager,
                    shadow_manager,
                    pipeline_manager,
                    gpu_cache,
                    ..
                } = self;

                let frame = FrameBuilder::build(
                    render_data.world,
                    render_data.asset_mgr,
                    render_data.selected,
                    render_data.globals,
                );

                let mut context = SceneRenderContext {
                    gpu_context,
                    gpu_manager,
                    shadow_manager,
                    pipeline_manager,
                    gpu_cache,
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
}

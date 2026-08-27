use super::RuntimeEvent;
use crate::EntityRawU64;
use crate::app::Application;
use crate::app::application::AppRenderData;
use crate::app::domain::events::CameraEvent::{CameraOrbit, CameraPan, CameraZoom};
use crate::app::domain::events::DomainEvent::{Camera, Selection};
use crate::app::domain::events::SelectionEvent::{Hovered, Select, SelectIbl};
use crate::assets::asset_manager::AssetManager;
use crate::assets::{IblAsset, IblId, TextureId};
use crate::editor::{EditorConnection, EditorStatisticsData};
use crate::engine::editor::EditorService;
use crate::engine::engine::EventBus;
use crate::engine::readback::{QueryResult, ReadbackManager};
use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{
    BindgroupLayoutKind, BufferKind, GpuCache, GpuContext, GpuInternalCounters, GpuManager,
    GpuMaterialCache, GpuMeshCache, GpuSurface, GpuTextureCache, HasGpuStats, IblManager,
    ShadowManager,
};
use crate::input::{Input, KeyButton};
use crate::prelude::info;
use crate::renderer::FrameData;
use crate::renderer::ImguiRender;
use crate::renderer::SceneRenderer;
use crate::renderer::framebuilder::{FrameBuilder, FrameTasks};
use crate::renderer::scene_renderer::SceneRenderContext;
use crate::renderer::uniform::{CameraUniform, GlobalUniform};
use crate::ui::{EditorInteraction, InternalCounter, UiLayer};
use legion::Entity;
use std::sync::Arc;
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
    pub readback: ReadbackManager,
    pub uilayer: UiLayer,
    pub input: Input,
    pub scene_renderer: SceneRenderer,
    pub editor_interaction: EditorInteraction,
    pub imgui_render: ImguiRender,
    pub hdr_vec: Vec<(TextureId, IblId)>,
    pub wait_for_exit: bool,
    pub editor_service: EditorService,
    last_ui_update: std::time::Instant,
    statistics_dt: f32,
}

impl Runtime {
    pub fn new(window: Arc<Window>) -> Self {
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
        let gpu_cache = GpuCache {
            textures: GpuTextureCache::new(&gpu_context.as_ref()),
            material: GpuMaterialCache::default(),
            mesh: GpuMeshCache::default(),
        };
        let (width, height) = (
            gpu_surface.get_config().width,
            gpu_surface.get_config().height,
        );
        let gpu_manager = GpuManager::new(&gpu_context.as_ref(), width, height);
        let shadow_manager = ShadowManager::new(&gpu_context.as_ref());
        let ibl_manager = IblManager::new(&gpu_context.as_ref());
        let pipeline_manager = PipelineManager::new(
            &gpu_context.device,
            &gpu_manager,
            gpu_surface.get_config().format,
        );
        let scene_renderer = SceneRenderer::new();
        let (connection, channels) = EditorConnection::new();
        let editor_service = EditorService::new(channels);
        let uilayer = UiLayer::new(
            &window,
            imgui_context,
            gpu_context.get_adapter_string(),
            connection,
        );
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
            readback: ReadbackManager::default(),
            editor_service,
            last_ui_update: std::time::Instant::now(),
            statistics_dt: 1.0 / 60.0,
        }
    }
    pub fn handle_winit_event(&mut self, event: &Event<()>) {
        self.uilayer.handle_event(&self.window, event);
        match event {
            Event::WindowEvent { .. } | Event::DeviceEvent { .. }
                if !self.uilayer.want_capture_mouse() =>
            {
                self.input.update_events(&event)
            }
            _ => {}
        }
    }
    pub fn handle_input(&mut self, bus: &mut EventBus) {
        use crate::input::MouseButton;
        let input = &self.input;
        if let Some(result) = self.readback.poll_results() {
            match result {
                QueryResult::Pick(id) => {
                    bus.send_domain(Selection(Hovered(id.map(Entity::from_raw_u64))))
                }
                QueryResult::Selection(ids) => {
                    bus.send_domain(Selection(Select(ids)));
                }
            }
        }
        if input.is_cursor_moved() {
            self.readback.request_pick(
                &self.gpu_context.as_ref(),
                &self
                    .gpu_manager
                    .get_framebuffer_texture(crate::gpu::FramebufferKind::EntityId),
                (input.mouse_position.x as u32, input.mouse_position.y as u32),
            );
        }

        match self.editor_interaction {
            EditorInteraction::None => {
                if input.is_mouse_button_pressed(MouseButton::Left)
                    && input.is_key_down(KeyButton::Control)
                {
                    let current = input.mouse_position;
                    self.editor_interaction = EditorInteraction::Selecting {
                        start: current,
                        current,
                    };
                }
            }
            EditorInteraction::Selecting { start, current: _ } => {
                if input.is_mouse_dragging(MouseButton::Left)
                    && input.is_key_down(KeyButton::Control)
                {
                    self.editor_interaction = EditorInteraction::Selecting {
                        start,
                        current: input.mouse_position,
                    };
                }
                if input.is_mouse_button_released(MouseButton::Left) {
                    let current = input.mouse_position;
                    let pos = (start.x.min(current.x) as u32, start.y.min(current.y) as u32);
                    let width = (start.x - current.x).abs() as u32;
                    let height = (start.y - current.y).abs() as u32;
                    self.readback.request_selection(
                        &self.gpu_context.as_ref(),
                        &self
                            .gpu_manager
                            .get_framebuffer_texture(crate::gpu::FramebufferKind::EntityId),
                        pos,
                        (width, height),
                    );
                    self.editor_interaction = EditorInteraction::None;
                }
            }
        }
        self.uilayer.set_editor_interaction(self.editor_interaction);
        if input.is_mouse_dragging(MouseButton::Left) && input.any_key_down() {
            bus.send_domain(Camera(CameraOrbit(
                input.mouse_delta.x as f64,
                input.mouse_delta.y as f64,
            )));
        }
        if input.is_mouse_dragging(MouseButton::Middle) {
            bus.send_domain(Camera(CameraPan(
                input.mouse_delta.x as f64,
                input.mouse_delta.y as f64,
            )));
        }
        if let Some(delta) = input.mouse_wheel_movement {
            bus.send_domain(Camera(CameraZoom(delta.y)));
        }
        self.input.clear();
    }
    pub fn handle_runtime_events<A: Application>(&mut self, app: &mut A, bus: &mut EventBus) {
        for event in bus.drain_runtime() {
            match event {
                RuntimeEvent::Resize { width, height } => {
                    if width == 0 || height == 0 {
                        return;
                    }
                    self.gpu_manager
                        .resize_frame(&self.gpu_context.as_ref(), width, height);
                    self.gpu_surface
                        .resize_frame(&self.gpu_context.device, width, height);
                    app.on_resize(width, height);
                }
                RuntimeEvent::CloseRequested => {
                    app.on_close();
                    self.wait_for_exit = true;
                }
                RuntimeEvent::DroppedFile(path) => app.on_drop(path, bus),
                RuntimeEvent::SetWindowTitle(title) => {
                    self.window.set_title(&title);
                    info!("Set window title");
                }
                RuntimeEvent::SyncImguiTextures => self
                    .imgui_render
                    .sync_imgui_texture(&self.gpu_context, &mut self.gpu_cache.textures),
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
    pub fn sync_gpu_assets(&mut self, asset_mgr: &mut AssetManager, bus: &mut EventBus) {
        use crate::assets::asset_manager::AssetEventKind;
        use crate::assets::material_asset::MaterialAsset;
        use crate::assets::mesh_asset::MeshAsset;
        use crate::assets::texture_asset::{TextureAsset, TextureDesc};
        use crate::assets::texture_upload::load_cpu_textures_par;
        use crate::gpu::texture::GpuTextureBuilder;
        use crate::gpu::{GpuMaterial, GpuMesh};
        let Self {
            gpu_context,
            ibl_manager,
            gpu_manager,
            ..
        } = self;
        let texture_cache = &mut self.gpu_cache.textures;
        let material_cache = &mut self.gpu_cache.material;
        let mesh_cache = &mut self.gpu_cache.mesh;
        let grouped = asset_mgr.drain_grouped_events();
        grouped.process_type::<TextureAsset, _>(|kind, events| match kind {
            AssetEventKind::Created => {
                let jobs: Vec<(TextureId, TextureDesc)> = events
                    .iter()
                    .filter_map(|ev| {
                        asset_mgr
                            .get::<TextureAsset>(ev.id)
                            .map(|a| (ev.id, a.desc.clone()))
                    })
                    .collect();
                for (id, data) in load_cpu_textures_par(jobs) {
                    texture_cache.insert(
                        id,
                        GpuTextureBuilder::from_cpu(data).build(&gpu_context.as_ref()),
                    );
                }
            }
            AssetEventKind::Removed => events.iter().for_each(|ev| texture_cache.remove(ev.id)),
            _ => {}
        });
        grouped.process_type::<IblAsset, _>(|kind, events| {
            if let AssetEventKind::Created = kind {
                events
                    .iter()
                    .filter_map(|ev| asset_mgr.get::<IblAsset>(ev.id).map(|a| (ev.id, a)))
                    .for_each(|(id, asset)| {
                        if let Some(hdr) = texture_cache.get(asset.hrd_id) {
                            ibl_manager.insert(id, ibl_manager.create(hdr, &gpu_context.as_ref()));
                            self.hdr_vec.push((asset.hrd_id, id));
                            bus.send_domain(Selection(SelectIbl(id)));
                            bus.send_runtime(RuntimeEvent::UpdateIblMaps(id));
                        }
                    });
            }
        });
        grouped.process_type::<MaterialAsset, _>(|kind, events| match kind {
            AssetEventKind::Created => events
                .iter()
                .filter_map(|ev| asset_mgr.get::<MaterialAsset>(ev.id).map(|a| (ev.id, a)))
                .for_each(|(id, asset)| {
                    let layout = gpu_manager.get_bindgroup_layout(BindgroupLayoutKind::Material);
                    material_cache.insert(
                        id,
                        GpuMaterial::new(&texture_cache, &asset.desc, &gpu_context.device, layout),
                    );
                }),
            AssetEventKind::Updated => events
                .iter()
                .filter_map(|ev| {
                    asset_mgr
                        .get::<MaterialAsset>(ev.id)
                        .map(|a| (ev.id, &a.desc))
                })
                .for_each(|(id, desc)| {
                    material_cache.update(&id, |m| m.update_uniform(&gpu_context.queue, desc))
                }),
            AssetEventKind::Removed => events.iter().for_each(|ev| material_cache.remove(ev.id)),
            _ => {}
        });
        grouped.process_type::<MeshAsset, _>(|kind, events| match kind {
            AssetEventKind::Created => events
                .iter()
                .filter_map(|ev| asset_mgr.get::<MeshAsset>(ev.id).map(|a| (ev.id, a)))
                .for_each(|(id, asset)| {
                    mesh_cache.insert(
                        id,
                        GpuMesh::new(
                            &gpu_context.device,
                            &asset.desc.vertices,
                            &asset.desc.indices,
                        ),
                    )
                }),
            AssetEventKind::Removed => events.iter().for_each(|ev| mesh_cache.remove(ev.id)),
            _ => {}
        });
        grouped.process_type::<TextureAsset, _>(|_, _| {
            bus.send_runtime(RuntimeEvent::SyncImguiTextures)
        });
    }
    pub fn update_ui<A: Application>(&mut self, app: &mut A, bus: &mut EventBus) {
        let now = std::time::Instant::now();
        let dt = now
            .duration_since(self.last_ui_update)
            .as_secs_f32()
            .clamp(1.0 / 1000.0, 0.25);
        self.last_ui_update = now;
        self.statistics_dt = self.statistics_dt * 0.9 + dt * 0.1;
        let frame = self.scene_renderer.get_render_stats();
        self.editor_service.set_statistics(EditorStatisticsData {
            fps: 1.0 / self.statistics_dt,
            frametime: self.statistics_dt,
            adapter_name: self.gpu_context.get_adapter_string(),
            root_nodes: 0,
            opaque_draw_calls: frame.opaque.draw_calls,
            opaque_instances: frame.opaque.instances,
            transmission_draw_calls: frame.transmission.draw_calls,
            transmission_instances: frame.transmission.instances,
        });
        self.editor_service.process(app, bus);
        self.uilayer.build(&self.window);
    }
    pub fn render<A: Application>(&mut self, app: &A) {
        let mut encoder = self.gpu_context.create_encoder();
        if let Some(frame) = self.gpu_surface.get_frame() {
            let target = frame.texture.create_view(&Default::default());
            let frame_data = self.prepare_frame_data(app.render_data());
            let context = SceneRenderContext {
                gpu_context: &self.gpu_context,
                gpu_manager: &self.gpu_manager,
                shadow_manager: &self.shadow_manager,
                pipeline_manager: &self.pipeline_manager,
                gpu_cache: &self.gpu_cache,
            };
            self.scene_renderer
                .render(&context, &mut encoder, &target, &frame_data);
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
    fn prepare_frame_data(&mut self, render_data: AppRenderData) -> FrameData {
        let AppRenderData {
            render_objects,
            asset_mgr,
            camera,
            globals,
            selected,
        } = render_data;
        let frame = FrameBuilder::prepare(render_objects, asset_mgr, globals);
        let camera_uniform = CameraUniform::from_camera_size(
            camera,
            (
                self.gpu_surface.get_config().width,
                self.gpu_surface.get_config().height,
            ),
        );
        let global_uniform =
            GlobalUniform::from_global_id(globals, selected.map(|id| id.as_raw_u64()).unwrap_or(0));
        self.gpu_manager.update_buffer(
            &self.gpu_context.queue,
            BufferKind::Lights,
            std::slice::from_ref(&frame.light_uniform),
        );
        self.gpu_manager.update_buffer(
            &self.gpu_context.queue,
            BufferKind::Camera,
            std::slice::from_ref(&camera_uniform),
        );
        self.gpu_manager.update_buffer(
            &self.gpu_context.queue,
            BufferKind::Globals,
            std::slice::from_ref(&global_uniform),
        );
        self.gpu_manager.update_buffer(
            &self.gpu_context.queue,
            BufferKind::Instances,
            frame.instances.as_slice(),
        );
        self.gpu_manager.update_buffer(
            &self.gpu_context.queue,
            BufferKind::Lines,
            frame.lines.as_slice(),
        );
        let tasks = FrameTasks {
            axis_enable: globals.axis_enable,
            build_mips_cp: globals.mips_cp,
            entity_selected: selected,
            skybox_enable: globals.skybox_enable,
            skybox_blur: globals.skybox_enable_blur,
        };
        FrameData {
            opaque_batches: frame.opaque_batches,
            transmission_batches: frame.transmission_batches,
            transmission_stats: frame.transmission_stats,
            lights: Some(frame.light_uniform),
            lines: frame.lines,
            opaque_stats: frame.opaque_stats,
            tasks,
        }
    }
}

use super::RuntimeEvent;
use super::gpu_sync::TextureLoadService;
use crate::app::Application;
use crate::ecs::entity_id::EntityRawU64;
use crate::app::application::AppRenderData;
use crate::app::domain::events::CameraEvent::{CameraOrbit, CameraPan, CameraZoom};
use crate::app::domain::events::DomainEvent::Camera;
use crate::assets::asset_manager::AssetManager;
use crate::assets::asset_manager::ResourceStats;
use crate::assets::{IblId, TextureId};
use crate::editor::{
    EditorConnection, EditorResourceStatsData, EditorStatisticsData, ResourceStatsData,
};
use crate::engine::editor::{EditorBackend, EditorService};
use crate::engine::engine::EventBus;
use crate::engine::picking::PickingService;
use crate::engine::readback::ReadbackManager;
use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{
    BufferKind, GpuCache, GpuContext, GpuManager, GpuMaterialCache, GpuMeshCache, GpuSurface,
    GpuTextureCache, HasGpuStats, IblManager, ShadowManager,
};
use crate::input::Input;
use crate::prelude::info;
use crate::renderer::FrameData;
use crate::renderer::ImguiRender;
use crate::renderer::framebuilder::{FrameBuilder, FrameTasks};
use crate::renderer::scene_renderer::SceneRenderContext;
use crate::renderer::uniform::{CameraUniform, GlobalUniform};
use crate::renderer::{MaterialPreviewRenderer, SceneRenderer};
use crate::ui::UiLayer;
use crate::winit_bridge::WindowHandle;

pub struct Runtime {
    pub window: WindowHandle,
    pub gpu_context: GpuContext,
    pub gpu_surface: GpuSurface,
    pub gpu_cache: GpuCache,
    pub gpu_manager: GpuManager,
    pub ibl_manager: IblManager,
    pub pipeline_manager: PipelineManager,
    pub shadow_manager: ShadowManager,
    readback: ReadbackManager,
    picking: PickingService,
    pub uilayer: UiLayer,
    pub input: Input,
    pub scene_renderer: SceneRenderer,
    pub material_preview_renderer: MaterialPreviewRenderer,
    pub imgui_render: ImguiRender,
    pub hdr_vec: Vec<(TextureId, IblId)>,
    pub wait_for_exit: bool,
    pub editor_service: EditorService,
    texture_loader: TextureLoadService,
    last_ui_update: std::time::Instant,
    statistics_dt: f32,
}

impl Runtime {
    fn asset_stats(stats: ResourceStats) -> ResourceStatsData {
        ResourceStatsData {
            count: stats.count,
            estimated_bytes: stats.estimated_bytes,
        }
    }

    fn gpu_stats(stats: crate::gpu::GpuResourceStats) -> ResourceStatsData {
        ResourceStatsData {
            count: stats.count,
            estimated_bytes: stats.estimated_bytes,
        }
    }

    pub(crate) fn new(window: WindowHandle) -> Self {
        let mut imgui_context = imgui::Context::create();
        let gpu_context = GpuContext::default();
        let gpu_surface = GpuSurface::new(
            gpu_context.adapter(),
            gpu_context.instance(),
            window.clone(),
        );
        let mut imgui_render = ImguiRender::new(
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
        imgui_render.registry.add(
            &mut imgui_render.renderer,
            &gpu_context.device,
            shadow_manager.get_rgba_id(),
            shadow_manager.get_rgba(),
        );
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
            material_preview_renderer: MaterialPreviewRenderer::new(&gpu_context.device),
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
            picking: PickingService::default(),
            editor_service,
            texture_loader: TextureLoadService::new(),
            last_ui_update: std::time::Instant::now(),
            statistics_dt: 1.0 / 60.0,
        }
    }

    pub fn handle_input(&mut self, bus: &mut EventBus) {
        use crate::input::MouseButton;
        let entity_id_texture = self
            .gpu_manager
            .get_framebuffer_texture(crate::gpu::FramebufferKind::EntityId);
        self.picking.handle_input(
            &mut self.readback,
            &self.input,
            &self.gpu_context.as_ref(),
            entity_id_texture,
            bus,
            &self.editor_service,
        );
        let input = &self.input;
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

    pub fn handle_runtime_events<A: Application>(
        &mut self,
        app: &mut A,
        bus: &mut EventBus,
    ) -> Option<String> {
        let mut window_title = None;
        for event in bus.drain_runtime() {
            match event {
                RuntimeEvent::Resize { width, height } => {
                    if width == 0 || height == 0 {
                        return window_title;
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
                    window_title = Some(title);
                    info!("Set window title");
                }
                RuntimeEvent::UpdateIblMaps(id) => {
                    self.material_preview_renderer.invalidate_environment();
                    self.gpu_manager.replace_pbrmap_skybox_bindgroup(
                        self.ibl_manager.get(&id),
                        &self.shadow_manager,
                        &self.gpu_context.device,
                    );
                }
                RuntimeEvent::ReadbackSelection(pos, size) => {
                    self.picking.request_selection(
                        &mut self.readback,
                        &self.gpu_context.as_ref(),
                        &self
                            .gpu_manager
                            .get_framebuffer_texture(crate::gpu::FramebufferKind::EntityId),
                        pos,
                        size,
                    );
                }
            }
        }
        window_title
    }

    pub fn sync_gpu_assets(&mut self, asset_mgr: &mut AssetManager, bus: &mut EventBus) {
        super::gpu_sync::sync_gpu_assets(
            asset_mgr,
            bus,
            &self.gpu_context,
            &mut self.gpu_cache,
            &mut self.texture_loader,
            &mut self.gpu_manager,
            &mut self.ibl_manager,
            &mut self.imgui_render,
            &mut self.hdr_vec,
            &mut self.material_preview_renderer,
        );
    }

    pub fn update_ui<A: Application + EditorBackend>(&mut self, app: &mut A, bus: &mut EventBus) {
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
            opaque_draw_calls: frame.opaque.draw_calls,
            opaque_instances: frame.opaque.instances,
            transmission_draw_calls: frame.transmission.draw_calls,
            transmission_instances: frame.transmission.instances,
        });

        let asset_mgr = app.render_data().asset_mgr;
        self.editor_service
            .set_resource_stats(EditorResourceStatsData {
                textures: Self::asset_stats(asset_mgr.get_stats::<crate::assets::TextureAsset>()),
                materials: Self::asset_stats(asset_mgr.get_stats::<crate::assets::MaterialAsset>()),
                meshes: Self::asset_stats(asset_mgr.get_stats::<crate::assets::MeshAsset>()),
                ibl: Self::asset_stats(asset_mgr.get_stats::<crate::assets::IblAsset>()),
                gpu_textures: Self::gpu_stats(self.gpu_cache.textures.get_stats()),
                gpu_materials: Self::gpu_stats(self.gpu_cache.material.get_stats()),
                gpu_meshes: Self::gpu_stats(self.gpu_cache.mesh.get_stats()),
                gpu_shadows: Self::gpu_stats(self.shadow_manager.get_stats()),
                gpu_ibl: Self::gpu_stats(self.ibl_manager.get_stats()),
            });
        self.editor_service.process(app, bus);
        let textures = self.imgui_render.registry.ui_textures(
            self.shadow_manager.get_rgba_id(),
            self.material_preview_renderer.resource_id(),
        );
        let output = self.uilayer.build(&self.window, &textures);
        self.material_preview_renderer
            .set_material(output.material_preview);
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

            if let Some(preview_texture) = self.material_preview_renderer.render(
                &mut encoder,
                &self.gpu_context,
                &self.gpu_manager,
                &self.gpu_cache,
                &self.pipeline_manager,
            ) {
                self.imgui_render.registry.add(
                    &mut self.imgui_render.renderer,
                    &self.gpu_context.device,
                    self.material_preview_renderer.resource_id(),
                    &preview_texture,
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

    fn prepare_frame_data(&mut self, render_data: AppRenderData) -> FrameData {
        let AppRenderData {
            render_objects,
            asset_mgr,
            camera,
            globals,
            selected,
        } = render_data;
        let frame = FrameBuilder::prepare(
            render_objects,
            asset_mgr,
            globals,
            &self.gpu_cache,
            &self.texture_loader,
        );
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

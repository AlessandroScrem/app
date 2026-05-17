use std::sync::Arc;

use super::RuntimeEvent;
use crate::UiLayer;
use crate::app::app_impl::RuntimeContext;
use crate::app::Application;
use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{
    GpuCache, GpuContext, GpuInternalCounters, GpuManager, GpuSurface, HasGpuStats, InternalCounter,
};
use crate::input::Input;
use crate::picking::PickObject;
use crate::prelude::*;
use crate::renderer::ImguiRender;
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

    pub fn tick<A: Application>(&mut self, app: &mut A) {
        let events = std::mem::take(&mut self.events);
        for event in events {
            self.handle_runtime_event(app, event);
        }

        app.update(self);

        self.render(app);

        // Clear Input
        self.input.clear();
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

                let mut context = RuntimeContext {
                    gpu_context,
                    gpu_manager,
                    pipeline_manager,
                    gpu_cache,
                    input,
                    pickobject,
                };

                scene_renderer.render(
                    &mut context,
                    &mut encoder,
                    &target,
                    size,
                    render_data.asset_mgr,
                    render_data.world,
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
                self.gpu_manager
                    .update_ibl_bind_group(&self.gpu_context.device);

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

use std::sync::Arc;

use super::RuntimeEvent;
use crate::UiLayer;
use crate::app::Application;
use crate::gpu::pipeline_manager::PipelineManager;
use crate::gpu::{
    GpuCache, GpuContext, GpuInternalCounters, GpuManager, GpuSurface, HasGpuStats, InternalCounter
};
use crate::input::Input;
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
    pub is_minimized: bool,
    pub timer: Timer,
    pub input: Input,

    pub events: Vec<RuntimeEvent>,
    pub scene_renderer: SceneRenderer,
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
        if self.is_minimized {
            return;
        }

        let events = std::mem::take(&mut self.events);
        for event in events {
            self.handle_runtime_event(app, event);
        }

        app.update(self);

        // Render
        app.render(self);

        // Clear Input
        self.input.clear();
    }

    fn handle_runtime_event<A: Application>(&mut self, app: &mut A, event: RuntimeEvent) {
        match event {
            RuntimeEvent::Resize { width, height } => {
                if width > 0 && height > 0 {
                    self.is_minimized = false;
                    self.gpu_manager
                        .resize_frame(&self.gpu_context.device, width, height);
                    self.gpu_surface
                        .resize_frame(&self.gpu_context.device, width, height);
                    app.on_resize(width, height);
                } else {
                    self.is_minimized = true;
                }
            }
            RuntimeEvent::CloseRequested => {
                app.on_close();
            }
        }
    }
}

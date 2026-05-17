use crate::{RenderStats, UiSnapshot, engine::RunningApp, gpu::HasStats};

use super::app::App;

impl App {
    pub fn update_uilayer(&mut self, runtime: &mut RunningApp) {
        let uilayer = &mut runtime.uilayer;
        let window = &runtime.window;
        let counter_trait = &runtime.gpu_cache;

        let render_stats = RenderStats {
            texture: self.asset_mgr.textures.get_stats(),
            mesh: self.asset_mgr.meshes.get_stats(),
            material: self.asset_mgr.materials.get_stats(),
            frame: runtime.scene_renderer.get_render_stats(),
        };

        let snapshot = UiSnapshot::from_world(
            &self.current_scene.world,
            self.selected,
            &self.asset_mgr,
            &self.camera,
            &self.globals,
            &runtime.imgui_render, //resolver trait
            counter_trait,         // internalcounter trait
            None,                  // no debug texture_id
            render_stats,
        );

        // Main operation: update_ui
        let mut events = uilayer.build(window, snapshot);
        self.domain_events.queue.append(&mut events);
    }
}

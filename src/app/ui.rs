use crate::{RenderStats, UiSnapshot, engine::UiRuntimeContext, gpu::HasStats};

use super::app::App;

impl App {
    pub fn update_uilayer(&mut self, ctx: UiRuntimeContext<'_>) {
        let render_stats = RenderStats {
            texture: self.asset_mgr.textures.get_stats(),
            mesh: self.asset_mgr.meshes.get_stats(),
            material: self.asset_mgr.materials.get_stats(),
            frame: ctx.frame_stats,
        };

        let snapshot = UiSnapshot::from_world(
            &self.current_scene.world,
            self.selected,
            &self.asset_mgr,
            &self.camera,
            &self.globals,
            ctx.imgui_render, //resolver trait
            ctx.gpu_cache,    // internalcounter trait
            None,                  // no debug texture_id
            render_stats,
        );

        // Main operation: update_ui
        let mut events = ctx.uilayer.build(ctx.window, snapshot);
        self.domain_events.queue.append(&mut events);
    }
}

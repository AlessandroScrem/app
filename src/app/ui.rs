use crate::{RenderStats, UiSnapshot, ui::UiRuntimeContext};
use crate::assets::global_asset_manager::resource_stats::ResourceStats;

use super::app::App;

impl App {
    pub fn update_uilayer(&mut self, ctx: UiRuntimeContext<'_>) {
        let render_stats = RenderStats {
            texture: ResourceStats::default(),
            mesh: ResourceStats::default(),
            material: ResourceStats::default(),
            // texture: self.asset_mgr.textures.get_stats(),
            // mesh: self.asset_mgr.meshes.get_stats(),
            // material: self.asset_mgr.materials.get_stats(),
            frame: ctx.frame_stats,
        };

        let snapshot = UiSnapshot::from_world(
            &self.current_scene.world,
            self.selected,
            &self.asset_mgr,
            &self.camera,
            &self.globals,
            ctx.texture_resolver,
            ctx.gpu_counters,
            None, // no debug texture_id
            render_stats,
        );

        // Main operation: update_ui
        let mut events = ctx.uilayer.build(ctx.window, snapshot);
        self.domain_events.queue.append(&mut events);
    }
}

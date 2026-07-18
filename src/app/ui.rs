use crate::ui::{RenderStats, UiSnapshot, UiRuntimeContext};
use crate::assets::{IblAsset, MaterialAsset, MeshAsset, TextureAsset};

use super::app::App;

impl App {
    pub fn update_uilayer(&mut self, ctx: UiRuntimeContext<'_>) {
        let render_stats = RenderStats {
            texture: self.asset_mgr.get_stats::<TextureAsset>(),
            mesh: self.asset_mgr.get_stats::<MeshAsset>(),
            material: self.asset_mgr.get_stats::<MaterialAsset>(),
            frame: ctx.frame_stats,
        };

        let hdr_id = self.ibl_id.and_then(|ibl_id| {
            self.asset_mgr
                .get::<IblAsset>(ibl_id)
                .map(|asset| asset.hrd_id)
        });


        let snapshot = UiSnapshot::from_world(
            &self.current_scene.world,
            self.selected,
            &self.asset_mgr,
            &self.camera,
            &self.globals,
            ctx.texture_resolver,
            ctx.gpu_counters,
            ctx.debug_texture,
            render_stats,
            hdr_id,
            self.current_scene.filename.clone()
        );

        // Main operation: update_ui
        let mut events = ctx.uilayer.build(ctx.window, snapshot);
        self.domain_events.queue.append(&mut events);
    }
}

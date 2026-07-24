use crate::app::domain::events::DomainEvents;
use crate::assets::MaterialAsset;
use crate::assets::MeshAsset;
use crate::assets::TextureAsset;
use crate::assets::asset_manager::AssetManager;
use crate::assets::asset_manager::GlobalAssetId;
use crate::engine::RuntimeEvent;
use crate::gpu::GpuInternalCounters;
use crate::prelude::*;

use crate::globals::Globals;

use crate::renderer::scene_renderer::FrameStats;
use crate::scene::Scene;
use crate::ui::RenderStats;
use crate::ui::UiSnapshot;
use crate::ui::UiTexture;
use crate::ui::UiTextureResolver;

use legion::Entity;

#[derive(Default)]
pub struct App {
    pub current_scene: Scene,
    pub asset_mgr: AssetManager,
    pub globals: Globals,
    pub camera: Camera,
    pub domain_events: DomainEvents,
    pub selected: Option<Entity>,
    pub hovered: Option<Entity>,
    pub exit_requested: bool,
    pub ibl_id: Option<GlobalAssetId>,
    #[allow(unused)]
    pub debug_texture_id: Option<UiTexture>,
}

impl App {
    pub fn update_scene(&mut self, events: &mut Vec<RuntimeEvent>) {
        self.current_scene.update_scene(events);
    }

    pub fn get_scene_snapshot<'a>(
        &'a self,
        texture_resolver: &'a dyn UiTextureResolver,
        frame_stats: FrameStats,
        gpu_counters: GpuInternalCounters,
        hdr_id: Option<GlobalAssetId>,
    ) -> UiSnapshot<'a> {
        let root_snapshot = self.current_scene.get_roots();
        let comp_state = self
            .current_scene
            .get_selected_componet_state(self.selected, &self.asset_mgr);

        let render_stats = RenderStats {
            gpu_counters,
            frame_stats,
            texture: self.asset_mgr.get_stats::<TextureAsset>(),
            mesh: self.asset_mgr.get_stats::<MeshAsset>(),
            material: self.asset_mgr.get_stats::<MaterialAsset>(),
        };

        UiSnapshot {
            texture_resolver,
            render_stats,
            camera: &self.camera,
            globals: &self.globals,
            root_snapshot,
            comp_state,
            selected: self.selected,
            hovered: self.hovered,
            debug_texture_id: self.debug_texture_id,
            hdr_id,
            scene_name: self.current_scene.filename.clone(),
        }
    }
}

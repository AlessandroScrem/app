use crate::app::domain::events::DomainEvents;
use crate::assets::asset_manager::AssetManager;
use crate::assets::asset_manager::GlobalAssetId;
use crate::Globals;
use crate::Camera;
use crate::scene::Scene;
use crate::ui::UiTexture;
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

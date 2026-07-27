use crate::assets::IblId;
use crate::assets::asset_manager::AssetManager;
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
    pub selected: Option<Entity>,
    pub hovered: Option<Entity>,
    pub selected_ibl: Option<IblId>,
    #[allow(unused)]
    pub debug_texture_id: Option<UiTexture>,
}

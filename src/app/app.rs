use crate::app::Settings;
use crate::assets::IblId;
use crate::assets::asset_manager::AssetManager;
use crate::Globals;
use crate::Camera;
use crate::scene::Scene;
use legion::Entity;

#[derive(Default)]
pub struct App {
    pub current_scene: Scene,
    pub asset_mgr: AssetManager,
    pub globals: Globals,
    pub camera: Camera,
    pub selected: Option<Entity>,
    pub multiselct: Vec<u64>,
    pub hovered: Option<Entity>,
    pub selected_ibl: Option<IblId>,
    pub settings: Settings,
    pub(crate) editor_scene_revision: u64,
    pub(crate) transform_edit: Option<(Entity, crate::editor::TransformData)>,
}

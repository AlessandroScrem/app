use std::path::PathBuf;

use crate::engine::RuntimeEvent;
use crate::input::Input;
use crate::ui::UiRuntimeContext;
use crate::{Camera, Globals};
use legion::{Entity, World};

use crate::assets::asset_manager::AssetManager;

pub struct AppRenderData<'a> {
    pub asset_mgr: &'a AssetManager,
    pub world: &'a World,
    pub camera: &'a Camera,
    pub globals: &'a Globals,
    pub selected: Option<Entity>,
}

pub trait HasAssetMgr {
    fn asset_mgr_mut(&mut self) -> &mut AssetManager;
}

pub trait HandlesPicking {
    fn set_hovered(&mut self, hovered: Option<Entity>);
}

pub trait HasUi {
    fn update_ui(&mut self, ctx: UiRuntimeContext<'_>);
}

pub trait RuntimeApp: Application + HasAssetMgr + HandlesPicking + HasUi {}

impl<T> RuntimeApp for T where T: Application + HasAssetMgr + HandlesPicking + HasUi {}

pub trait Application {
    fn init(&mut self);
    fn update(&mut self, input: &Input) -> Option<RuntimeEvent>;
    fn render_data(&self) -> AppRenderData<'_>;
    fn on_resize(&mut self, width: u32, height: u32);
    fn on_drop(&mut self, path: PathBuf);
    fn on_close(&mut self);
    fn exit_requested(&self) -> bool;
}

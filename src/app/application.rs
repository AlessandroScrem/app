use std::path::PathBuf;

use crate::engine::RunningApp;
use crate::{AssetManager, Camera, Globals};
use legion::{Entity, World};

pub struct AppRenderData<'a> {
    pub asset_mgr: &'a AssetManager,
    pub world: &'a World,
    pub camera: &'a Camera,
    pub globals: &'a Globals,
    pub selected: Option<Entity>,
}

pub trait Application {
    fn init(&mut self);
    fn update(&mut self, runtime: &mut RunningApp);
    fn render_data(&self) -> AppRenderData<'_>;
    fn on_resize(&mut self, width: u32, height: u32);
    fn on_drop(&mut self, path: PathBuf);
    fn on_close(&mut self);
    fn exit_requested(&self) -> bool;
}
